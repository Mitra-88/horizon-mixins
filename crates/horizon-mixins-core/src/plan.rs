use crate::{
    Diagnostic, Result,
    diagnostic::io,
    index::{self, Function, Index, tokens},
    manifest::{self, Binding, Kind},
    source,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, process::Command};
use syn::{
    Expr, FnArg, Item, ReturnType,
    visit::{self, Visit},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: u32,
    pub transformer: String,
    pub rustc: String,
    pub inputs: BTreeMap<String, String>,
    pub injections: Vec<PlannedInjection>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedInjection {
    pub mod_id: String,
    pub mod_version: String,
    pub injection_id: String,
    pub priority: i32,
    pub target: String,
    pub file: String,
    pub node: Vec<usize>,
    pub function_fingerprint: String,
    pub kind: Kind,
    /// Preorder call number in the pristine body, never in an already rewritten tree.
    pub call_site: Option<usize>,
    pub site_fingerprint: Option<String>,
    pub selector: Option<String>,
    pub ordinal: Option<usize>,
    pub require: Option<usize>,
    pub helper_source: String,
    pub bindings: Vec<Binding>,
}
impl Plan {
    /// Validate externally deserialized IR before traversing any AST path.
    pub fn validate_structure(&self) -> Result<()> {
        let mut ids = std::collections::BTreeSet::new();
        for p in &self.injections {
            source::relative(Path::new(&p.file))?;
            let call_fields_valid = match p.kind {
                Kind::BeforeCall => {
                    p.call_site.is_some()
                        && p.site_fingerprint.is_some()
                        && p.selector.as_ref().is_some_and(|s| !s.is_empty())
                        && matches!((p.ordinal, p.require), (Some(o), Some(n)) if n > 0 && o < n)
                }
                _ => {
                    p.call_site.is_none()
                        && p.site_fingerprint.is_none()
                        && p.selector.is_none()
                        && p.ordinal.is_none()
                        && p.require.is_none()
                }
            };
            if !call_fields_valid
                || p.node.is_empty()
                || !self.inputs.contains_key(&p.file)
                || !ids.insert((&p.mod_id, &p.injection_id))
            {
                return Err(Diagnostic::new(
                    "E_PLAN_SITE",
                    "invalid call fields, AST path, input file or duplicate injection in plan",
                )
                .file(&p.file));
            }
        }
        Ok(())
    }
}

pub fn rustc(root: &Path) -> Result<String> {
    let out = Command::new("rustc")
        .current_dir(root)
        .arg("--version")
        .output()
        .map_err(io)?;
    if !out.status.success() {
        return Err(Diagnostic::new(
            "E_TOOLCHAIN",
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().into())
}
pub fn build_plan(workspace: &Path, mods: &Path) -> Result<Plan> {
    let root = workspace.canonicalize().map_err(io)?;
    let mods = manifest::load(mods)?;
    let inputs = source::files(&root)?
        .into_iter()
        .map(|(name, bytes)| (name, source::fingerprint(bytes)))
        .collect();
    let index = Index::load(&root)?;
    let mut injections = Vec::new();
    for bundle in mods {
        if let Some(revision) = &bundle.manifest.compatibility.horizon_revision {
            let actual = if root.join(".horizon-revision").is_file() {
                std::fs::read_to_string(root.join(".horizon-revision")).map_err(io)?
            } else {
                let output = Command::new("git")
                    .arg("-C")
                    .arg(&root)
                    .args(["rev-parse", "HEAD"])
                    .output()
                    .map_err(io)?;
                if !output.status.success() {
                    return Err(Diagnostic::new(
                        "E_REVISION",
                        "cannot establish a pinned Horizon revision",
                    )
                    .file(&bundle.file));
                }
                String::from_utf8_lossy(&output.stdout).into_owned()
            };
            if revision.len() != 40
                || !revision.bytes().all(|b| b.is_ascii_hexdigit())
                || actual.trim() != revision
            {
                return Err(Diagnostic::new(
                    "E_REVISION",
                    format!(
                        "requires exact Horizon revision {revision}, found {}",
                        actual.trim()
                    ),
                )
                .file(&bundle.file));
            }
        }
        for (injection, helper) in bundle.manifest.injection.iter().zip(&bundle.helpers) {
            let context = |e: Diagnostic| {
                let e = e.injection(&bundle.manifest.info.id, injection);
                if e.file.is_none() {
                    e.file(&bundle.file)
                } else {
                    e
                }
            };
            let f = index.resolve(&injection.target).map_err(context)?;
            validate_target(f).map_err(context)?;
            validate_helper(f, injection.kind, helper, &injection.bindings).map_err(context)?;
            if injection.kind == Kind::BeforeCall {
                reject_shadowed_parameters(f).map_err(context)?;
            }
            let function_fingerprint =
                source::fingerprint(format!("{}{}", tokens(&f.signature), tokens(&f.block)));
            if injection
                .fingerprint
                .as_ref()
                .is_some_and(|expected| expected != &function_fingerprint)
            {
                return Err(context(Diagnostic::new(
                    "E_FINGERPRINT",
                    format!(
                        "original function changed; expected {}, got {function_fingerprint}",
                        injection.fingerprint.as_ref().unwrap()
                    ),
                )));
            }
            let (call_site, site_fingerprint) = if injection.kind == Kind::BeforeCall {
                let selector = injection.selector.as_ref().unwrap();
                validate_selector(selector).map_err(context)?;
                let mut calls = Calls {
                    all: 0,
                    selector,
                    found: Vec::new(),
                    next_is_root: false,
                };
                calls.visit_block(&f.block);
                let count = injection.require.unwrap();
                if calls.found.len() != count {
                    return Err(context(Diagnostic::new("E_MATCH_COUNT", format!("selector '{selector}' requires {count} syntactic matches, found {} in pristine {}", calls.found.len(), f.id)).file(&f.file)));
                }
                let ordinal = injection.ordinal.unwrap();
                let Some((site, fingerprint, eligible)) = calls.found.get(ordinal) else {
                    return Err(context(
                        Diagnostic::new(
                            "E_ORDINAL",
                            format!(
                                "ordinal {ordinal} is outside 0..{} for '{selector}'",
                                calls.found.len()
                            ),
                        )
                        .file(&f.file),
                    ));
                };
                if !eligible {
                    return Err(context(Diagnostic::new("E_UNSUPPORTED_SITE", "BEFORE_CALL requires the complete expression of a statement, a plain let initializer, or a function/block tail; nesting can change temporary drop timing").file(&f.file)));
                }
                (Some(*site), Some(fingerprint.clone()))
            } else {
                (None, None)
            };
            injections.push(PlannedInjection {
                mod_id: bundle.manifest.info.id.clone(),
                mod_version: bundle.manifest.info.version.clone(),
                injection_id: injection.id.clone(),
                priority: injection.priority.unwrap_or(bundle.manifest.info.priority),
                target: f.id.clone(),
                file: f.file.clone(),
                node: f.node.clone(),
                function_fingerprint,
                kind: injection.kind,
                call_site,
                site_fingerprint,
                selector: injection.selector.clone(),
                ordinal: injection.ordinal,
                require: injection.require,
                helper_source: helper.clone(),
                bindings: injection.bindings.clone(),
            });
        }
    }
    injections.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.node.cmp(&b.node))
            .then(a.call_site.cmp(&b.call_site))
            .then(b.priority.cmp(&a.priority))
            .then(a.mod_id.cmp(&b.mod_id))
            .then(a.injection_id.cmp(&b.injection_id))
    });
    let mut groups: BTreeMap<(&str, &[usize]), Vec<&PlannedInjection>> = BTreeMap::new();
    for p in &injections {
        groups.entry((&p.file, &p.node)).or_default().push(p);
    }
    for entries in groups.values() {
        if entries.len() > 1 && entries.iter().any(|p| p.kind == Kind::Overwrite) {
            let p = entries[0];
            return Err(Diagnostic {
                mod_id: Some(p.mod_id.clone().into()),
                injection_id: Some(p.injection_id.clone().into()),
                target: Some(p.target.clone().into()),
                ..Diagnostic::new(
                    "E_OVERWRITE_CONFLICT",
                    format!(
                        "OVERWRITE must be exclusive on {}; competing injections: {}",
                        p.target,
                        entries
                            .iter()
                            .map(|p| format!("{}/{}", p.mod_id, p.injection_id))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
                .file(&p.file)
            });
        }
    }
    // Ensure metadata did not change inputs and catch concurrent source edits.
    let after: BTreeMap<_, _> = source::files(&root)?
        .into_iter()
        .map(|(name, bytes)| (name, source::fingerprint(bytes)))
        .collect();
    if inputs != after {
        return Err(Diagnostic::new(
            "E_STALE_SOURCE",
            "workspace changed during planning; retry from pristine source",
        ));
    }
    Ok(Plan {
        schema: 1,
        transformer: env!("CARGO_PKG_VERSION").into(),
        rustc: rustc(&root)?,
        inputs,
        injections,
    })
}
fn validate_target(f: &Function) -> Result<()> {
    let s = &f.signature;
    if s.asyncness.is_some()
        || s.constness.is_some()
        || s.unsafety.is_some()
        || s.abi.is_some()
        || s.variadic.is_some()
        || !s.generics.params.is_empty()
        || s.generics.where_clause.is_some()
        || f.generic_impl
    {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_SIGNATURE",
            "MVP requires a synchronous, safe, non-const, non-generic Rust function",
        )
        .file(&f.file));
    }
    let mut safety = Safety { error: None };
    safety.visit_signature(s);
    safety.visit_block(&f.block);
    if let Some((code, why)) = safety.error {
        return Err(Diagnostic::new(code, why).file(&f.file));
    }
    Ok(())
}
fn reject_shadowed_parameters(f: &Function) -> Result<()> {
    struct Shadow {
        names: std::collections::BTreeSet<String>,
        found: Option<String>,
    }
    impl<'ast> Visit<'ast> for Shadow {
        fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
            if self.names.contains(&pat.ident.to_string()) {
                self.found = Some(pat.ident.to_string());
            }
            visit::visit_pat_ident(self, pat);
        }
    }
    let mut shadow = Shadow {
        names: index::parameters(&f.signature)?,
        found: None,
    };
    shadow.visit_block(&f.block);
    if let Some(name) = shadow.found {
        return Err(Diagnostic::new(
            "E_INVALID_BINDING",
            format!("BEFORE_CALL cannot bind parameters in a body that shadows '{name}'"),
        )
        .file(&f.file));
    }
    Ok(())
}

struct Safety {
    error: Option<(&'static str, &'static str)>,
}
impl<'ast> Visit<'ast> for Safety {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::Macro(_) => {
                self.error = Some((
                    "E_UNSUPPORTED_MACRO",
                    "opaque macros in target body are unsupported; expose ordinary Rust source",
                ))
            }
            Expr::Closure(_)
            | Expr::Async(_)
            | Expr::Await(_)
            | Expr::Try(_)
            | Expr::TryBlock(_)
            | Expr::Yield(_)
            | Expr::Unsafe(_)
            | Expr::Const(_) => {
                self.error = Some((
                    "E_UNSUPPORTED_CONTROL_FLOW",
                    "closures, async, try/?, unsafe and const blocks are unsupported injection contexts",
                ))
            }
            // Calls under a reference can depend on temporary lifetime extension.
            Expr::Reference(r) if contains_call(&r.expr) => {
                self.error = Some((
                    "E_UNSUPPORTED_CONTROL_FLOW",
                    "calls beneath a reference may change temporary lifetimes",
                ))
            }
            _ => {}
        }
        visit::visit_expr(self, expr);
    }
    fn visit_macro(&mut self, _: &'ast syn::Macro) {
        self.error = Some(("E_UNSUPPORTED_MACRO", "target contains an opaque macro"));
    }
    fn visit_item(&mut self, _: &'ast Item) {
        self.error = Some((
            "E_UNSUPPORTED_CONTROL_FLOW",
            "nested items in target functions are unsupported",
        ));
    }
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if index::macro_attributes(std::slice::from_ref(attr)) {
            self.error = Some((
                "E_UNSUPPORTED_MACRO",
                "target contains an unexpanded attribute macro",
            ));
        }
        if attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr") {
            self.error = Some((
                "E_UNSUPPORTED_CFG",
                "target body contains unresolved cfg/cfg_attr",
            ));
        }
    }
}
fn contains_call(expr: &Expr) -> bool {
    struct Find(bool);
    impl<'ast> Visit<'ast> for Find {
        fn visit_expr(&mut self, e: &'ast Expr) {
            if is_call(e) {
                self.0 = true;
            }
            visit::visit_expr(self, e);
        }
    }
    let mut f = Find(false);
    f.visit_expr(expr);
    f.0
}
pub fn is_call(expr: &Expr) -> bool {
    matches!(expr, Expr::Call(_) | Expr::MethodCall(_))
}
pub fn call_key(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Call(c) => Some(tokens(&c.func).replace(' ', "")),
        Expr::MethodCall(c) => Some(format!(
            "{}.{}{}",
            tokens(&c.receiver).replace(' ', ""),
            c.method,
            tokens(&c.turbofish).replace(' ', "")
        )),
        _ => None,
    }
}
fn validate_selector(selector: &str) -> Result<()> {
    let parsed = syn::parse_str::<Expr>(&format!("{selector}()"));
    if parsed.as_ref().ok().and_then(call_key).as_deref() != Some(selector) {
        return Err(Diagnostic::new(
            "E_SELECTOR",
            "selector must be an exact callable path or receiver.method without spaces or parentheses",
        ));
    }
    Ok(())
}
struct Calls<'a> {
    all: usize,
    selector: &'a str,
    found: Vec<(usize, String, bool)>,
    next_is_root: bool,
}
impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        match stmt {
            syn::Stmt::Expr(expr, _) => {
                self.next_is_root = true;
                self.visit_expr(expr);
            }
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    self.next_is_root = matches!(&local.pat, syn::Pat::Ident(p) if p.by_ref.is_none() && p.subpat.is_none());
                    self.visit_expr(&init.expr);
                    if let Some((_, expr)) = &init.diverge {
                        self.visit_expr(expr);
                    }
                }
            }
            _ => visit::visit_stmt(self, stmt),
        }
    }
    fn visit_expr(&mut self, expr: &'ast Expr) {
        let eligible = std::mem::take(&mut self.next_is_root);
        if let Some(key) = call_key(expr) {
            let site = self.all;
            self.all += 1;
            if key == self.selector {
                self.found
                    .push((site, source::fingerprint(tokens(expr)), eligible));
            }
        }
        if matches!(expr, Expr::Paren(_)) {
            self.next_is_root = eligible;
        }
        visit::visit_expr(self, expr);
    }
}
pub fn parse_helper(source: &str) -> Result<syn::ItemFn> {
    let file =
        syn::parse_file(source).map_err(|e| Diagnostic::new("E_INVALID_HELPER", e.to_string()))?;
    if !file.attrs.is_empty() || file.items.len() != 1 {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper file must contain exactly one private function",
        ));
    }
    let Some(Item::Fn(helper)) = file.items.into_iter().next() else {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper must be a function",
        ));
    };
    let sig = &helper.sig;
    if !helper.attrs.is_empty()
        || !matches!(helper.vis, syn::Visibility::Inherited)
        || sig.asyncness.is_some()
        || sig.constness.is_some()
        || sig.unsafety.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
        || !sig.generics.params.is_empty()
        || sig.generics.where_clause.is_some()
    {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper must be private, synchronous, safe, non-generic and have no attributes",
        ));
    }
    Ok(helper)
}
pub fn validate_helper(f: &Function, kind: Kind, source: &str, bindings: &[Binding]) -> Result<()> {
    let helper = parse_helper(source)?;
    let parameters = index::parameters(&f.signature)?;
    if parameters.contains(&helper.sig.ident.to_string()) {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper name shadows a target parameter; rename it",
        ));
    }
    if helper.sig.inputs.len() != bindings.len() {
        return Err(Diagnostic::new(
            "E_INVALID_BINDING",
            "one explicit binding is required per helper parameter",
        ));
    }
    if kind != Kind::Overwrite
        && !matches!(&helper.sig.output, ReturnType::Default)
        && !matches!(&helper.sig.output, ReturnType::Type(_, ty) if matches!(&**ty, syn::Type::Tuple(t) if t.elems.is_empty()))
    {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "insertion helpers must return ()",
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    for (arg, binding) in helper.sig.inputs.iter().zip(bindings) {
        let FnArg::Typed(arg) = arg else {
            return Err(Diagnostic::new(
                "E_INVALID_HELPER",
                "helpers cannot have self receivers",
            ));
        };
        let syn::Pat::Ident(pattern) = &*arg.pat else {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                "helper parameters must be named identifiers",
            ));
        };
        let ty: syn::Type = syn::parse_str(&binding.ty)
            .map_err(|e| Diagnostic::new("E_INVALID_BINDING", e.to_string()))?;
        let value: Expr = syn::parse_str(&binding.value)
            .map_err(|e| Diagnostic::new("E_INVALID_BINDING", e.to_string()))?;
        if pattern.ident != binding.name
            || !names.insert(&binding.name)
            || tokens(&ty) != tokens(&arg.ty)
        {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                format!(
                    "binding '{}' must match helper parameter name and type in declaration order",
                    binding.name
                ),
            ));
        }
        if !binding_expr(&value, &parameters) {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                format!(
                    "binding '{}' must be a target parameter or its explicit borrow/reborrow; no calls or locals",
                    binding.value
                ),
            ));
        }
        if tokens(&ty).split_whitespace().any(|s| s == "Self") {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                "use an explicit concrete type instead of Self in local helper signatures",
            ));
        }
    }
    Ok(())
}
fn binding_expr(expr: &Expr, parameters: &std::collections::BTreeSet<String>) -> bool {
    match expr {
        Expr::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            parameters.contains(&p.path.segments[0].ident.to_string())
        }
        Expr::Reference(r) => binding_expr(&r.expr, parameters),
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Deref(_)) => binding_expr(&u.expr, parameters),
        Expr::Paren(p) => binding_expr(&p.expr, parameters),
        _ => false,
    }
}
