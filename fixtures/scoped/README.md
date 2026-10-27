# Scoped injection fixture

Pinned dependency-free two-crate workspace for lexical captures, slices,
GET/GET_MUT/SET/INVOKE bridges, MODIFY_ARGS and MODIFY_VARIABLE. Two independent
bundles in `../scoped-mods` declare 25 injections and nine bridges. Eight staged
runtime tests check values, order, ownership and public bridge calls.

From the repository root:

```sh
cargo run --locked -p horizon-mixins-cli -- check \
  --workspace fixtures/scoped --mods fixtures/scoped-mods \
  --out target/scoped-game --trust-native-mods
```

This workspace tests generated behavior and is intended to run after staging.
Engine integration tests add negative planning/Cargo cases and source/plan
snapshots. See [the API guide](../../docs/scopes-and-bridges.md).
