use crate::{Diagnostic, Result, diagnostic::io, source};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "mod")]
    pub info: Mod,
    #[serde(default)]
    pub compatibility: Compatibility,
    #[serde(default)]
    pub injection: Vec<Injection>,
    #[serde(default)]
    pub bridge: Vec<Bridge>,
    #[serde(default)]
    pub module: Vec<Module>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub id: String,
    pub file: String,
    pub name: String,
    pub source: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mod {
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub priority: i32,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub horizon_revision: Option<String>,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Kind {
    Head,
    BeforeCall,
    Overwrite,
    Return,
    ModifyArg,
    ModifyConstant,
    WrapOperation,
    ModifyArgs,
    ModifyVariable,
}
impl Kind {
    pub fn selects_call(self) -> bool {
        matches!(
            self,
            Self::BeforeCall | Self::ModifyArg | Self::ModifyArgs | Self::WrapOperation
        )
    }
    pub fn selects_site(self) -> bool {
        self.selects_call() || matches!(self, Self::ModifyConstant | Self::ModifyVariable)
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Injection {
    pub id: String,
    pub target: String,
    pub kind: Kind,
    pub helper: String,
    #[serde(default)]
    pub cancellable: bool,
    pub argument: Option<usize>,
    #[serde(default)]
    pub bindings: Vec<Binding>,
    pub priority: Option<i32>,
    pub selector: Option<String>,
    pub ordinal: Option<usize>,
    pub require: Option<usize>,
    pub fingerprint: Option<String>,
    pub slice: Option<Slice>,
    pub variable: Option<Variable>,
    pub result: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub selector: String,
    pub ordinal: usize,
    pub require: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slice {
    pub from: Option<Anchor>,
    pub to: Option<Anchor>,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Access {
    Store,
    Load,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variable {
    pub name: String,
    pub ordinal: usize,
    pub at: Access,
    #[serde(default)]
    pub parameter: bool,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgeKind {
    Get,
    GetMut,
    Set,
    Invoke,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bridge {
    pub id: String,
    pub kind: BridgeKind,
    pub target: String,
    pub member: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub name: String,
    pub ty: String,
    pub value: String,
}
#[derive(Debug)]
pub struct LoadedMod {
    pub manifest: Manifest,
    pub file: String,
    pub helpers: Vec<String>,
    pub modules: Vec<String>,
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
/// Only direct TOML files and one directory level of horizon-mixin.toml bundles.
pub fn load(root: &Path) -> Result<Vec<LoadedMod>> {
    let root = root.canonicalize().map_err(io)?;
    let mut paths = Vec::new();
    if !root.is_dir() {
        return Err(Diagnostic::new("E_INVALID_PATH", "--mods must be a directory").file(&root));
    }
    for entry in fs::read_dir(&root).map_err(io)? {
        let entry = entry.map_err(io)?;
        if entry.file_type().map_err(io)?.is_symlink() {
            return Err(
                Diagnostic::new("E_INVALID_PATH", "mod bundles cannot be symlinks")
                    .file(entry.path()),
            );
        }
        if entry.path().extension().is_some_and(|e| e == "toml") {
            paths.push(entry.path());
        } else if entry.file_type().map_err(io)?.is_dir()
            && entry.path().join("horizon-mixin.toml").exists()
        {
            paths.push(entry.path().join("horizon-mixin.toml"));
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(Diagnostic::new("E_MANIFEST", "no TOML mod manifests found").file(&root));
    }
    let mut ids = BTreeSet::new();
    let mut mods = Vec::new();
    for path in paths {
        let path = source::contained(&root, path.strip_prefix(&root).unwrap())?;
        let text = fs::read_to_string(&path).map_err(|e| io(e).file(&path))?;
        let manifest: Manifest = toml::from_str(&text)
            .map_err(|e| Diagnostic::new("E_MANIFEST", e.to_string()).file(&path))?;
        let fail = |code: &str, message: String| Diagnostic::new(code, message).file(&path);
        if !valid_id(&manifest.info.id)
            || manifest.info.version.is_empty()
            || (manifest.injection.is_empty()
                && manifest.bridge.is_empty()
                && manifest.module.is_empty())
        {
            return Err(fail(
                "E_MANIFEST",
                "mod needs an ASCII id, a version and at least one injection, bridge or module"
                    .into(),
            ));
        }
        if !ids.insert(manifest.info.id.clone()) {
            return Err(fail(
                "E_DUPLICATE_ID",
                format!("duplicate mod id '{}'", manifest.info.id),
            ));
        }
        let mut injections = BTreeSet::new();
        let mut helpers = Vec::new();
        let mut modules = Vec::new();
        for module in &manifest.module {
            if !valid_id(&module.id) || !injections.insert(&module.id) {
                return Err(fail(
                    "E_DUPLICATE_ID",
                    format!("invalid or duplicate module id '{}'", module.id),
                ));
            }
            source::relative(Path::new(&module.file))?;
            syn::parse_str::<syn::Ident>(&module.name)
                .map_err(|e| fail("E_MODULE", e.to_string()))?;
            let payload = source::contained(path.parent().unwrap(), Path::new(&module.source))?;
            modules.push(fs::read_to_string(&payload).map_err(io)?);
        }
        for bridge in &manifest.bridge {
            if !valid_id(&bridge.id) || !injections.insert(&bridge.id) {
                return Err(fail(
                    "E_DUPLICATE_ID",
                    format!("invalid or duplicate bridge id '{}'", bridge.id),
                ));
            }
            if (bridge.kind == BridgeKind::Invoke) == bridge.member.is_some() {
                return Err(fail("E_BRIDGE_TARGET", "GET/GET_MUT/SET require member; INVOKE uses an exact method target without member".into()));
            }
        }
        for injection in &manifest.injection {
            let context = |e: Diagnostic| e.injection(&manifest.info.id, injection).file(&path);
            if !valid_id(&injection.id) || !injections.insert(&injection.id) {
                return Err(context(fail(
                    "E_DUPLICATE_ID",
                    format!("invalid or duplicate injection id '{}'", injection.id),
                )));
            }
            if injection.cancellable && injection.kind != Kind::Head {
                return Err(context(fail(
                    "E_MANIFEST",
                    "cancellable is supported only for HEAD".into(),
                )));
            }
            if (injection.kind == Kind::ModifyArg) != injection.argument.is_some() {
                return Err(context(fail(
                    "E_ARGUMENT",
                    "argument is required only for MODIFY_ARG".into(),
                )));
            }
            if (injection.kind == Kind::ModifyVariable) != injection.variable.is_some() {
                return Err(context(fail(
                    "E_VARIABLE",
                    "variable is required only for MODIFY_VARIABLE".into(),
                )));
            }
            if (injection.kind == Kind::ModifyArgs) != injection.result.is_some() {
                return Err(context(fail(
                    "E_ARGUMENT_SIGNATURE",
                    "result is required only for MODIFY_ARGS".into(),
                )));
            }
            if injection.kind.selects_site() {
                if (injection.kind != Kind::ModifyVariable && injection.selector.is_none())
                    || injection.ordinal.is_none()
                    || injection.require.is_none_or(|n| n == 0)
                {
                    return Err(context(fail("E_SELECTOR", "site injections require a selector (or variable), zero-based ordinal and positive require".into())));
                }
                if injection.kind == Kind::ModifyVariable && injection.selector.is_some() {
                    return Err(context(fail(
                        "E_SELECTOR",
                        "MODIFY_VARIABLE uses variable rather than selector".into(),
                    )));
                }
            } else if injection.selector.is_some()
                || injection.ordinal.is_some()
                || injection.require.is_some()
            {
                return Err(context(fail(
                    "E_SELECTOR",
                    "selector/ordinal/require apply only to site injections".into(),
                )));
            }
            if injection.slice.is_some() && !injection.kind.selects_site() {
                return Err(context(fail(
                    "E_SLICE",
                    "slices apply only to site injections".into(),
                )));
            }
            let helper_path =
                source::contained(path.parent().unwrap(), Path::new(&injection.helper))
                    .map_err(context)?;
            let helper = fs::read_to_string(&helper_path).map_err(|e| context(io(e)))?;
            helpers.push(helper);
        }
        mods.push(LoadedMod {
            manifest,
            helpers,
            modules,
            file: path.display().to_string(),
        });
    }
    mods.sort_by(|a, b| a.manifest.info.id.cmp(&b.manifest.info.id));
    Ok(mods)
}
