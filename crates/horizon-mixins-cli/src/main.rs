mod lockfile;
mod staging;
use clap::{Args, Parser, Subcommand};
use horizon_mixins_core::{Diagnostic, Result, build_plan, build_staged, diagnostic::io, native};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::Instant,
};

#[derive(Parser)]
#[command(
    name = "horizon-mixins",
    version,
    about = "Deterministic, staged source mixins for trusted native mods"
)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    /// Validate all mods on pristine source and atomically write a JSON plan.
    Plan(Common),
    /// Stage transformed source; original workspace is immutable.
    Apply(Common),
    /// Stage and run cargo test --locked --offline. Native code is unsandboxed.
    Check {
        #[command(flatten)]
        common: Common,
        #[arg(long)]
        trust_native_mods: bool,
        #[arg(long)]
        target_dir: Option<PathBuf>,
        #[arg(long = "package")]
        packages: Vec<String>,
    },
}
#[derive(Args)]
struct Common {
    #[arg(long)]
    workspace: PathBuf,
    #[arg(long)]
    mods: PathBuf,
    #[arg(long)]
    out: PathBuf,
}
fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", serde_json::to_string_pretty(&error).unwrap());
            ExitCode::FAILURE
        }
    }
}
fn run(cli: Cli) -> Result<serde_json::Value> {
    let start = Instant::now();
    let (common, check) = match cli.command {
        Action::Plan(common) => {
            let root = common.workspace.canonicalize().map_err(io)?;
            let out = staging::output_path(&common.out)?;
            protect(&root, &out)?;
            let mods = common.mods.canonicalize().map_err(io)?;
            protect(&mods, &out)?;
            let plan = build_plan(&root, &mods)?;
            let mut bytes = serde_json::to_vec_pretty(&plan)
                .map_err(|e| Diagnostic::new("E_JSON", e.to_string()))?;
            bytes.push(b'\n');
            let _lock = staging::lock(&out, "plan")?;
            staging::write_atomic(&out, &bytes)?;
            return Ok(
                json!({"status":"ok", "plan":out, "injections":plan.injections.len(), "bridges":plan.bridges.len(), "modules":plan.modules.len(), "states":plan.states.len(), "crates":plan.crates.len(), "data":plan.data.len(), "post_build":plan.post_build.len(), "seconds":start.elapsed().as_secs_f64()}),
            );
        }
        Action::Apply(common) => (common, None),
        Action::Check {
            common,
            trust_native_mods,
            target_dir,
            packages,
        } => {
            if !trust_native_mods {
                return Err(Diagnostic::new(
                    "E_TRUST_NATIVE_MODS",
                    "native mods, dependencies and build scripts execute unsandboxed; review them and pass --trust-native-mods to run Cargo",
                ));
            }
            (common, Some((target_dir, packages)))
        }
    };
    let root = common.workspace.canonicalize().map_err(io)?;
    let mods = common.mods.canonicalize().map_err(io)?;
    let out = staging::output_path(&common.out)?;
    protect(&root, &out)?;
    protect(&mods, &out)?;
    let target = check
        .as_ref()
        .map(|(target, _)| {
            let target = target.clone().unwrap_or_else(|| {
                out.with_file_name(format!(
                    ".{}.cargo-target",
                    out.file_name().unwrap().to_string_lossy()
                ))
            });
            staging::output_path(&target)
        })
        .transpose()?;
    if let Some(target) = &target {
        protect(&root, target)?;
        protect(&mods, target)?;
        protect(&out, target)?;
    }
    let _stage_lock = staging::lock(&out, "stage")?;
    let (plan, extras) = build_staged(&root, &mods)?;
    let mut files = horizon_mixins_transform::transform(&root, &plan)?;
    native::stage(&plan, &mut files, extras)?;
    lockfile::complete(&root, &out, &plan, &mut files)?;
    let stage_changed = staging::publish(&root, &out, files, &plan)?;
    let transformation_seconds = start.elapsed().as_secs_f64();
    let mut result = json!({"status":"ok", "stage":out, "injections":plan.injections.len(), "bridges":plan.bridges.len(), "modules":plan.modules.len(), "states":plan.states.len(),
        "crates":plan.crates.len(), "data":plan.data.len(), "post_build":plan.post_build.len(),
        "stage_changed":stage_changed, "transformation_seconds":transformation_seconds});
    if let Some((_, packages)) = check {
        let target = target.unwrap();
        let _target_lock = staging::lock(&target, "cargo")?;
        let cargo_start = Instant::now();
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&out)
            .args(["test", "--locked", "--offline", "--target-dir"])
            .arg(&target);
        if packages.is_empty() {
            cmd.arg("--workspace");
        }
        for package in packages {
            cmd.args(["--package", &package]);
        }
        let run = cmd.output().map_err(io)?;
        let stdout = String::from_utf8_lossy(&run.stdout);
        let stderr = String::from_utf8_lossy(&run.stderr);
        if !run.status.success() {
            return Err(Diagnostic::new(
                "E_BUILD",
                format!(
                    "staged cargo test failed ({}):\n{stderr}\n{stdout}",
                    run.status
                ),
            )
            .file(out.join("Cargo.toml")));
        }
        result["cargo_seconds"] = json!(cargo_start.elapsed().as_secs_f64());
        result["target_dir"] = json!(target);
        result["cargo_stdout"] = json!(stdout);
        result["cargo_stderr"] = json!(stderr);
    }
    result["seconds"] = json!(start.elapsed().as_secs_f64());
    Ok(result)
}
fn protect(source: &Path, output: &Path) -> Result<()> {
    if output.starts_with(source) || source.starts_with(output) {
        return Err(Diagnostic::new(
            "E_INVALID_PATH",
            "output/target must be separate from and cannot contain workspace, mods or stage",
        )
        .file(output));
    }
    // Existing paths have already been canonicalized after rejecting symlinks.
    if output.is_file() && source == output {
        return Err(Diagnostic::new("E_INVALID_PATH", "cannot overwrite source"));
    }
    Ok(())
}
