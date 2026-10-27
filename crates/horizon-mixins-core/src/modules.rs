use crate::{Diagnostic, Result};
use syn::{
    Item, UseTree,
    visit::{self, Visit},
};

pub fn validate_payload(source: &str) -> Result<syn::File> {
    let file = syn::parse_file(source).map_err(|e| Diagnostic::new("E_MODULE", e.to_string()))?;
    if !file.attrs.is_empty() || file.shebang.is_some() {
        return Err(Diagnostic::new(
            "E_MODULE",
            "support module source cannot have file attributes or a shebang",
        ));
    }
    struct External(bool);
    impl<'ast> Visit<'ast> for External {
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            self.0 |=
                item.content.is_none() || item.attrs.iter().any(|a| a.path().is_ident("path"));
            visit::visit_item_mod(self, item);
        }
        fn visit_macro(&mut self, item: &'ast syn::Macro) {
            self.0 |= ["include", "include_str", "include_bytes"]
                .iter()
                .any(|n| item.path.is_ident(n));
            visit::visit_macro(self, item);
        }
    }
    let mut external = External(false);
    external.visit_file(&file);
    if external.0 {
        return Err(Diagnostic::new(
            "E_MODULE",
            "support modules must be self-contained; external mod/path/include sources are unsupported",
        ));
    }
    Ok(file)
}
fn imported(tree: &UseTree, name: &str) -> bool {
    match tree {
        UseTree::Path(p) => imported(&p.tree, name),
        UseTree::Name(n) => n.ident == name,
        UseTree::Rename(n) => n.rename == name,
        UseTree::Group(g) => g.items.iter().any(|t| imported(t, name)),
        UseTree::Glob(_) => true,
    }
}
pub fn validate_destination(file: &syn::File, name: &str) -> Result<()> {
    let collision = file.items.iter().any(|item| match item {
        Item::Mod(i) => i.ident == name,
        Item::Struct(i) => i.ident == name,
        Item::Enum(i) => i.ident == name,
        Item::Union(i) => i.ident == name,
        Item::Type(i) => i.ident == name,
        Item::Trait(i) => i.ident == name,
        Item::ExternCrate(i) => i.rename.as_ref().map_or(&i.ident, |(_, i)| i) == name,
        Item::Use(i) => imported(&i.tree, name),
        _ => false,
    });
    if collision {
        return Err(Diagnostic::new(
            "E_MODULE_CONFLICT",
            format!(
                "support module '{name}' collides with an existing type/module/import or unresolved glob import"
            ),
        ));
    }
    Ok(())
}
