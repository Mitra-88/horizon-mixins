# horizon-mixins

**Status: design specification; no implemented Mixin engine yet.**

Proposed stable-Rust, build-time source Mixins for [Horizon](https://github.com/ludovic111/horizon), the Rust Minecraft rewrite. This repository is prepared as a Codex implementation handoff.

## Goal

Let trusted native mods inject into Horizon Rust functions **before** Cargo compilation without requiring manual runtime hook sites. Match and validate injection locations against original source, perform deterministic AST transformations, build in a separate staging workspace, and cache compilations and completed executables. Preserve Horizon's existing WebAssembly mod system.

```text
Pinned Horizon source + mod manifests
       ↓
Index original functions and validate targets
       ↓
Resolve and conflict-check combined injection plan
       ↓
Transform Rust AST with syn, stage generated workspace
       ↓
Pinned Cargo/rustc + incremental build cache
       ↓
Modded executable
```

## Implementation documentation

1. [AGENTS.md](AGENTS.md) — Codex implementation instructions, MVP acceptance criteria.
2. [Architecture](docs/architecture.md) — crates, indexing, data flow, privacy and dependency-cycle issues.
3. [Mixin API](docs/mixin-api.md) — proposed HEAD, BEFORE_CALL and OVERWRITE syntax, future injection types.
4. [Transformations](docs/transformations.md) — syntactic selector matching, rewrite ordering, validation and diagnostics.
5. [Build and cache](docs/build-and-cache.md) — launcher pipeline, stable staging and Cargo reuse.
6. [Security](docs/security.md) — unsandboxed native code and supported boundaries.
7. [Roadmap](docs/roadmap.md) — staged implementation, compile-tested fixtures, CI.
8. [Manifest example](examples/horizon-mixin.toml) — illustrative only.

## Verified Horizon context (2026-10-27)

Horizon's Cargo workspace contains `horizon-core`, `horizon-world`, `horizon-client`, `horizon-server`, `horizon-mod-sdk`, `horizon-mods`, and `cargo-horizon-mod`, among others. Its launcher lives separately in `launcher/`; mods currently use Rust compiled to WebAssembly. Root Cargo.toml declares edition 2024 / minimum Rust 1.85 while README requests Rust 1.90+ for source builds; confirm toolchain at integration. The fast build profile enables incremental compilation and disables LTO.

## MVP scope

A source-only stable-Rust transformer using `syn`, `quote`, declarative manifests and exact syntax selectors. Implement HEAD, BEFORE_CALL, OVERWRITE, deterministic conflict resolution, immutable input, staged output and compilation-tested fixtures. Not full semantic Rust resolution, macro expansion, arbitrary local injection, MIR rewriting or runtime binary patching.

**Important:** Source rewriting does not bypass Rust privacy, type checking, borrow checking or Cargo dependency cycles. Native mods are unsandboxed and must be explicitly trusted.

Respect Horizon's public-tree and Minecraft asset policies. No Minecraft game assets, decompiled Java or prepared resources.

## License

No license selected yet. Repository owner should select one before accepting external contributions or distributing binaries.
