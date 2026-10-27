# Effects and state compilation fixture

This pinned workspace is transformed using `../effects-mods`. Two mods modify
the same expression/call and add independent fields to the same private-layout
struct. Five driver tests cover short-circuit and once-only evaluation,
BEFORE/AFTER cancellation, RETURN, temporary Drop order, typed post-assignment
captures and per-instance/clone/update/serde behavior. The original driver
intentionally expects generated APIs; compile the staged workspace with CLI
`check`. See [contracts](../../docs/effects-and-state.md).
