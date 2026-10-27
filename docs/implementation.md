# Feasibility, acceptance and measurements

Implemented and exercised on 2026-10-28 with stable rustc 1.95.0
(`59807616e`), Linux x86_64. The Cargo workspace and fixtures commit their
lockfiles; the fixture also carries its toolchain pin into staging.

The phase-0 prototype was compiled before implementation of the engine. A
private `Entity.velocity` field is modified by a local helper taking
`&mut Entity`, called with an explicit reborrow of `self`; the separate driver
crate verifies two successive results. The positive two-crate fixture passed.
A separate mod crate trying to access that field failed Rust privacy checking.
A game→mod→game fixture failed Cargo's cyclic dependency check. These are real
compiler/Cargo failures asserted by `tools/feasibility.py`, not simulated errors.

The resulting binding design compiles one ordinary helper function directly
inside the target's lexical module/function scope. Every context value comes
from a declared target parameter binding. Concrete helper argument types,
return types, private access and reborrows are checked by staged Cargo tests.
The MVP adds no dependencies and requires no cross-crate private visibility.
See [mixin-api.md](mixin-api.md) for syntax and limits.

The main fixture has a game crate and a driver crate, plus two independent mod
bundles. Its four runtime tests prove private-field HEAD ordering, function-call
result/exactly-once evaluation, method receiver/argument evaluation, and
OVERWRITE. The engine integration suite adds source/plan snapshots, manifest
and selector failures, cfg/macros, ambiguity, conflicts, borrowing/return-type
compilation failures, concurrent checks, immutable originals and preserved
mtimes. All 16 integration tests pass. Linux CI runs rustfmt, clippy with
warnings denied, phase-0 compilation and the integration suite.

A first four-scenario fixture measurement produced:

| Scenario | Transform/stage wall | Cargo compile | Cargo build + tests |
|---|---:|---:|---:|
| First modded build | 0.056 s | 0.13 s | 0.171 s |
| Unchanged run | 0.053 s | 0.00 s | 0.042 s |
| One helper changed | 0.051 s | 0.09 s | 0.129 s |
| Shared game source changed | 0.053 s | 0.08 s | 0.128 s |

Measured platform: `Linux-7.3.0-2-cachyos-x86_64-with-glibc2.44`.
These tiny builds establish cache reuse; they do not predict full-game compile
time. `tools/benchmark.py` stores raw Cargo logs/reports under its fresh output
directory. The Cargo compile column is Cargo's reported test-profile duration;
the wall column includes executing tests and doc tests.

The Horizon experiment exports the complete `horizon-core` crate from local
Git commit `028f60881678a9a0e3e198f693a82858fa475c76`. Its crate sources remain
unchanged before transformation. The experiment replaces only the exported
root workspace manifest to isolate that crate, reduces the exported lockfile
to its existing exact `libm = 0.2.16` dependency, adds an export revision stamp,
and adds two test probes. The local Horizon checkout is never edited.

Two separately declared HEAD hooks target `horizon_core::jmath::atan`. Each
panics at a different sentinel input, and the probe catches the exact messages
to prove both hooks execute. A normal input retains the original bitwise result.
All upstream library and integration tests pass: 106 library tests, 8 existing
integration tests, 2 added probes, and 2 deliberately ignored upstream
microbenchmarks. No Minecraft resources are needed.

The first upstream run measured 0.377 s transformation/staging, 2.71 s Cargo
compilation and 12.324 s Cargo-plus-tests. An identical run preserved the stage
and reported 0.00 s Cargo compilation; its 9.606 s wall time is primarily the
upstream tests, rather than recompilation. The baseline untransformed library
test run took 11.884 s including compilation and execution (2.33 s reported compilation). Fresh repetitions
will differ. Run `tools/horizon_prototype.py` with a fresh `--out` to generate
raw baseline/Cargo logs and a report on the current machine.

This validates the isolated complete foundation crate, not the full Horizon
game, graphics/audio stack, prepared assets, launcher, or production gameplay
compatibility. Advanced injection semantics, semantic resolution, cross-platform
atomic replacement and executable publication/caching remain future work.
