//! Explicit per-instance state. No global registry or implicit serialization hooks.
use crate::{
    Diagnostic, Result, bridges, helpers,
    index::{Function, Index, tokens},
    manifest::{Kind, State},
    plan,
    sites::Sites,
    source,
};
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use syn::{
    Expr, Item, Type,
    visit::{self, Visit},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructorSite {
    pub file: String,
    pub node: Vec<usize>,
    pub function_fingerprint: String,
    pub expression: usize,
    pub fingerprint: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedState {
    pub mod_id: String,
    pub mod_version: String,
    pub state_id: String,
    pub priority: i32,
    pub target: String,
    pub file: String,
    pub node: Vec<usize>,
    pub fingerprint: String,
    pub name: String,
    pub ty: String,
    pub initial: String,
    pub deserialize: bool,
    pub constructors: Vec<ConstructorSite>,
}
fn fail(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("E_STATE", message)
}
fn literal_owner(f: &Function, path: &syn::Path) -> Option<String> {
    let ty = if path.is_ident("Self") {
        f.self_type.clone()?
    } else {
        Type::Path(syn::TypePath {
            qself: None,
            path: path.clone(),
        })
    };
    bridges::type_id(&f.module, &ty).ok()
}
pub fn plan(
    index: &Index,
    mod_id: &str,
    version: &str,
    priority: i32,
    state: &State,
) -> Result<PlannedState> {
    let owner = bridges::owner(index, &state.target)?;
    let syn::Fields::Named(fields) = &owner.item.fields else {
        return Err(fail("state requires a named-field struct"));
    };
    if owner.item.attrs.iter().any(|a| a.path().is_ident("repr")) {
        return Err(fail(
            "state changes layout; repr/FFI structs are unsupported",
        ));
    }
    if matches!(owner.item.vis, syn::Visibility::Public(_))
        && fields
            .named
            .iter()
            .all(|f| matches!(f.vis, syn::Visibility::Public(_)))
        && !owner
            .item
            .attrs
            .iter()
            .any(|a| a.path().is_ident("non_exhaustive"))
    {
        return Err(fail(
            "state requires a private field or non_exhaustive struct; external construction would break",
        ));
    }
    if fields.named.iter().any(|f| crate::index::cfg(&f.attrs)) {
        return Err(fail("state owner fields cannot have unresolved cfg"));
    }
    let ty: Type = syn::parse_str(&state.ty).map_err(|e| fail(e.to_string()))?;
    if helpers::unsupported_type(&ty, true) {
        return Err(fail(
            "state type must be explicit and owned, without references, Self or inferred types",
        ));
    }
    let initial: Expr = syn::parse_str(&state.initial).map_err(|e| fail(e.to_string()))?;
    // The initializer becomes a standalone function in the owner's module.
    let mut deserialize = false;
    for attr in owner
        .item
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
    {
        let paths = attr
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .map_err(|e| fail(e.to_string()))?;
        for path in paths {
            let name = path.segments.last().unwrap().ident.to_string();
            match name.as_str() {
                "Default"
                    if tokens(&initial)
                        != tokens(
                            &syn::parse_str::<Expr>("::core::default::Default::default()").unwrap(),
                        ) =>
                {
                    return Err(fail(
                        "derived Default requires initial = \"::core::default::Default::default()\"; custom defaults must declare constructor literals",
                    ));
                }
                "Deserialize" => deserialize = true,
                "Default" | "Clone" | "Copy" | "Debug" | "PartialEq" | "Eq" | "PartialOrd"
                | "Ord" | "Hash" | "Serialize" => {}
                _ => return Err(fail(format!("unproven derive {name} on state owner"))),
            }
        }
    }
    let normalize = |s: &str| s.replace('-', "_").to_ascii_lowercase();
    let name = format!(
        "__horizon_mixins_{}_{}",
        normalize(mod_id),
        normalize(&state.id)
    );
    if fields
        .named
        .iter()
        .any(|f| f.ident.as_ref().is_some_and(|i| i == &name))
        || index.functions.iter().any(|f| {
            [name.clone(), format!("{name}_mut"), format!("{name}_init")]
                .iter()
                .any(|n| f.signature.ident == n)
        })
    {
        return Err(Diagnostic::new(
            "E_STATE_CONFLICT",
            format!("reserved state name {name} already exists"),
        ));
    }
    let parent = if owner.node.len() == 1 {
        &index.files[&owner.file].items
    } else {
        let Item::Mod(module) = bridges::item(
            &index.files[&owner.file].items,
            &owner.node[..owner.node.len() - 1],
        )?
        else {
            return Err(fail("state owner parent is not a module"));
        };
        &module
            .content
            .as_ref()
            .ok_or_else(|| fail("state module is external"))?
            .1
    };
    let init_name = format!("{name}_init");
    if parent.iter().any(|i| match i {
        Item::Const(i) => i.ident == init_name,
        Item::Static(i) => i.ident == init_name,
        Item::Fn(i) => i.sig.ident == init_name,
        _ => false,
    }) {
        return Err(Diagnostic::new(
            "E_STATE_CONFLICT",
            "generated initializer name already exists",
        ));
    }
    if state.constructor.is_empty() {
        return Err(fail(
            "state requires explicit constructors with positive require counts",
        ));
    }
    let mut constructors = Vec::new();
    let mut seen = BTreeSet::new();
    for c in &state.constructor {
        if c.require == 0 || !seen.insert((&c.target, &c.selector)) {
            return Err(fail(
                "constructor requires a positive count and a unique target/selector",
            ));
        }
        let f = index.resolve(&c.target)?;
        plan::validate_target(f, Kind::ModifyExpressionValue)?;
        let selector: syn::Path = syn::parse_str(&c.selector).map_err(|e| fail(e.to_string()))?;
        if literal_owner(f, &selector).as_ref() != Some(&state.target) {
            return Err(fail(
                "constructor selector must resolve syntactically to the state owner (aliases unsupported)",
            ));
        }
        let sites = Sites::in_block(&f.block);
        let matches: Vec<_> = sites
            .values
            .iter()
            .filter(|s| matches!(&s.expr, Expr::Struct(e) if tokens(&e.path) == tokens(&selector)))
            .collect();
        if matches.len() != c.require {
            return Err(Diagnostic::new(
                "E_MATCH_COUNT",
                format!(
                    "state constructor {} expected {} '{}' literals, found {}",
                    c.target,
                    c.require,
                    c.selector,
                    matches.len()
                ),
            ));
        }
        for s in matches {
            constructors.push(ConstructorSite {
                file: f.file.clone(),
                node: f.node.clone(),
                function_fingerprint: source::fingerprint(format!(
                    "{}{}",
                    tokens(&f.signature),
                    tokens(&f.block)
                )),
                expression: s.expression,
                fingerprint: s.fingerprint.clone(),
            });
        }
    }
    let selected: BTreeSet<_> = constructors
        .iter()
        .map(|c| (&c.file, &c.node, c.expression))
        .collect();
    if selected.len() != constructors.len() {
        return Err(fail("duplicate state constructor site"));
    }
    // Prove coverage for visible, syntactically resolved literals; aliases and macro expansion
    // are deliberately not guessed. Cargo checks the final workspace as the last boundary.
    for f in &index.functions {
        for site in Sites::in_block(&f.block).values {
            if let Expr::Struct(e) = &site.expr
                && literal_owner(f, &e.path).as_ref() == Some(&state.target)
                && !selected.contains(&(&f.file, &f.node, site.expression))
            {
                return Err(fail(format!(
                    "unlisted constructor literal in {}; declare its selector/count",
                    f.id
                )));
            }
        }
        struct Patterns<'a> {
            f: &'a Function,
            target: &'a str,
            bad: bool,
        }
        impl<'ast> Visit<'ast> for Patterns<'_> {
            fn visit_pat_struct(&mut self, p: &'ast syn::PatStruct) {
                if p.rest.is_none()
                    && literal_owner(self.f, &p.path).as_deref() == Some(self.target)
                {
                    self.bad = true;
                }
                visit::visit_pat_struct(self, p);
            }
        }
        let mut p = Patterns {
            f,
            target: &state.target,
            bad: false,
        };
        p.visit_signature(&f.signature);
        p.visit_block(&f.block);
        if p.bad {
            return Err(fail(format!(
                "{} destructures the state owner without '..'; add an explicit rest pattern",
                f.id
            )));
        }
    }
    constructors
        .sort_by(|a, b| (&a.file, &a.node, a.expression).cmp(&(&b.file, &b.node, b.expression)));
    Ok(PlannedState {
        mod_id: mod_id.into(),
        mod_version: version.into(),
        state_id: state.id.clone(),
        priority: state.priority.unwrap_or(priority),
        target: state.target.clone(),
        file: owner.file.clone(),
        node: owner.node.clone(),
        fingerprint: source::fingerprint(tokens(&Item::Struct(owner.item.clone()))),
        name,
        ty: tokens(&ty),
        initial: tokens(&initial),
        deserialize,
        constructors,
    })
}
impl PlannedState {
    pub fn initializer(&self) -> Result<syn::Path> {
        let module = self
            .target
            .rsplit_once("::")
            .ok_or_else(|| fail("invalid state target"))?
            .0;
        let path = format!(
            "crate{}::{}_init",
            module.find("::").map(|i| &module[i..]).unwrap_or(""),
            self.name
        );
        syn::parse_str(&path).map_err(|e| fail(e.to_string()))
    }
    pub fn field(&self) -> Result<syn::Field> {
        let name = syn::parse_str::<syn::Ident>(&self.name).map_err(|e| fail(e.to_string()))?;
        let ty = syn::parse_str::<Type>(&self.ty).map_err(|e| fail(e.to_string()))?;
        let attr = if self.deserialize {
            let init = format!("{}_init", self.name);
            quote!(#[serde(default = #init)])
        } else {
            quote!()
        };
        Ok(syn::parse_quote!(#attr #name: #ty))
    }
    pub fn items(&self) -> Result<Vec<Item>> {
        let name = syn::parse_str::<syn::Ident>(&self.name).map_err(|e| fail(e.to_string()))?;
        let mutable = format_ident!("{}_mut", name);
        let init = format_ident!("{}_init", name);
        let owner = syn::parse_str::<syn::Ident>(self.target.rsplit("::").next().unwrap())
            .map_err(|e| fail(e.to_string()))?;
        let ty = syn::parse_str::<Type>(&self.ty).map_err(|e| fail(e.to_string()))?;
        let initial = syn::parse_str::<Expr>(&self.initial).map_err(|e| fail(e.to_string()))?;
        let file:syn::File=syn::parse2(quote!(pub(crate) fn #init()->#ty { #initial } impl #owner { pub fn #name(&self)-> &#ty { &self.#name } pub fn #mutable(&mut self)-> &mut #ty { &mut self.#name } })).map_err(|e| fail(e.to_string()))?;
        Ok(file.items)
    }
}
