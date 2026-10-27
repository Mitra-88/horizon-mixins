#!/usr/bin/env python3
"""Export and test the pinned horizon-core crate; never write to the upstream clone."""
import argparse
import io
import json
import pathlib
import subprocess
import tarfile
import time
from benchmark import compilation_seconds

PIN = "028f60881678a9a0e3e198f693a82858fa475c76"
ROOT = pathlib.Path(__file__).resolve().parents[1]


def run(args, log_path=None, **kwargs):
    result = subprocess.run(args, capture_output=True, text=True, **kwargs)
    if log_path:
        log_path.write_text(result.stderr)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--horizon", type=pathlib.Path, required=True)
    parser.add_argument("--out", type=pathlib.Path, default=ROOT / "target/horizon-prototype")
    args = parser.parse_args()
    upstream = args.horizon.resolve()
    out = args.out.resolve()
    if upstream == out or upstream in out.parents or out in upstream.parents:
        parser.error("prototype output must be separate from the upstream clone")
    if out.exists():
        parser.error("use a fresh --out directory; prior prototype artifacts are preserved")
    archive = subprocess.run(["git", "-C", str(upstream), "archive", PIN,
                              "Cargo.toml", "Cargo.lock", "crates/horizon-core"],
                             check=True, capture_output=True).stdout
    original = out / "original"
    original.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(original, filter="data")
    # Isolate only the unmodified, pinned horizon-core source and its exact libm dependency.
    # This avoids preparing Minecraft data or resolving graphics/audio/launcher dependencies.
    (original / "Cargo.toml").write_text('''[workspace]
members = ["crates/horizon-core"]
resolver = "2"
[workspace.package]
edition = "2024"
license = "MIT"
rust-version = "1.85"
[workspace.lints.clippy]
too_many_arguments = "allow"
type_complexity = "allow"
needless_range_loop = "allow"
large_enum_variant = "allow"
''')
    (original / ".horizon-revision").write_text(PIN + "\n")
    (original / "rust-toolchain.toml").write_bytes((ROOT / "fixtures/game/rust-toolchain.toml").read_bytes())
    # Cargo reduces the exported workspace lockfile to the pinned crate's dependency set.
    run(["cargo", "generate-lockfile", "--offline", "--manifest-path", str(original / "Cargo.toml")])
    probe = original / "crates/horizon-core/tests/horizon_mixins_probe.rs"
    probe.write_text('''#[test]
fn both_independent_head_hooks_execute() {
    for (input, message) in [(1337.0, "alpha-hook"), (1338.0, "beta-hook")] {
        let panic = std::panic::catch_unwind(|| horizon_core::jmath::atan(input)).unwrap_err();
        assert_eq!(panic.downcast_ref::<&str>().copied(), Some(message));
    }
}
#[test]
fn ordinary_atan_result_is_preserved() {
    assert_eq!(horizon_core::jmath::atan(0.5).to_bits(), libm::atan(0.5).to_bits());
}
''')
    run(["cargo", "build", "--locked", "-p", "horizon-mixins-cli"], cwd=ROOT)
    binary = ROOT / "target/debug/horizon-mixins"
    mods = ROOT / "fixtures/horizon-mods"
    stage = out / "stage"
    target = out / "cargo-target"
    baseline_start = time.monotonic()
    # Run upstream tests before adding transformed hooks. The exported probe intentionally
    # expects transformed behavior, so baseline runs only the upstream library suite.
    baseline = run(["cargo", "test", "--locked", "--offline", "--manifest-path", str(original / "Cargo.toml"),
                    "--lib", "--target-dir", str(target)], log_path=out / "baseline.cargo.log")
    measurements = {"baseline_seconds": round(time.monotonic() - baseline_start, 4),
                    "baseline_cargo_compile_seconds": compilation_seconds((out / "baseline.cargo.log").read_text())}
    for label in ["first_modded", "no_change"]:
        report = json.loads(run([str(binary), "check", "--workspace", str(original),
                                 "--mods", str(mods), "--out", str(stage),
                                 "--target-dir", str(target), "--trust-native-mods"]))
        measurements[label] = {key: report[key] for key in ["stage_changed", "transformation_seconds", "cargo_seconds"]}
        measurements[label]["cargo_compile_seconds"] = compilation_seconds(report["cargo_stderr"])
        (out / f"{label}.json").write_text(json.dumps(report, indent=2) + "\n")
    result = {"revision": PIN, "scope": "isolated complete horizon-core crate, not full game/launcher",
              "measurements": measurements, "stage": str(stage)}
    (out / "baseline.txt").write_text(baseline)
    (out / "report.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
