use super::*;

/// MODIFY_VARIABLE hooks on `amount` in `fixture_game::run`, whose body is `body`: a STORE on its
/// first value and a LOAD on its first read. The helpers log, then add 100 and 1000.
fn fixture(body: &str) -> Fixture {
    let f = Fixture::new();
    fs::remove_dir_all(&f.mods).unwrap();
    fs::create_dir_all(f.mods.join("alpha")).unwrap();
    fs::write(
        f.root.join("engine/src/lib.rs"),
        format!(
            r#"
pub static LOG: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
pub fn mark(what: &'static str) {{ LOG.lock().unwrap().push(what); }}
pub struct Guard(pub &'static str, pub i32);
impl Guard {{ pub fn new(name: &'static str, n: i32) -> Self {{ Guard(name, n) }} }}
impl Drop for Guard {{ fn drop(&mut self) {{ mark(self.0); }} }}
pub fn size(g: &Guard) -> i32 {{ g.1 }}
pub fn sink(n: i32) -> i32 {{ n }}
pub fn run(early: bool) -> Option<i32> {{ {body} }}
"#
        ),
    )
    .unwrap();
    fs::write(f.root.join("driver/src/lib.rs"), "").unwrap();
    fs::write(
        f.mods.join("alpha/horizon-mixin.toml"),
        r#"
[mod]
id="alpha"
version="1"
[[injection]]
id="store"
target="fixture_game::run"
kind="MODIFY_VARIABLE"
variable={name="amount",ordinal=0,at="STORE"}
require=1
ordinal=0
helper="store.rs"
bindings=[{name="value",ty="i32",value="$value"}]
[[injection]]
id="load"
target="fixture_game::run"
kind="MODIFY_VARIABLE"
variable={name="amount",ordinal=0,at="LOAD"}
require=1
ordinal=0
helper="load.rs"
bindings=[{name="value",ty="i32",value="$value"}]
"#,
    )
    .unwrap();
    fs::write(
        f.mods.join("alpha/store.rs"),
        "fn store(value:i32)->i32 { crate::mark(\"store\"); value+100 }",
    )
    .unwrap();
    fs::write(
        f.mods.join("alpha/load.rs"),
        "fn load(value:i32)->i32 { crate::mark(\"load\"); value+1000 }",
    )
    .unwrap();
    f
}

#[test]
fn variable_hooks_leave_closures_question_marks_and_referenced_calls_outside_the_value_alone() {
    // As in Horizon's `Mob::damage_with`: a closure elsewhere in the body (here with its own
    // `return` and its own `let amount`), a `?`, a call beneath a reference in a statement and a
    // reference whose temporary a `let` extends to the end of the function.
    let f = fixture(
        r#"
        let inner = |n: i32| { let amount = n * 10; if n > 0 { return amount; } 0 };
        let extended = &Guard::new("extended", 3);
        let first = Some(size(&Guard::new("argument", 2)))?;
        let amount = first + inner(1);
        if early { return None; }
        mark("body");
        Some(sink(amount + size(extended)) + Some(0).map(|n| n + inner(0)).unwrap())
    "#,
    );
    fs::write(
        f.root.join("driver/src/lib.rs"),
        r#"
#[test]
fn each_hook_runs_once_and_temporaries_keep_their_lifetimes() {
    use fixture_game::{run, LOG};
    // STORE: 2 + 10 + 100; LOAD: 112 + 1000; then + 3. The closure's `amount` is not `amount:0`.
    assert_eq!(run(false), Some(1115));
    assert_eq!(*LOG.lock().unwrap(), ["argument", "store", "body", "load", "extended"]);
    LOG.lock().unwrap().clear();
    assert_eq!(run(true), None);
    assert_eq!(*LOG.lock().unwrap(), ["argument", "store", "extended"]);
}
"#,
    )
    .unwrap();
    f.run("check");
}

#[test]
fn variable_hooks_still_refuse_closures_question_marks_and_referenced_calls_in_the_value() {
    for body in [
        "let amount = (|| sink(1))(); Some(sink(amount))",
        "let amount = Some(sink(1))?; Some(sink(amount))",
        "let amount = *&sink(1); Some(sink(amount))",
        "let amount = size(&Guard::new(\"value\", 1)); Some(sink(amount))",
    ] {
        let f = fixture(body);
        let e = build_plan(&f.root, &f.mods).unwrap_err();
        assert_eq!(e.code, "E_UNSUPPORTED_CONTROL_FLOW", "{body}: {e}");
        if body.contains('&') {
            assert!(e.message.contains("beneath a reference"), "{body}: {e}");
        }
    }
    // An access inside a closure counts, but no hook runs there: a read, and a second store.
    let f = fixture("let amount = sink(1); let f = || sink(amount); Some(f())");
    let e = build_plan(&f.root, &f.mods).unwrap_err();
    assert_eq!(e.code, "E_UNSUPPORTED_LOCAL", "{e}");
    let f = fixture(
        "let mut amount = sink(1); let mut f = || amount = sink(2); f(); Some(sink(amount))",
    );
    f.replace(
        "mods/alpha/horizon-mixin.toml",
        "require=1\nordinal=0\nhelper=\"store.rs\"",
        "require=2\nordinal=1\nhelper=\"store.rs\"",
    );
    let e = build_plan(&f.root, &f.mods).unwrap_err();
    assert_eq!(e.code, "E_UNSUPPORTED_LOCAL", "{e}");
    // What a closure holds is still checked: async stays refused.
    let f = fixture("let f = async || 1; drop(f); let amount = sink(1); Some(sink(amount))");
    let e = build_plan(&f.root, &f.mods).unwrap_err();
    assert_eq!(e.code, "E_UNSUPPORTED_CONTROL_FLOW", "{e}");
    // Hooks that rewrite inside the body keep the check for the whole body.
    let f = fixture("let amount = Some(sink(1))?; let n = (|| 2)(); Some(sink(amount + n))");
    f.append(
        "mods/alpha/horizon-mixin.toml",
        r#"
[[injection]]
id="before"
target="fixture_game::run"
kind="BEFORE_CALL"
selector="sink"
require=2
ordinal=1
helper="before.rs"
"#,
    );
    fs::write(f.mods.join("alpha/before.rs"), "fn before() {}").unwrap();
    let e = build_plan(&f.root, &f.mods).unwrap_err();
    assert_eq!(e.code, "E_UNSUPPORTED_CONTROL_FLOW", "{e}");
}
