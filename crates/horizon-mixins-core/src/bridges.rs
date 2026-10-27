use crate::{
    Diagnostic, Result,
    index::{Index, TypeDefinition, cfg, tokens},
    manifest::{Bridge, BridgeKind},
    source,
};
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{Item, Type};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedBridge {
    pub mod_id: String,
    pub bridge_id: String,
    pub mod_version: String,
    pub kind: BridgeKind,
    pub target: String,
    pub owner: String,
    pub name: String,
    pub file: String,
    pub node: Vec<usize>,
    pub fingerprint: String,
    pub generated: String,
}
pub fn item<'a>(items: &'a [Item], node: &[usize]) -> Result<&'a Item> {
    let Some((first, rest)) = node.split_first() else {
        return Err(Diagnostic::new("E_PLAN_SITE", "empty bridge path"));
    };
    let value = items
        .get(*first)
        .ok_or_else(|| Diagnostic::new("E_PLAN_SITE", "invalid bridge item index"))?;
    if rest.is_empty() {
        return Ok(value);
    }
    if let Item::Mod(m) = value
        && let Some((_, children)) = &m.content
    {
        return item(children, rest);
    }
    Err(Diagnostic::new(
        "E_PLAN_SITE",
        "bridge parent is not an inline module",
    ))
}
pub(crate) fn owner<'a>(index: &'a Index, id: &str) -> Result<&'a TypeDefinition> {
    if index
        .blocked
        .iter()
        .any(|m| id.starts_with(&format!("{m}::")))
    {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_CFG",
            "bridge type is under unresolved cfg",
        ));
    }
    let matches: Vec<_> = index.types.iter().filter(|t| t.id == id).collect();
    let [ty] = matches.as_slice() else {
        return Err(Diagnostic::new(
            if matches.is_empty() {
                "E_BRIDGE_TARGET"
            } else {
                "E_TARGET_AMBIGUOUS"
            },
            format!("bridge requires exactly one owned struct '{id}'"),
        ));
    };
    if index
        .types
        .iter()
        .filter(|t| t.file == ty.file && t.node == ty.node)
        .count()
        != 1
    {
        return Err(Diagnostic::new(
            "E_TARGET_AMBIGUOUS",
            "bridge struct source is reused by modules/targets",
        ));
    }
    if ty.unsupported_cfg {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_CFG",
            "bridge struct has unresolved cfg",
        ));
    }
    if ty.unsupported_macro {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_MACRO",
            "bridge struct has an unexpanded attribute macro",
        ));
    }
    if !ty.item.generics.params.is_empty() || ty.item.generics.where_clause.is_some() {
        return Err(Diagnostic::new(
            "E_BRIDGE_TARGET",
            "generic bridge owner types are unsupported",
        ));
    }
    Ok(ty)
}
pub(crate) fn type_id(module: &str, ty: &Type) -> Result<String> {
    let Type::Path(p) = ty else {
        return Err(Diagnostic::new(
            "E_BRIDGE_TARGET",
            "bridge owner must be a concrete struct path",
        ));
    };
    if p.qself.is_some()
        || p.path
            .segments
            .iter()
            .any(|s| !matches!(s.arguments, syn::PathArguments::None))
    {
        return Err(Diagnostic::new(
            "E_BRIDGE_TARGET",
            "generic/qualified owner types are unsupported",
        ));
    }
    let mut names: Vec<String> = p
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    let mut base: Vec<String> = module.split("::").map(Into::into).collect();
    if names.first().is_some_and(|s| s == "crate") {
        base.truncate(1);
        names.remove(0);
    } else if names.first().is_some_and(|s| s == "self") {
        names.remove(0);
    } else {
        while names.first().is_some_and(|s| s == "super") {
            if base.len() <= 1 {
                return Err(Diagnostic::new(
                    "E_BRIDGE_TARGET",
                    "owner path escapes crate",
                ));
            }
            base.pop();
            names.remove(0);
        }
    }
    base.extend(names);
    Ok(base.join("::"))
}
pub fn plan(
    index: &Index,
    mod_id: &str,
    mod_version: &str,
    bridge: &Bridge,
) -> Result<PlannedBridge> {
    let normalize = |s: &str| s.replace('-', "_").to_ascii_lowercase();
    let name = format!(
        "__horizon_mixins_{}_{}",
        normalize(mod_id),
        normalize(&bridge.id)
    );
    let ident = format_ident!("{name}");
    let (owner_id, file, node, generated) = if bridge.kind == BridgeKind::Invoke {
        let f = index.resolve(&bridge.target)?;
        let ty = f.self_type.as_ref().ok_or_else(|| {
            Diagnostic::new(
                "E_BRIDGE_TARGET",
                "INVOKE target must be an inherent/trait impl method or associated function",
            )
        })?;
        let owner_id = type_id(&f.module, ty)?;
        owner(index, &owner_id)?;
        let sig = &f.signature;
        struct Projection(bool);
        impl<'ast> syn::visit::Visit<'ast> for Projection {
            fn visit_type_path(&mut self, p: &'ast syn::TypePath) {
                self.0 |= p.qself.is_none()
                    && p.path.segments.len() > 1
                    && p.path.segments.first().is_some_and(|s| s.ident == "Self");
                syn::visit::visit_type_path(self, p);
            }
        }
        let mut projection = Projection(false);
        syn::visit::Visit::visit_signature(&mut projection, sig);
        if projection.0 {
            return Err(Diagnostic::new(
                "E_BRIDGE_TARGET",
                "unqualified Self associated-type projections need an explicit concrete/qualified signature",
            ));
        }

        if f.generic_impl
            || !sig.generics.params.is_empty()
            || sig.generics.where_clause.is_some()
            || sig.constness.is_some()
            || sig.asyncness.is_some()
            || sig.unsafety.is_some()
            || sig.abi.is_some()
            || sig.variadic.is_some()
        {
            return Err(Diagnostic::new(
                "E_BRIDGE_TARGET",
                "invoker requires a safe synchronous non-generic Rust signature",
            ));
        }
        let mut args = Vec::new();
        for arg in &sig.inputs {
            match arg {
                syn::FnArg::Receiver(_) => args.push(quote!(self)),
                syn::FnArg::Typed(arg) => {
                    if let syn::Pat::Ident(p) = &*arg.pat {
                        if p.by_ref.is_some() || p.subpat.is_some() {
                            return Err(Diagnostic::new(
                                "E_BRIDGE_TARGET",
                                "invoker parameters must be plain names",
                            ));
                        }
                        let id = &p.ident;
                        args.push(quote!(#id));
                    } else {
                        return Err(Diagnostic::new(
                            "E_BRIDGE_TARGET",
                            "invoker cannot forward destructured/unnamed parameters",
                        ));
                    }
                }
            }
        }
        let impl_node = f.node[..f.node.len() - 1].to_vec();
        let Item::Impl(original) = item(&index.files[&f.file].items, &impl_node)? else {
            return Err(Diagnostic::new("E_PLAN_SITE", "invoker impl disappeared"));
        };
        let original_name = &sig.ident;
        let call = if let Some((_, tr, _)) = &original.trait_ {
            quote!(<#ty as #tr>::#original_name(#(#args),*))
        } else {
            quote!(<#ty>::#original_name(#(#args),*))
        };
        let mut sig = sig.clone();
        for input in &mut sig.inputs {
            match input {
                syn::FnArg::Typed(arg) => {
                    if let syn::Pat::Ident(p) = &mut *arg.pat {
                        p.mutability = None;
                    }
                }
                syn::FnArg::Receiver(r) if r.reference.is_none() => r.mutability = None,
                _ => {}
            }
        }
        sig.ident = ident.clone();
        let generated: Item = syn::parse2(quote!(impl #ty { pub #sig { #call } }))
            .map_err(|e| Diagnostic::new("E_BRIDGE_TARGET", e.to_string()))?;
        (owner_id, f.file.clone(), impl_node, generated)
    } else {
        let ty = owner(index, &bridge.target)?;
        let member = bridge
            .member
            .as_ref()
            .ok_or_else(|| Diagnostic::new("E_BRIDGE_TARGET", "field bridge requires member"))?;
        let field = match &ty.item.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().is_some_and(|i| i == member)),
            syn::Fields::Unnamed(fields) => member
                .parse::<usize>()
                .ok()
                .and_then(|i| fields.unnamed.get(i)),
            syn::Fields::Unit => None,
        }
        .ok_or_else(|| {
            Diagnostic::new(
                "E_BRIDGE_TARGET",
                format!("field {member} does not exist on {}", bridge.target),
            )
        })?;
        if cfg(&field.attrs) {
            return Err(Diagnostic::new(
                "E_UNSUPPORTED_CFG",
                "bridge field has unresolved cfg",
            ));
        }
        let owner_name = &ty.item.ident;
        let field_ty = &field.ty;
        let member: syn::Member = syn::parse_str(member)
            .map_err(|e| Diagnostic::new("E_BRIDGE_TARGET", e.to_string()))?;
        let method = match bridge.kind {
            BridgeKind::Get => quote!(pub fn #ident(&self)-> &#field_ty { &self.#member }),
            BridgeKind::GetMut => {
                quote!(pub fn #ident(&mut self)-> &mut #field_ty { &mut self.#member })
            }
            BridgeKind::Set => {
                quote!(pub fn #ident(&mut self,value:#field_ty) { self.#member=value; })
            }
            BridgeKind::Invoke => unreachable!(),
        };
        let generated: Item = syn::parse2(quote!(impl #owner_name { #method }))
            .map_err(|e| Diagnostic::new("E_BRIDGE_TARGET", e.to_string()))?;
        (ty.id.clone(), ty.file.clone(), ty.node.clone(), generated)
    };
    if index.functions.iter().any(|f| {
        f.signature.ident == name
            && f.self_type
                .as_ref()
                .and_then(|t| type_id(&f.module, t).ok())
                .as_ref()
                == Some(&owner_id)
    }) {
        return Err(Diagnostic::new(
            "E_BRIDGE_CONFLICT",
            format!("generated method {name} already exists on {owner_id}"),
        ));
    }
    let fingerprint = source::fingerprint(tokens(item(&index.files[&file].items, &node)?));
    Ok(PlannedBridge {
        mod_id: mod_id.into(),
        bridge_id: bridge.id.clone(),
        mod_version: mod_version.into(),
        kind: bridge.kind,
        target: bridge.target.clone(),
        owner: owner_id,
        name,
        file,
        node,
        fingerprint,
        generated: tokens(&generated),
    })
}
