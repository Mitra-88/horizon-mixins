# Codex implementation instructions

This repo now contains a compilation-tested MVP. Read every doc before coding. Do not implement launcher integration before a small compilation-tested prototype.

## Highest-priority outcome
A working CLI that rewrites a tiny pinned Rust workspace where **two independently declared mods inject into the same function**. Staged output must compile, run tests and be deterministic.

## Deliverables
- Cargo workspace crates: `horizon-mixins-core` (IR, manifests, indexing, planning, diagnostics), `horizon-mixins-transform` (syn rewrite), `horizon-mixins-cli` (CLI).
- Use stable Rust; `syn` full/visit-mut, `quote`, `proc-macro2`, `serde`, `toml`, `cargo_metadata`, `prettyplease`, `thiserror` as useful.
- `plan --workspace PATH --mods PATH --out PATH` outputs validated machine-readable JSON; `apply` stages rewritten source, never mutates originals; `check` runs cargo test on the fixture.
- Start with declarative TOML, *not* proc-macro attributes.
- Index free functions, inline/external modules, inherent methods and trait impl methods; reject unresolved cfg and ambiguous paths.
- Resolve every injection **against pristine original AST** before rewriting. Sort same-site inserts by priority descending, mod ID ascending, injection ID ascending. Reject competing OVERWRITE and OVERWRITE mixed with insertion.
- The baseline implements HEAD, BEFORE_CALL and OVERWRITE. The user-authorized extension adds cancellable HEAD, RETURN, MODIFY_ARG, MODIFY_CONSTANT and WRAP_OPERATION; follow docs/advanced-injections.md. The next user-authorized extension adds local capture, accessor/invoker bridges, slices, MODIFY_ARGS and MODIFY_VARIABLE; follow docs/scopes-and-bridges.md. Version 0.4.0 adds lifetime-method HEAD support and support modules; follow docs/boundary-head-and-modules.md. Syntactic call selectors require explicit match counts and ordinals; never silently skip.
- Test preservation of expression return value and exactly-once argument evaluation. Reject unsupported macro-expanded sites, closures/async cases not proven safe.
- Respect Rust privacy and crate dependency cycles: generated same-crate bridge or another tested approach, not magical cross-crate private access.
- Atomic staging, unchanged-file mtime preservation and locked Cargo target dirs.
- Provide snapshots, positive/negative integration tests, structured errors, CI.

## Definition of done
Two mods inject HEAD in deterministic order and generated workspace passes cargo test. BEFORE_CALL invokes exactly once and preserves original call behavior. OVERWRITE succeeds, conflicts fail during planning. Repeated runs output byte-identical source, without rewriting unchanged files. Missing/ambiguous targets, wrong ordinal/count, duplicate IDs, unsupported macros and malformed mods report actionable diagnostics.

## Workflow
(0) privacy/compilation spike, (1) parsing/index, (2) plan and conflict detection, (3) transform and test, (4) staging/Cargo, (5) isolated test on pinned Horizon commit. No Minecraft assets; no upstream edits without separate PR. Measure compile speed, don't assume it.
