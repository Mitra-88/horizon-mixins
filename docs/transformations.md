# AST rewriting and conflict semantics

## Plan first
Resolve each mod's targets and injection sites on **pristine original** `syn::File` AST. Store stable function identity + original-site syntax path/fingerprint. Validate `require`, ordinal, feature/cfg and supported expression forms. Never match code injected by another mod.

## Deterministic order
At same site, execute injections sorted by (priority descending, mod ID ascending, injection ID ascending). Resolve all sites first, then rewrite once. Two OVERWRITE requests or OVERWRITE plus any other injection into one function are errors, never silent wins.

## HEAD
Prepend generated statements to `syn::Block.stmts`. Require explicitly mapped arguments/context, with borrow-checked generated Rust; don't hard-code local names.

## BEFORE_CALL
Conceptually replace `entity.apply_friction()` by:
```rust
{ generated_bridge::before_friction(/* validated context */); entity.apply_friction() }
```
This block normally preserves the call result. Do not duplicate receiver/argument evaluation; preserve `?`, await/temporaries and lifetimes or fail as unsupported. A bare method name is only a syntactic match, not a resolved trait identity.

## OVERWRITE
Replace only target function body, preserving surrounding attrs/signature; validate helper visibility and return compatibility by staged compilation.

## Diagnostics / fail closed
Suggested error codes: `E_TARGET_NOT_FOUND`, `E_TARGET_AMBIGUOUS`, `E_MATCH_COUNT`, `E_ORDINAL`, `E_OVERWRITE_CONFLICT`, `E_UNSUPPORTED_MACRO`, `E_UNSUPPORTED_CFG`, `E_INVALID_HELPER`. Include file path, mod ID and selector. Macro-generated bodies, uncertain cfg, semantic overload matching and unsupported control-flow rewrites must fail clearly, not silently skip.

## Tests
Snapshots of emitted code, two-mod same-site priority test, exact-once evaluation, unchanged return values, conflict and ambiguity tests, determinism independent of filesystem enumeration, and cargo test of generated fixtures.
