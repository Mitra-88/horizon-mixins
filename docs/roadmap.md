# Roadmap / acceptance matrix

## Phase 0 — Feasibility
Create two-crate Cargo fixture testing a same-crate injected helper, access to private fields, borrowing and dependency cycles. Write down supported binding design.

## Phase 1 — Parsing and indexing
Workspace/CLI skeleton; TOML parser; explicit module tree, free functions and impl methods; precise ambiguous/missing errors.

## Phase 2 — Planning
JSON plan of all injections resolved against original source; ordinals, require counts, deterministic ordering; reject conflicting overwrites before mutation.

## Phase 3 — AST transformer
HEAD, BEFORE_CALL, OVERWRITE; generation snapshots; positive/negative compilation-tested fixtures.

## Phase 4 — Staging/build cache
Separate immutable-original workspace; no-change file preservation; locking; cargo test of output; stable build directories and profiling.

## Phase 5 — Upstream Horizon prototype
Pin Horizon commit; choose a small low-risk game function, run isolated source patch and compilation. Only integrate launcher through a separate maintainer-approved change.

## Phase 6 — Advanced Mixins
Add AFTER_CALL, REDIRECT, RETURN, MODIFY_ARG, MODIFY_VARIABLE with independent semantic specifications/tests; consider optional type-aware analysis.

## Mandatory tests
Positive: two mods at HEAD, BEFORE_CALL preserving original call result, OVERWRITE, stable ordering independent of discovery, two identical runs unchanged outputs/mtimes, staged `cargo test` success.

Negative: unknown/ambiguous function, selector absent, invalid ordinal or require, two overwrites, overwrite+HEAD, duplicate mod IDs, invalid paths, syntax errors, macro-only targets, uncertain cfg, invalid helper/borrow. Every failure must be nonzero with actionable diagnostics.

CI: rustfmt, clippy, unit tests, source snapshots and Linux compilation fixtures first; expand Windows/macOS after validation.
