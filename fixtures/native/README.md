Fixture for horizon-mixins 0.6.0: a named-hook catalog (`native-hooks.json`) and a lock file
that the `boost` mod's own crate (`fixtures/native-mods/boost/engine`) resolves `serde_json` from.
The driver locks `hashbrown` 0.15 without its default features, as Horizon does, so a bundle crate
asking for the defaults needs only new dependency edges between locked packages.
