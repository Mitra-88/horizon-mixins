# Preserved standard macros in body-rewriting hooks

`RETURN`, call/site hooks, value/variable modifiers and state constructors can
now coexist with validated `format!`, `matches!` and `vec!` invocations. The
planner parses their inputs for validation and preserves the original macro
invocations during transformation. It does not expand macros or rewrite their
arguments. All existing signature, cfg, borrowing and control-flow restrictions
outside macros still apply; planning alone does not replace `check`.

For example, a RETURN hook can modify the result of this function, or a
BEFORE_CALL hook can target its ordinary `consume` call:

```rust
fn calculate(value: i32) -> i32 {
    let label = format!("value: {value}");
    let values = vec![value; 9];
    let positive = matches!(value, n if n > 0);
    consume(label, values, positive)
}
```

The accepted syntax covers list/repetition `vec!`, positional/named arguments
and a literal format string in `format!`, and patterns/optional guards in
`matches!`. Nested invocations of the same three standard macros are supported.
Unselected ordinary function/method calls inside these inputs remain unchanged.
Implicit format captures also remain unchanged. Rust validates actual macro
syntax, formatting traits, types and lifetimes when compiling the stage.

Macro identity must be unambiguous. Bare names are rejected when matching
macro definitions/imports, glob imports in the destination module, `macro_use`
or opaque ancestor item expansions prevent this proof. Relative `std::`,
`core::` and `alloc::` paths must not be shadowed either. An absolute path such
as `::std::vec!` avoids ordinary local name shadowing; Cargo dependencies
named/renamed `std`, `core` or `alloc`, renamed roots in `extern crate`, and opaque ancestor item
expansions, still cause rejection. No dependency is added to make a path resolve.

## Rejected inputs and hidden sites

- Inputs containing `return`, `?`, `break`, `continue`, closures, async/await,
  yield, unsafe/const blocks, unsupported items, unresolved cfg/attributes or
  other opaque macros fail with `E_UNSUPPORTED_MACRO` or the corresponding
  existing syntax/context diagnostic. Even internally scoped control flow is
  conservatively rejected. Move it into ordinary source outside the macro.
- A selector that also matches a call, constant or expression inside parsed
  inputs fails with `E_UNSUPPORTED_MACRO_SITE`. The planner never silently
  undercounts those matches, even if another matching site is outside the macro.
  Slice anchors are checked too; hidden matches are rejected conservatively
  even when they might be outside the requested slice.
- A variable modifier whose selected name occurs inside inputs is rejected.
  Format strings are checked for identifiers too, including implicit captures
  and dynamic width/precision. Literal text or shadowed names can therefore
  cause conservative rejections. Local captures/selectors cannot target macro
  internals, and state constructor literals inside inputs cannot receive added
  fields. Selecting an entire macro with MODIFY_EXPRESSION_VALUE remains outside
  that hook's supported value shapes; RETURN can wrap a macro tail result.

HEAD-only hooks retain their broader preservation contract, including arbitrary
expression/statement macros and macro-generated early returns. When another
hook requires body analysis, every macro in that function must meet the narrower
contract above. See [HEAD boundaries](boundary-head-and-modules.md).

The integration suite compiles all existing hook fixtures with nested standard
macros added, checks argument evaluation order/counts and owned-value drops,
verifies macro tokens remain unchanged, and exercises macro tail RETURN values.
Negative cases cover hidden exits/sites/variables/constructors and macro name
shadowing. Repeated staging preserves original files, bytes and mtimes.
