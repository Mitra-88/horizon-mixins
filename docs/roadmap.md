# Roadmap and acceptance matrix

Phases 0–4 are implemented. Phase 5 is verified for an isolated complete
`horizon-core` crate at `028f60881678a9a0e3e198f693a82858fa475c76`.
Full game and launcher integration remain future work.

| Phase | Result |
|---|---|
| 0 — Feasibility | Two-crate compilation fixture proves local bridge/private fields/reborrowing; private cross-crate access and dependency cycles fail as expected |
| 1 — Parsing/index | Three-crate CLI workspace; strict TOML; inline/external/path modules; free/inherent/trait functions; missing/ambiguous/cfg/macro diagnostics |
| 2 — Planning | Canonical JSON; original-site fingerprints; explicit counts/ordinals; deterministic priority/ID order; overwrite conflicts before mutation |
| 3 — Transform | HEAD, BEFORE_CALL and OVERWRITE; source/plan snapshots; compiled behavior, borrowing and return-type tests |
| 4 — Staging/Cargo | Immutable originals; atomic Linux publication; unchanged mtimes; stable locked target directories; concurrency tests and four-scenario benchmark |
| 5 — Upstream prototype | Isolated pinned horizon-core export, two HEAD hooks in jmath::atan, upstream tests and hook probes; no assets/upstream edits |
| 6 — Advanced mixins | Future: independent semantics/tests for AFTER_CALL, RETURN, REDIRECT, MODIFY_ARG, MODIFY_VARIABLE and semantic matching |

`cargo test --workspace --locked` checks two independent mods, same-site priority
and ID tie order, original selectors unaffected by injected calls, call results
and exactly-once arguments/receiver evaluation, overwrite success/conflicts,
source/plan snapshots, discovery order independence, repeated bytes/mtimes,
immutable originals, removed files, concurrent Cargo checks, and stale source.
It also checks missing/ambiguous targets, wrong counts/ordinals, duplicate IDs,
malformed manifests/helpers, invalid paths, cfg/macros, unproven temporal scopes,
parameter shadowing and compiler-diagnosed helper borrow/return failures.

Linux CI runs the phase-0 script, rustfmt, clippy with warnings denied, and the
integration suite. Expand atomic staging and compilation CI to Windows/macOS
after implementing their directory-publication strategy. Launcher work requires
a separate change; it is not part of this MVP.
