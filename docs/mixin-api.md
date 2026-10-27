# Proposed mod API (not yet implemented)

Start with declarative manifest entries. Attribute syntax below is **future ergonomic sugar**, not valid today.

```rust
#[mixin(target = "horizon_world::entity::update_velocity")]
mod movement {
    #[inject(at = "HEAD")]
    fn at_start(/* declared parameter bindings */) {}

    #[inject(at = "BEFORE_CALL", target = "entity.apply_friction", ordinal = 0, require = 1)]
    fn before_friction(/* validated bindings */) {}
}
```

## v1 semantics
- **HEAD:** insert a helper invocation before the first statement of the resolved target body. Bind arguments by explicit names/types; never assume `entity` or `self` exists.
- **BEFORE_CALL:** insert before a specific AST `Expr::Call` or `Expr::MethodCall` in the target function. Require a syntactic selector plus `ordinal` and `require`. Do not claim semantic trait/method resolution using `syn`.
- **OVERWRITE:** replace body of exactly one target function. Conflicts with all other injections targeting the same function in MVP.
- Priorities order compatible same-site insertion deterministically; they do not resolve overwrites.
- Version restrictions are pinned to exact Horizon commit initially, with optional original-function fingerprint.

## Future
AFTER_CALL, RETURN, REDIRECT, MODIFY_ARG, MODIFY_VARIABLE. RETURN must handle implicit tail return, early return, `?`, async/closures. MODIFY_ARG must preserve argument evaluation and borrow order. MODIFY_VARIABLE must select a particular binding/scope rather than equal text.

Native Mixins require recompilation and are trusted unsandboxed code; Horizon's WASM mods remain separate.
