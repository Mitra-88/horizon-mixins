use crate::{Diagnostic, Result, diagnostic::io};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

pub fn fingerprint(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(Diagnostic::new(
            "E_INVALID_PATH",
            "expected a nonempty relative path without '.' or '..'",
        )
        .file(path));
    }
    Ok(())
}
pub fn contained(root: &Path, path: &Path) -> Result<PathBuf> {
    relative(path)?;
    let resolved = root
        .join(path)
        .canonicalize()
        .map_err(|e| io(e).file(root.join(path)))?;
    if !resolved.starts_with(root) {
        return Err(Diagnostic::new("E_INVALID_PATH", "path escapes its source root").file(path));
    }
    Ok(resolved)
}
/// Snapshot all staged inputs. Exclude build outputs, VCS and local player/assets data.
pub fn files(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(io)? {
            let entry = entry.map_err(io)?;
            let path = entry.path();
            let name = entry.file_name();
            if matches!(
                name.to_str(),
                Some("target" | ".git" | ".local" | ".horizon-mixins" | "node_modules")
            ) {
                continue;
            }
            let ty = entry.file_type().map_err(io)?;
            if ty.is_symlink() {
                return Err(Diagnostic::new(
                    "E_INVALID_PATH",
                    "symlinks are not supported in staged inputs",
                )
                .file(&path));
            }
            if ty.is_dir() {
                walk(root, &path, out)?;
            } else if ty.is_file() {
                let key = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .ok_or_else(|| Diagnostic::new("E_INVALID_PATH", "paths must be UTF-8"))?
                    .replace('\\', "/");
                out.insert(key, fs::read(&path).map_err(io)?);
            } else {
                return Err(
                    Diagnostic::new("E_INVALID_PATH", "special files cannot be staged").file(&path),
                );
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out)?;
    Ok(out)
}
