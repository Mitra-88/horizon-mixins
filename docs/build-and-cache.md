# Launcher build and cache

## Pipeline
Resolve exact Horizon source commit, Cargo.lock, mod versions and dependencies, toolchain and target. Verify bundles. Produce validated plan, stage only changed files under a stable profile directory, then compile with pinned Cargo. Atomically publish binary only after success and reuse last working binary on failure.

## Cache layers
1. **Source transformation:** source-file fingerprint + transformer version + applicable mods/patches → staged bytes.
2. **Cargo artifacts:** stable workspace paths and compatible target/profile/rustc config; preserve incremental compilation metadata.
3. **Final executable:** complete fingerprint including game revision, all mod sources, lockfile/dependencies, target triple, rustc, features, rustflags, profile, relevant environment and build scripts.

## Rules
Do not rewrite files if bytes unchanged (maintain mtimes), recreate target dirs on every invocation, share mutable targets across concurrent builds without locks, or assume rustc incremental artifacts are portable across machines/toolchains. Cargo invalidation is often crate-granular: changing a core crate may recompile dependents and relink. Benchmark baseline, no-change, single-mod-change, and shared-core-change builds.

Horizon defines a `fast` profile with LTO disabled, 16 codegen units and incremental compilation enabled. Test profile suitability. Portable compiler distribution needs verified downloads plus OS-specific toolchains/linker prerequisites.

## Launcher integration (future)
Use a JSON status/progress protocol. Distinguish validation failures, incompatible upstream revision, build failures and cache hits. Never make launcher integration prerequisite for standalone tested CLI MVP.
