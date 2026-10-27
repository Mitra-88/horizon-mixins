# Architecture

The implemented workspace has three crates:

- `horizon-mixins-core`: strict manifests, source snapshots, module/function
  indexing, original-site IR, conflict detection and structured diagnostics.
- `horizon-mixins-transform`: pure `syn` AST rewrite and `prettyplease` output.
- `horizon-mixins-cli`: `plan`, `apply`, `check`, atomic staging and Cargo locks.

The planner snapshots inputs, asks `cargo_metadata` for workspace libraries and
binaries with `--no-deps --locked --offline`, and walks their original module
trees. It handles inline modules, external `mod.rs`/`name.rs`, `#[path]`, free
functions, inherent impls and trait impls. It rejects external path dependencies,
recursive/escaping module files, ambiguous identities and source files compiled
under more than one module/target identity. Tests/examples/benches are copied
and compiled by Cargo but are not injection targets in the MVP.

Every injection resolves before any transformation. The schema-1 JSON plan
records transformer/rustc version, every input file's SHA-256, mod identity and
version, target ID, original file and item-index path, signature/body fingerprint,
call preorder number/fingerprint, selector/count/ordinal, priority, helper source,
and explicit bindings. It contains no absolute checkout paths or discovery paths,
so bundle enumeration does not affect serialized bytes.

The transformer verifies input/toolchain fingerprints, groups patches by source
function, numbers only original call nodes in preorder, and rewrites their
children before replacing each selected node. It never traverses generated
helper code. It applies HEAD statements after rewriting original call sites.
This preserves original coordinates even when hooks contain matching calls.
Only files with target functions are pretty-printed; other files retain bytes.
Non-doc comments in transformed files are not preserved by `syn`/`prettyplease`.

The privacy bridge is a local function inside the patched body. Explicit
parameter bindings supply all context, with Rust enforcing privacy, moves,
borrows and return types during `check`. No mod crate becomes a dependency of
the game, so this design introduces no Cargo cycle. The phase-0 two-crate fixture
proves private field access/reborrowing and checks that cross-crate private
access and circular dependencies actually fail. Native helpers use the target
crate's existing dependency/import environment; adding dependencies is future work.

The CLI rebuilds a validated plan for each invocation, transforms in memory,
then publishes a complete separate workspace. Saved JSON plans are reviewable
artifacts and are not executable CLI inputs. The transform library validates
IR structure and original fingerprints before traversing it; callers should
obtain semantic plans through `build_plan`.

See [staging](build-and-cache.md), [supported bindings](mixin-api.md), and
[prototype results](implementation.md). Compiler-aided resolution, launcher
protocols and fingerprinted executable publication remain future work.
