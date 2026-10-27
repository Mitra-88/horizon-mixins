# Lifetime methods, boundary HEAD hooks and support modules (0.4.0)

Veinminer is the first full-game consumer of this extension. Plans use schema 5;
regenerate older plans. It declares three cancellable HEAD injections and one
support module. All changes to the exported Horizon Rust source are performed
by the mixin planner/transformer and atomic staging CLI.

## Lifetime-bearing methods

Inherent impls may declare lifetime parameters, including `impl<'a> Game<'a>`.
Their self-type IDs normalize non-static lifetimes to `'_`, so a renamed
lifetime does not change a target ID. A method in an external `commands` module
is identified by its declaring module:

```toml
[[injection]]
id = "command"
kind = "HEAD"
cancellable = true
target = "horizon_client::game::commands::Game<'_>::run_command"
helper = "command.rs"
bindings = [
  { name = "game", ty = "&mut crate::game::Game<'_>", value = "&mut *self" },
  { name = "command", ty = "&str", value = "command" },
]
```

Type/const generic impls, impl where clauses, explicitly generic methods and
async/unsafe/const/foreign targets remain unsupported. Helpers still use
concrete types and do not capture the target's named lifetime parameters.
Compiler checking is required for actual lifetime and borrow compatibility.

## Preserved body control flow

HEAD can prepend a call before a body containing closures, `?`, reference
expressions or nested ordinary items. It does not inspect or rewrite their
sites or exits. If a cancellable hook continues, the original body executes
with its original returns, error propagation and destruction behavior. If it
breaks, normal Rust return semantics drop the target's owned arguments once.

Source macros, unresolved cfg and attribute macros still fail closed. Async
functions remain rejected. RETURN and site injections continue to use their
stricter control-flow validation, including when combined with HEAD. HEAD-only
planning does not assign expression/local coordinates inside the untouched body;
the transform skips expression traversal when there are no planned sites.

The compilation fixture tests two mods in priority order, closure errors,
owned argument destruction, cancellation, stable lifetime IDs and deterministic
source/mtimes. It also tests rejection of RETURN over `?`, type generics,
source macros, ambiguous module destinations and unsupported payloads.

## Support modules

A bundle can contribute a self-contained Rust module to the top level of an
existing indexed source file:

```toml
[[module]]
id = "runtime"
file = "crates/horizon-client/src/game.rs"
name = "veinminer"
source = "runtime.rs"
```

`source` is relative to the bundle and contains the module's items, without an
outer module wrapper. The transformer emits `pub(crate) mod veinminer { ... }`
after all edits to original functions and bridges. Its types/functions are
available to helpers through ordinary Rust paths and same-crate privacy.
It adds no Cargo dependencies or fields to existing structs. IDs are unique
across injections, bridges and modules in a bundle.

The destination must have exactly one unconditional source identity. Existing
type/module/import names, competing module names and unresolved glob imports
are rejected. Payloads are parsed during planning. External `mod`, `#[path]`,
direct include macros, file attributes and shebangs are unsupported; assemble
multi-file mod implementations into a self-contained payload before planning.
Ordinary Rust macros and cfg inside new implementation code are compiled by
Cargo; injected helpers are never matched against that code.

Plans contain module identity/version, destination fingerprint and source.
Original fingerprints are checked before function rewrites; module insertion
happens afterward so original node paths cannot shift. Changed support code
changes the plan and staged file; repeated identical runs preserve bytes and
mtimes. `E_MODULE` and `E_MODULE_CONFLICT` describe payload/destination errors.
Saved JSON remains a review artifact, not an executable CLI input.

## Veinminer integration

The separate `horizon-veinminer` repository commits `mixins/veinminer/` with its
manifest, three typed hooks and runtime payload. Its development tool exports
committed Horizon files, then invokes `horizon-mixins plan` and `apply`; it has
no custom Rust-source replacement logic. The runtime keeps UUID-keyed toggles
and cooldowns in per-world state on the hosting thread, avoiding any change to
`Game` fields or constructors. Extras call the unchanged native mining method.

Native validation type-checks the real pinned game binary and runs the 13 mining
engine tests embedded in it. A separate syn audit removes only the three planned
HEAD statements and module, then compares every affected file's AST with the
pristine export. Gameplay sessions with prepared assets remain a manual check;
no game assets or upstream source edits are needed for these checks.
