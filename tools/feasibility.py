#!/usr/bin/env python3
"""Compile the bridge and assert Rust rejects private access and crate cycles."""
import json
import pathlib
import subprocess
import time

root = pathlib.Path(__file__).resolve().parents[1]
results = []
for fixture, command, expected in [
    ("feasibility", "test", None),
    ("privacy-failure", "check", "private"),
    ("cycle", "check", "cyclic package dependency"),
]:
    start = time.monotonic()
    run = subprocess.run(
        ["cargo", command, "--offline", "--locked", "--manifest-path", str(root / "fixtures" / fixture / "Cargo.toml"),
         "--target-dir", str(root / "target" / "feasibility")], capture_output=True, text=True,
    )
    assert (run.returncode == 0) == (expected is None), run.stdout + run.stderr
    if expected:
        assert expected in run.stderr, run.stderr
    results.append({"fixture": fixture, "seconds": round(time.monotonic() - start, 3),
                    "expected_failure": expected is not None})
print(json.dumps(results, indent=2))
