//! Bundle crates (0.6.0): the dependency edges their dependencies' features turn on.
//!
//! `native::stage` writes each bundle crate into the lock, but it cannot know which optional
//! dependencies a registry crate's features (its default ones included) use. Cargo can: it runs
//! `cargo update --workspace` on a scratch copy of the stage, which re-resolves only the workspace
//! members and keeps every locked version. A package that resolution adds or changes is refused
//! ([`native::refuse_new_packages`]), so the workspace's own lock stays the allow list.
use horizon_mixins_core::{Diagnostic, Plan, Result, diagnostic::io, native};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

/// Replace the stage's `Cargo.lock` in `files` with Cargo's resolution of it. Offline first, from
/// Cargo's cache; else from the registry index, which downloads no crate.
pub fn complete(
    root: &Path,
    out: &Path,
    plan: &Plan,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    if plan.crates.is_empty() {
        return Ok(());
    }
    let scratch = tempfile::Builder::new()
        .prefix(".horizon-mixins-lock-")
        .tempdir_in(out.parent().unwrap())
        .map_err(io)?;
    for (name, bytes) in files.iter() {
        let path = scratch.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(io)?;
        fs::write(path, bytes).map_err(io)?;
    }
    let update = |offline: bool| {
        // Like `cargo metadata` in `Index::load`: Cargo's own `CARGO` when set.
        let mut cmd = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
        cmd.args(["update", "--workspace", "--quiet", "--manifest-path"])
            .arg(scratch.path().join("Cargo.toml"));
        if offline {
            cmd.arg("--offline");
        }
        cmd.output().map_err(io)
    };
    let mut run = update(true)?;
    if !run.status.success() {
        run = update(false)?;
    }
    if !run.status.success() {
        return Err(Diagnostic::new(
            "E_CRATE_DEPENDENCY",
            format!(
                "cargo update could not resolve the bundle crates ({}):\n{}",
                run.status,
                String::from_utf8_lossy(&run.stderr)
            ),
        ));
    }
    let original = fs::read_to_string(root.join("Cargo.lock")).map_err(io)?;
    let updated = fs::read_to_string(scratch.path().join("Cargo.lock")).map_err(io)?;
    native::refuse_new_packages(plan, &original, &updated)?;
    files.insert("Cargo.lock".into(), updated.into_bytes());
    Ok(())
}
