# Architecture

## Stages
1. Discover pinned Horizon revision/toolchain and mod manifests.
2. Build module/function index from Cargo metadata + source files using `syn`.
3. Resolve every selector against the **original** AST, not prior mods' output.
4. Construct canonical patch plan and preflight conflicts.
5. Transform a copied syntax tree; generate staged source and bridge code.
6. Compile staged workspace with pinned Cargo/rustc; publish a fingerprinted executable.
7. Eventually add a launcher-facing JSON integration.

## Suggested workspace
```text
crates/
  horizon-mixins-core/       # manifests, IR, selectors, planner, errors
  horizon-mixins-transform/  # syn AST visitor, code generation
  horizon-mixins-cli/        # plan, apply, check
fixtures/                    # standalone game crate, mod snippets, tests
docs/
```

## Index and naming
Index module tree including `mod name { }`, `mod name;`, `#[path]`, free functions, inherent impls and `impl Trait for Type`. Include crate, module, impl identity and function in target IDs. Rust `cfg` and macro-generated functions cannot be confidently inferred by a syntax-only pass; reject unsupported targets. Same-named methods require explicit disambiguation. Fingerprint original body/site for version compatibility.

## Major integration issue: privacy and crate cycles
A function patched inside `horizon-world` cannot simply call private code in a separate mod crate; Rust privacy remains. Also game→mod→game introduces Cargo dependency cycles. Prototype a generated **same-crate shim** (or inlined mod-specific helper), with explicit validated context bindings, and compile a fixture to prove visibility/lifetimes. For later versions a stable typed bridge API can expose game types safely.

## IR (illustrative)
```rust
enum Kind { Head, BeforeCall, Overwrite }
struct Injection {
    mod_id: String,
    injection_id: String,
    target: String,
    kind: Kind,
    selector: Option<String>,
    ordinal: Option<usize>,
    require: usize,
    priority: i32,
}
```
Plan record contains original file path, target, AST site path, original fingerprint, insertion order, required helper and compatibility metadata. Stable serialization permits launcher diagnostics and caching.

## Constraints
Keep original checkout unchanged, pin revisions and lockfiles, avoid distributing Minecraft assets or prepared resources. Later compiler-aided resolution may augment the syntactic core but must not be required for MVP.
