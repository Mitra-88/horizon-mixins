Pinned, dependency-free, edition-2024 two-crate compilation fixture. The driver
asserts the modded behavior, so run it through `horizon-mixins check` with
`fixtures/mods`, rather than running its untransformed tests directly.
Toolchain: repository rust-toolchain.toml; dependencies: committed Cargo.lock.
