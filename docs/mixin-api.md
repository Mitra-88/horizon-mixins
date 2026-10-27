# Declarative MVP API

Mods are trusted Rust snippets described by TOML, rather than proc-macro
attributes. `--mods DIR` discovers direct `.toml` files and
`DIR/<bundle>/horizon-mixin.toml`. Discovery order has no semantic effect.
Unknown fields, duplicate IDs and missing helpers fail validation.

```toml
[mod]
id = "better-movement"
version = "0.1.0"
priority = 100 # default; each injection can override it

# Optional, exact 40-character commit; see the isolated Horizon prototype.
[compatibility]
horizon_revision = "028f60881678a9a0e3e198f693a82858fa475c76"

[[injection]]
id = "before-second-consume"
target = "fixture_game::calls::compute"
kind = "BEFORE_CALL"
selector = "consume"
ordinal = 1
require = 2
helper = "before.rs"
bindings = [{ name = "log", ty = "&mut Vec<&'static str>", value = "&mut *log" }]
# fingerprint = "<SHA-256 from the original plan>" # optional compatibility guard
```

`horizon_revision` is checked against Git HEAD or the `.horizon-revision` stamp
of a controlled source export. A stamp is provenance metadata, not cryptographic
attestation. The source files, Cargo.lock, toolchain file when present, and all
other staged inputs also have SHA-256 fingerprints in the plan.

Helper paths must be relative to their manifest directory, without traversal
or escapes through symlinks. A helper file contains exactly one private,
synchronous, safe, non-generic function without attributes. It becomes a local
function in a separate block inside the target. Independent helpers can use
the same Rust function name. Module types, imports and private fields are in
scope; outer function locals and `Self` are not implicit context.

```rust
fn before(log: &mut Vec<&'static str>) {
    log.push("before");
}
```

Every helper argument needs a binding in declaration order. `name` and `ty`
must exactly match its parsed parameter name/type. `value` is a target parameter,
its borrow, or explicit dereference/reborrow (`self`, `&self`, `&mut *entity`).
Calls, fields and arbitrary locals are not binding expressions. Use explicit
concrete types in helper signatures. BEFORE_CALL rejects parameter shadowing.
Rust compilation validates the actual types, mutability, lifetimes and moves.

HEAD inserts helper calls before the original first statement. BEFORE_CALL
inserts before receiver/callee and argument evaluation, preserving one original
call as the final expression. OVERWRITE replaces only the body with a helper
call as its tail, retaining the original attributes/signature. Insertion helpers
return `()`; overwrite return compatibility is checked by Cargo. HEAD and
OVERWRITE do not accept selector/count/ordinal fields.

Function IDs include Cargo target name (hyphens become underscores), module,
impl identity, and function:

| Source | Target ID |
|---|---|
| Free function | `fixture_game::entity::advance` |
| Inherent method | `fixture_game::entity::Entity::tick` |
| Trait impl method | `fixture_game::entity::<Entity as Marker>::marked` |

Selectors are syntax, not semantic resolution: `consume`, `crate::consume`,
`entity.apply_friction`, or `make_receiver(evaluations).take`. Whitespace is not
accepted. `require` counts all exact matches in the original body, and
`ordinal` selects one in AST preorder. A selector is never matched against an
injected helper. Counts and ordinals are mandatory even when only one match
is expected. Bare method names do not match qualified receiver expressions.

BEFORE_CALL supports complete expression statements, plain identifier `let`
initializers and block/function tails. Other placements fail with
`E_UNSUPPORTED_SITE`. Functions containing opaque macros, unresolved cfg,
closures, async, await, `?`/try, unsafe/const blocks, nested items or calls under
references are outside the proven MVP subset. Generic, const, unsafe and
foreign-ABI target functions are also rejected. Feature-gated module trees
and Cargo targets are not guessed or expanded.

Future attribute syntax, additional dependencies, arbitrary helper modules,
semantic selectors, AFTER_CALL, RETURN, REDIRECT, MODIFY_ARG and
MODIFY_VARIABLE require separate specifications and tests.
