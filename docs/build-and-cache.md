# Staging, Cargo reuse and measurements

`apply` transforms all validated inputs in memory and assembles a complete
sibling temporary directory. The first stage is published with directory
rename. Linux updates use `renameat2(RENAME_EXCHANGE)` so a reader sees a
complete old or new workspace. A failed plan, transformation or exchange leaves
the prior stage intact. Old staged generations are cleaned up after exchange.
Atomic replacement currently requires Linux; another platform can publish a
fresh output directory, but cannot replace an existing stage.

A stage marker prevents replacing arbitrary directories. A sibling advisory
stage lock serializes publishers and remains held throughout `check`'s Cargo
process. The configurable Cargo target directory has its own advisory lock,
including when different stages share it. Locks release on process exit; Cargo
also retains its own internal locks. Direct user Cargo commands do not acquire
the CLI's stage lock, so use `check` while staging concurrently.

If all bytes are unchanged, staging does nothing. Changed generations hard-link
unchanged files only from the previous stage, preserving file mtimes and modes;
original input files are never hard-linked or modified. New/rewritten files are
written independently. Removed input files disappear on the next publication.
Build directories, `.git`, `.local`, `.horizon-mixins` and `node_modules` are
excluded. Other symlinks and special files are rejected. Use source-only exports
for large upstream trees containing unrelated runtime data.

`check` executes only `cargo test --workspace --locked --offline`, optionally
limited by repeated `--package`. `--trust-native-mods` explicitly opts into
unsandboxed native execution. It captures compiler/test logs and reports timing
as JSON. A failed compile retains the staged source for inspection and returns
`E_BUILD`; it does not publish a binary. Stable sibling target paths are reused
across no-change and mod-change builds.

`tools/benchmark.py` measures a first modded build, unchanged run, one helper
change, and a shared game-source change. It stores transformation time, Cargo
build-plus-test wall time, and Cargo's reported compile duration, plus platform
and rustc version. Each invocation needs a fresh `--out`. The Horizon prototype
also records a baseline library test and two staged Cargo test runs. Results
are documented in [implementation.md](implementation.md); they are measurements
on one machine, not promises about game rebuild speed.

The full launcher cache is future work: source transformation cache entries,
complete environment/feature/rustflag/build-script fingerprints, verified toolchain
packages, final executable caching, successful-binary publication and rollback.
No launcher integration is implemented. Rust incremental artifacts are not
assumed portable. Horizon's `fast` profile is not selected automatically; the
MVP uses Cargo's test profile and measures it.

Expression, after-hook and state changes use the same atomic stage, unchanged
mtime preservation and Cargo target lock. Constructor source, explicit state
initializers and helper sources participate in the canonical plan/stage identity.
No runtime injection or save-data migration is performed by staging.
