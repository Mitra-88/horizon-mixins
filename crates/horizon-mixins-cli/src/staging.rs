use fs2::FileExt;
use horizon_mixins_core::{Diagnostic, Plan, Result, diagnostic::io, source};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
};
const MARKER: &str = ".horizon-mixins-stage.json";

/// Reject symlinks in every existing output component, including parent directories.
pub fn output_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io)?.join(path)
    };
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(
                    Diagnostic::new("E_INVALID_PATH", "output paths cannot contain '..'")
                        .file(path),
                );
            }
            Component::CurDir => continue,
            // A drive prefix (`C:`, `\\?\C:`) names a volume, not a path to a file: Windows can't
            // stat a verbatim one (ERROR_INVALID_FUNCTION), and a volume is never a link.
            Component::Prefix(_) => {
                output.push(component);
                continue;
            }
            _ => output.push(component),
        }
        match fs::symlink_metadata(&output) {
            Ok(meta) if meta.is_symlink() => {
                return Err(Diagnostic::new(
                    "E_INVALID_PATH",
                    "output paths cannot traverse symlinks",
                )
                .file(output));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(e).file(&output)),
        }
    }
    if output.file_name().is_none() {
        return Err(Diagnostic::new(
            "E_INVALID_PATH",
            "output must have a filename",
        ));
    }
    Ok(output)
}
pub fn lock(path: &Path, purpose: &str) -> Result<File> {
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).map_err(io)?;
    let name = format!(
        ".{}.horizon-mixins-{purpose}.lock",
        path.file_name().unwrap().to_string_lossy()
    );
    let lock_path = parent.join(name);
    if fs::symlink_metadata(&lock_path).is_ok_and(|m| m.is_symlink()) {
        return Err(Diagnostic::new("E_INVALID_PATH", "lock file is a symlink").file(&lock_path));
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(io)?;
    file.lock_exclusive().map_err(io)?;
    Ok(file)
}
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if fs::read(path).is_ok_and(|old| old == bytes) {
        return Ok(());
    }
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).map_err(io)?;
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(io)?;
    use std::io::Write;
    pending.write_all(bytes).map_err(io)?;
    pending.as_file().sync_all().map_err(io)?;
    pending.persist(path).map_err(|e| io(e.error))?;
    Ok(())
}
pub fn publish(
    root: &Path,
    out: &Path,
    mut files: BTreeMap<String, Vec<u8>>,
    plan: &Plan,
) -> Result<bool> {
    if files.contains_key(MARKER) {
        return Err(Diagnostic::new(
            "E_INVALID_PATH",
            "input uses reserved stage marker filename",
        ));
    }
    let metadata =
        serde_json::to_vec_pretty(plan).map_err(|e| Diagnostic::new("E_JSON", e.to_string()))?;
    files.insert(MARKER.into(), metadata);
    let old = if out.exists() {
        if !out.is_dir() || !out.join(MARKER).is_file() {
            return Err(Diagnostic::new(
                "E_STAGE_OWNERSHIP",
                "refusing to replace a directory without a Horizon Mixins stage marker",
            )
            .file(out));
        }
        let old = source::files(out)?;
        if old == files {
            return Ok(false);
        }
        old
    } else {
        BTreeMap::new()
    };
    let pending = tempfile::Builder::new()
        .prefix(".horizon-mixins-pending-")
        .tempdir_in(out.parent().unwrap())
        .map_err(io)?;
    for (name, bytes) in files {
        source::relative(Path::new(&name))?;
        let destination = pending.path().join(&name);
        fs::create_dir_all(destination.parent().unwrap()).map_err(io)?;
        if old.get(&name) == Some(&bytes) {
            // Link only our prior staged files. Never hard-link immutable originals.
            fs::hard_link(out.join(&name), &destination).map_err(io)?;
        } else {
            fs::write(&destination, bytes).map_err(io)?;
            if let Ok(meta) = fs::metadata(root.join(&name)) {
                fs::set_permissions(&destination, meta.permissions()).map_err(io)?;
            }
        }
    }
    if out.exists() {
        exchange(pending.path(), out)?;
        // pending now owns the old complete workspace and removes it on drop.
    } else {
        fs::rename(pending.path(), out).map_err(io)?;
    }
    Ok(true)
}
#[cfg(target_os = "linux")]
fn exchange(a: &Path, b: &Path) -> Result<()> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        a,
        rustix::fs::CWD,
        b,
        rustix::fs::RenameFlags::EXCHANGE,
    )
    .map_err(|e| {
        Diagnostic::new(
            "E_ATOMIC_STAGE",
            format!("atomic directory exchange failed; prior stage retained: {e}"),
        )
    })
}
#[cfg(not(target_os = "linux"))]
fn exchange(_: &Path, _: &Path) -> Result<()> {
    Err(Diagnostic::new(
        "E_ATOMIC_STAGE",
        "atomic replacement of an existing stage currently requires Linux; use a fresh output directory",
    ))
}
