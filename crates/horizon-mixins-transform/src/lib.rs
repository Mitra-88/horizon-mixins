use horizon_mixins_core::{
    Diagnostic, Plan, PlannedInjection, Result,
    index::{Function, tokens},
    manifest::Kind,
    plan, source,
};
use quote::quote;
use std::{collections::BTreeMap, path::Path};
use syn::{
    Block, Expr, Item, Signature,
    visit_mut::{self, VisitMut},
};

/// Rewrite only after validating all original fingerprints. No filesystem mutation.
pub fn transform(root: &Path, plan: &Plan) -> Result<BTreeMap<String, Vec<u8>>> {
    if plan.schema != 1 || plan.transformer != env!("CARGO_PKG_VERSION") {
        return Err(Diagnostic::new(
            "E_PLAN_VERSION",
            "unsupported plan schema/transformer",
        ));
    }
    plan.validate_structure()?;
    if plan.rustc != plan::rustc(root)? {
        return Err(Diagnostic::new(
            "E_TOOLCHAIN",
            "rustc differs from the planned toolchain",
        ));
    }
    let mut files = source::files(root)?;
    let fingerprints: BTreeMap<_, _> = files
        .iter()
        .map(|(name, bytes)| (name.clone(), source::fingerprint(bytes)))
        .collect();
    if fingerprints != plan.inputs {
        return Err(Diagnostic::new(
            "E_STALE_SOURCE",
            "workspace no longer matches planned inputs; regenerate plan",
        ));
    }
    let mut groups: BTreeMap<(&str, &[usize]), Vec<&PlannedInjection>> = BTreeMap::new();
    for patch in &plan.injections {
        groups
            .entry((&patch.file, &patch.node))
            .or_default()
            .push(patch);
    }
    let mut trees = BTreeMap::new();
    for (file, _) in groups.keys() {
        if !trees.contains_key(*file) {
            let bytes = files
                .get(*file)
                .ok_or_else(|| Diagnostic::new("E_STALE_SOURCE", "planned file is absent"))?;
            let text = std::str::from_utf8(bytes)
                .map_err(|e| Diagnostic::new("E_RUST_SYNTAX", e.to_string()))?;
            trees.insert(
                file.to_string(),
                syn::parse_file(text)
                    .map_err(|e| Diagnostic::new("E_RUST_SYNTAX", e.to_string()).file(file))?,
            );
        }
    }
    for ((file, node), mut patches) in groups {
        patches.sort_by(|a, b| {
            a.call_site
                .cmp(&b.call_site)
                .then(b.priority.cmp(&a.priority))
                .then(a.mod_id.cmp(&b.mod_id))
                .then(a.injection_id.cmp(&b.injection_id))
        });
        let tree = trees.get_mut(file).unwrap();
        let (signature, block) = locate(&mut tree.items, node)?;
        let fingerprint = source::fingerprint(format!("{}{}", tokens(signature), tokens(block)));
        for patch in &patches {
            if patch.function_fingerprint != fingerprint {
                return Err(
                    Diagnostic::new("E_STALE_SOURCE", "function fingerprint mismatch").file(file),
                );
            }
            let f = Function {
                id: patch.target.clone(),
                file: file.into(),
                node: node.to_vec(),
                signature: signature.clone(),
                block: block.clone(),
                unsupported_cfg: false,
                unsupported_macro: false,
                generic_impl: false,
            };
            plan::validate_helper(&f, patch.kind, &patch.helper_source, &patch.bindings)?;
        }
        if patches.iter().any(|p| p.kind == Kind::Overwrite) {
            if patches.len() != 1 {
                return Err(Diagnostic::new(
                    "E_OVERWRITE_CONFLICT",
                    "OVERWRITE must be exclusive",
                ));
            }
            // The helper call is the tail expression, preserving the target signature's return type.
            let expr = helper_call(patches[0])?;
            *block = syn::parse2(quote!({ #expr })).map_err(syntax)?;
            continue;
        }
        let calls: BTreeMap<usize, Vec<&PlannedInjection>> = patches
            .iter()
            .filter(|p| p.kind == Kind::BeforeCall)
            .fold(BTreeMap::new(), |mut groups, p| {
                groups.entry(p.call_site.unwrap()).or_default().push(*p);
                groups
            });
        let expected = calls.len();
        let mut rewrite = Rewrite {
            calls,
            cursor: 0,
            matched: 0,
            error: None,
            next_discards_result: false,
        };
        rewrite.visit_block_mut(block);
        if let Some(e) = rewrite.error {
            return Err(e.file(file));
        }
        if rewrite.matched != expected {
            return Err(
                Diagnostic::new("E_STALE_SOURCE", "planned call site disappeared").file(file),
            );
        }
        let mut head = Vec::new();
        for patch in patches.iter().filter(|p| p.kind == Kind::Head) {
            let expr = helper_call(patch)?;
            head.push(syn::parse2(quote!(#expr;)).map_err(syntax)?);
        }
        head.append(&mut block.stmts);
        block.stmts = head;
    }
    for (file, tree) in trees {
        files.insert(file, prettyplease::unparse(&tree).into_bytes());
    }
    Ok(files)
}
fn syntax(e: syn::Error) -> Diagnostic {
    Diagnostic::new("E_TRANSFORM", e.to_string())
}
fn helper_call(patch: &PlannedInjection) -> Result<Expr> {
    let helper = plan::parse_helper(&patch.helper_source)?;
    let name = &helper.sig.ident;
    let values: Vec<Expr> = patch
        .bindings
        .iter()
        .map(|b| syn::parse_str(&b.value).map_err(syntax))
        .collect::<Result<_>>()?;
    syn::parse2(quote!({ #helper #name(#(#values),*) })).map_err(syntax)
}
struct Rewrite<'a> {
    calls: BTreeMap<usize, Vec<&'a PlannedInjection>>,
    cursor: usize,
    matched: usize,
    error: Option<Diagnostic>,
    next_discards_result: bool,
}
impl VisitMut for Rewrite<'_> {
    fn visit_stmt_mut(&mut self, stmt: &mut syn::Stmt) {
        // Retain the original call's own statement boundary when its value is
        // discarded, so a returned Drop value is destroyed before its receiver.
        self.next_discards_result = matches!(stmt, syn::Stmt::Expr(_, Some(_)));
        visit_mut::visit_stmt_mut(self, stmt);
    }
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        let discards_result = std::mem::take(&mut self.next_discards_result);
        let site = if plan::is_call(expr) {
            let site = self.cursor;
            self.cursor += 1;
            Some(site)
        } else {
            None
        };
        let original = source::fingerprint(tokens(expr));
        // Descend into original children first, but assign IDs in the same preorder as the planner.
        if matches!(expr, Expr::Paren(_)) {
            self.next_discards_result = discards_result;
        }
        visit_mut::visit_expr_mut(self, expr);
        if let Some(patches) = site.and_then(|site| self.calls.get(&site)) {
            if patches
                .iter()
                .any(|p| p.site_fingerprint.as_ref() != Some(&original))
            {
                self.error = Some(Diagnostic::new(
                    "E_STALE_SOURCE",
                    "call site fingerprint mismatch",
                ));
                return;
            }
            let mut calls = Vec::new();
            for patch in patches {
                match helper_call(patch) {
                    Ok(call) => calls.push(call),
                    Err(e) => {
                        self.error = Some(e);
                        return;
                    }
                }
            }
            let rewritten = if discards_result {
                quote!({ #(#calls;)* #expr; })
            } else {
                quote!({ #(#calls;)* #expr })
            };
            match syn::parse2(rewritten) {
                Ok(new) => {
                    *expr = new;
                    self.matched += 1;
                }
                Err(e) => self.error = Some(syntax(e)),
            }
        }
    }
}
fn locate<'a>(items: &'a mut [Item], path: &[usize]) -> Result<(&'a Signature, &'a mut Block)> {
    let Some((first, rest)) = path.split_first() else {
        return Err(Diagnostic::new("E_PLAN_SITE", "empty AST path"));
    };
    let item = items
        .get_mut(*first)
        .ok_or_else(|| Diagnostic::new("E_PLAN_SITE", "invalid AST item index"))?;
    match item {
        Item::Fn(f) if rest.is_empty() => Ok((&f.sig, &mut f.block)),
        Item::Mod(m) => {
            if let Some((_, items)) = &mut m.content {
                locate(items, rest)
            } else {
                Err(Diagnostic::new("E_PLAN_SITE", "expected inline module"))
            }
        }
        Item::Impl(i) if rest.len() == 1 => match i.items.get_mut(rest[0]) {
            Some(syn::ImplItem::Fn(f)) => Ok((&f.sig, &mut f.block)),
            _ => Err(Diagnostic::new("E_PLAN_SITE", "expected impl method")),
        },
        _ => Err(Diagnostic::new(
            "E_PLAN_SITE",
            "AST path does not address a supported function",
        )),
    }
}
