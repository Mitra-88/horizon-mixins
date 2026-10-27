# AST rewriting and conflict semantics

All targets and sites resolve on pristine source. A function has a stable
module/impl identity, source item-index path, and signature/body fingerprint.
A call has an original preorder index and syntax fingerprint. Rewriting never
matches helpers emitted by another injection.

Compatible same-site inserts execute in priority descending, mod ID ascending,
injection ID ascending order. Priorities never select an overwrite winner.
Two overwrites, or OVERWRITE plus any other hook on the same physical function,
fail with `E_OVERWRITE_CONFLICT` during planning, before stage publication.

HEAD prepends separate blocks, each defining and invoking a local helper.
OVERWRITE replaces the body with a local helper and a tail call, retaining the
surrounding original function signature and attributes.

BEFORE_CALL replaces a selected call with an expression block:

```rust
let result = {
    { fn before() {} before() };
    original(argument())
};
```

The hook runs before receiver/callee and arguments, and the original call is
emitted exactly once. Its result remains the tail expression. For a discarded
expression statement, the original call instead retains a semicolon inside the
generated block; this preserves destruction of a returned value before its
receiver temporary. A separate compiled Drop-tracing test covers that order. The compiled
fixture tests both a function call and a side-effecting method receiver, checks
argument/receiver counts and returned values, and uses two hooks at one call
site. A helper deliberately contains another matching call to prove it does
not change original `require` or `ordinal` resolution.

An extra block can shorten temporary lifetimes or change drop order inside a
larger expression. Consequently the selected call must be the complete
expression of a statement, a plain identifier `let` initializer, or a block or
function tail (parentheses are allowed). Binary operands, call arguments,
conditions and match-arm expressions are rejected as `E_UNSUPPORTED_SITE`.
Reference-wrapped calls, closures/async/await/try/`?`, nested items, unsafe/const
contexts, cfg and opaque macros fail closed. Parameter shadowing is rejected
for BEFORE_CALL so bindings cannot silently refer to a different local.
Compilation remains necessary to validate actual borrowing and return types.

Representative diagnostics:

| Code | Meaning |
|---|---|
| `E_TARGET_NOT_FOUND` | Exact source function is absent |
| `E_TARGET_AMBIGUOUS` | Multiple definitions or shared source identities |
| `E_MATCH_COUNT`, `E_ORDINAL` | Selector compatibility mismatch |
| `E_OVERWRITE_CONFLICT` | Overwrite is not exclusive |
| `E_UNSUPPORTED_MACRO`, `E_UNSUPPORTED_CFG` | Syntax cannot be resolved safely |
| `E_UNSUPPORTED_SIGNATURE`, `E_UNSUPPORTED_CONTROL_FLOW`, `E_UNSUPPORTED_SITE` | Unsupported source context |
| `E_MANIFEST`, `E_DUPLICATE_ID`, `E_INVALID_HELPER`, `E_INVALID_BINDING` | Invalid mod declaration |
| `E_INVALID_PATH`, `E_STAGE_OWNERSHIP` | Unsafe input/output location |
| `E_UNPINNED`, `E_REVISION`, `E_FINGERPRINT`, `E_STALE_SOURCE` | Compatibility/source mismatch |
| `E_BUILD` | Staged Cargo failed; compiler output included |

Diagnostics carry code/message and applicable file, mod, injection, target and
selector. The CLI exits nonzero and prints JSON on stderr. Source and plan
snapshots live in `crates/horizon-mixins-cli/tests/snapshots`; update deliberately
with `UPDATE_SNAPSHOTS=1 cargo test --workspace --locked`.
