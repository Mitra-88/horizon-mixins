# Security / trust model

Native mod source, build.rs, proc macros and Cargo dependencies can execute code with user privileges. **This is not sandboxed.** Show explicit trust warnings and native-mod opt-in; keep Horizon's WASM mod environment as the safer separated path.

Pin and verify source/dependencies/toolchain, prevent manifest path traversal and symlink escapes, keep original Horizon checkout read-only, isolate build staging and avoid executing arbitrary build commands outside clearly approved Cargo steps. An OS build sandbox is desirable future work but not assumed available.

`syn` cannot resolve Rust methods/types or expand opaque macros. Injection cannot bypass privacy, borrowing, orphan rules or trait resolution. Build artifacts aren't generally portable. Compilation success does not guarantee mod safety, compatibility or gameplay correctness.

Do not redistribute Minecraft binaries, game data, decompiled source or prepared assets. Project license remains for owner selection.
