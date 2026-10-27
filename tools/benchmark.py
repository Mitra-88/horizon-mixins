#!/usr/bin/env python3
"""Measure fixture transformation, Cargo compilation, and tests under four edits."""
import argparse
import json
import pathlib
import platform
import re
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]


def compilation_seconds(log):
    match = re.search(r"Finished .* target\(s\) in ([0-9.]+)(s|ms)", log)
    if match:
        return float(match[1]) / (1000 if match[2] == "ms" else 1)
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=pathlib.Path, default=ROOT / "target/benchmark")
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists():
        parser.error("use a fresh --out directory")
    out.mkdir(parents=True)
    shutil.copytree(ROOT / "fixtures/game", out / "original")
    shutil.copytree(ROOT / "fixtures/mods", out / "mods")
    subprocess.run(["cargo", "build", "--locked", "-p", "horizon-mixins-cli"], cwd=ROOT, check=True)
    results = {"rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
               "platform": platform.platform(), "scenarios": {}}
    for scenario in ["baseline_modded", "no_change", "single_mod_change", "shared_core_change"]:
        if scenario == "single_mod_change":
            helper = out / "mods/alpha/replace.rs"
            helper.write_text(helper.read_text().replace("input * 7", "input * 7 + 0"))
        if scenario == "shared_core_change":
            source = out / "original/engine/src/entity.rs"
            source.write_text(source.read_text().replace("entity.velocity += 1", "entity.velocity += 1 + 0"))
        run = subprocess.run([str(ROOT / "target/debug/horizon-mixins"), "check", "--workspace", str(out / "original"),
                              "--mods", str(out / "mods"), "--out", str(out / "stage"),
                              "--target-dir", str(out / "cargo-target"), "--trust-native-mods"], capture_output=True, text=True)
        if run.returncode:
            raise RuntimeError(run.stderr)
        report = json.loads(run.stdout)
        results["scenarios"][scenario] = {key: report[key] for key in ["stage_changed", "transformation_seconds", "cargo_seconds"]}
        results["scenarios"][scenario]["cargo_compile_seconds"] = compilation_seconds(report["cargo_stderr"])
        (out / f"{scenario}.json").write_text(run.stdout)
    (out / "report.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
