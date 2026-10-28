use super::*;
use syn::visit_mut::VisitMut;

fn fixture(body: &str) -> Fixture {
    let f = Fixture::new();
    fs::remove_dir_all(&f.mods).unwrap();
    fs::create_dir_all(f.mods.join("alpha")).unwrap();
    fs::write(
        f.root.join("engine/src/lib.rs"),
        format!("pub fn sink(n:i32)->i32 {{ n }} pub fn run()->i32 {{ {body} }}"),
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
id="return"
target="fixture_game::run"
kind="RETURN"
helper="return.rs"
bindings=[{name="value",ty="i32",value="$value"}]
"#,
    )
    .unwrap();
    fs::write(
        f.mods.join("alpha/return.rs"),
        "fn modify(value:i32)->i32 { value+1 }",
    )
    .unwrap();
    f
}
fn reject(f: &Fixture, code: &str) {
    let e = build_plan(&f.root, &f.mods).unwrap_err();
    assert_eq!(e.code, code, "{e}");
    assert!(e.file.is_some());
    assert!(!f.out.exists());
}
#[test]
fn all_hook_families_compile_with_preserved_standard_macros() {
    for (workspace, mods) in [
        ("game", "mods"),
        ("advanced", "advanced-mods"),
        ("scoped", "scoped-mods"),
        ("effects", "effects-mods"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("original");
        let bundles = temp.path().join("mods");
        copy(&repo().join("fixtures").join(workspace), &root);
        copy(&repo().join("fixtures").join(mods), &bundles);
        let f = Fixture {
            out: temp.path().join("stage"),
            _temp: temp,
            root,
            mods: bundles,
        };
        struct Add;
        impl VisitMut for Add {
            fn visit_block_mut(&mut self, b: &mut syn::Block) {
                // Add no local names or selector matches. Existing captures, counts,
                // return values, cancellation and Drop assertions must all still pass.
                let extra: syn::Block = syn::parse_quote!({
                    let _ = ::std::format!("macro {named:?}", named = ::std::vec!['q'; usize::MIN]);
                    let _ = ::std::matches!('q', macro_probe if macro_probe == 'q');
                });
                b.stmts.splice(0..0, extra.stmts);
            }
        }
        for (path, bytes) in source::files(&f.root).unwrap() {
            if !path.ends_with(".rs") {
                continue;
            }
            let mut file = syn::parse_file(std::str::from_utf8(&bytes).unwrap()).unwrap();
            Add.visit_file_mut(&mut file);
            fs::write(f.root.join(path), horizon_mixins_core::index::tokens(&file)).unwrap();
        }
        let before = source::files(&f.root).unwrap();
        f.run("check");
        let staged = source::files(&f.out).unwrap();
        let times = mtimes(&f.out);
        assert_eq!(f.run("check")["stage_changed"], false);
        assert_eq!(staged, source::files(&f.out).unwrap());
        assert_eq!(times, mtimes(&f.out));
        assert_eq!(before, source::files(&f.root).unwrap());
    }
}
#[test]
fn parsed_forms_keep_argument_order_counts_and_return_composition() {
    let f = fixture(
        r#"
        fn bump(n:&mut i32)->i32 { *n+=1; *n }
        let mut count=0;
        let list=vec![bump(&mut count),bump(&mut count),];
        let repeat=std::vec![bump(&mut count);9];
        let text=format!("{named}",named=std::format!("{}",bump(&mut count)));
        let yes=matches!(Some(bump(&mut count)), | Some(n) if { bump(&mut count); n==5 },);
        let referenced=format!("{:?}",&bump(&mut count));
        let log=std::cell::RefCell::new(Vec::new());
        let owned=format!("{}", token(&log));
        if list!=[1,2] || repeat!=[3;9] || text!="4" || !yes || referenced!="7"
            || owned!="t" || log.borrow().as_slice()!=["create","format","drop"] { return -999; }
        sink(count)
    "#,
    );
    // Ordinary nested function items remain outside the supported rewrite subset.
    f.replace(
        "engine/src/lib.rs",
        "pub fn sink(n:i32)",
        "fn bump(n:&mut i32)->i32 { *n+=1; *n } pub fn sink(n:i32)",
    );
    f.replace(
        "engine/src/lib.rs",
        "        fn bump(n:&mut i32)->i32 { *n+=1; *n }",
        "",
    );
    f.append("engine/src/lib.rs", r#"
struct Token<'a>(&'a std::cell::RefCell<Vec<&'static str>>);
fn token<'a>(log:&'a std::cell::RefCell<Vec<&'static str>>)->Token<'a> { log.borrow_mut().push("create"); Token(log) }
impl std::fmt::Display for Token<'_> {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { self.0.borrow_mut().push("format"); f.write_str("t") }
}
impl Drop for Token<'_> { fn drop(&mut self) { self.0.borrow_mut().push("drop"); } }
"#);
    f.append(
        "mods/alpha/horizon-mixin.toml",
        r#"
[[injection]]
id="before"
target="fixture_game::run"
kind="BEFORE_CALL"
selector="sink"
require=1
ordinal=0
helper="before.rs"
"#,
    );
    fs::write(f.mods.join("alpha/before.rs"), "fn before() {}").unwrap();
    fs::write(
        f.root.join("driver/src/lib.rs"),
        "#[test] fn macros_evaluate_once() { assert_eq!(fixture_game::run(),8); }",
    )
    .unwrap();
    struct Macros(Vec<String>);
    impl<'a> syn::visit::Visit<'a> for Macros {
        fn visit_macro(&mut self, m: &'a syn::Macro) {
            self.0.push(horizon_mixins_core::index::tokens(m));
        }
    }
    let macros = |path: PathBuf| {
        let file = syn::parse_file(&fs::read_to_string(path).unwrap()).unwrap();
        let mut macros = Macros(Vec::new());
        syn::visit::Visit::visit_file(&mut macros, &file);
        macros.0
    };
    let before = macros(f.root.join("engine/src/lib.rs"));
    f.run("check");
    assert_eq!(before, macros(f.out.join("engine/src/lib.rs")));
}
#[test]
fn parsed_macros_reject_hidden_exits_unknown_expansions_and_bad_syntax() {
    for body in [
        "let _=vec![return 7]; sink(1)",
        "let _=vec![Some(1)?]; sink(1)",
        "let _=format!(\"{}\", { return 7; 1 }); sink(1)",
        "let _=format!(\"{named}\",named=Some(1)?); sink(1)",
        "let _=matches!(1, n if { return 7; n==1 }); sink(1)",
        "loop { let _=vec![break]; } sink(1)",
        "loop { let _=format!(\"{}\",continue); } sink(1)",
        "let _=vec![unknown!()]; sink(1)",
        "let _=matches!(1, pattern!()); sink(1)",
        "let _=format!(concat!(\"{}\"),1); sink(1)",
        "let _=vec![1;2;3]; sink(1)",
        "let _=matches!(1,1 garbage); sink(1)",
        "let _=vec![#[cfg(any())] 1]; sink(1)",
    ] {
        let f = fixture(body);
        let e = build_plan(&f.root, &f.mods).unwrap_err();
        assert!(
            matches!(e.code.as_str(), "E_UNSUPPORTED_MACRO" | "E_UNSUPPORTED_CFG"),
            "{body}: {e}"
        );
    }
}
#[test]
fn hidden_selector_matches_fail_instead_of_undercounting() {
    for (declaration, body) in [
        (
            "kind='BEFORE_CALL'\nselector='sink'",
            "let _=vec![sink(1)]; sink(1)",
        ),
        (
            "kind='MODIFY_CONSTANT'\nselector='123'",
            "let _=vec![123]; sink(123)",
        ),
        (
            "kind='MODIFY_EXPRESSION_VALUE'\nselector='123 + 1'",
            "let _=format!(\"{}\",123+1); sink(123+1)",
        ),
        (
            "kind='BEFORE_CALL'\nselector='other'\nslice={from={selector='sink',require=1,ordinal=0}}",
            "let _=vec![sink(1)]; sink(1)",
        ),
    ] {
        let f = fixture(body);
        f.append("mods/alpha/horizon-mixin.toml", &format!("\n[[injection]]\nid='site'\ntarget='fixture_game::run'\n{declaration}\nrequire=1\nordinal=0\nhelper='before.rs'\n"));
        fs::write(f.mods.join("alpha/before.rs"), "fn before() {}").unwrap();
        reject(&f, "E_UNSUPPORTED_MACRO_SITE");
    }
}
#[test]
fn variable_modifiers_reject_hidden_reads_and_implicit_format_captures() {
    for mac in [
        "vec![count]",
        "format!(\"{count}\")",
        "format!(\"{value:count$}\",value=1)",
        "format!(\"{value:0count$}\",value=1)",
    ] {
        let f = fixture(&format!("let count=1; let _={mac}; sink(count)"));
        f.append(
            "mods/alpha/horizon-mixin.toml",
            r#"
[[injection]]
id="variable"
target="fixture_game::run"
kind="MODIFY_VARIABLE"
variable={name="count",ordinal=0,at="LOAD"}
require=1
ordinal=0
helper="return.rs"
bindings=[{name="value",ty="i32",value="$value"}]
"#,
        );
        reject(&f, "E_UNSUPPORTED_MACRO_SITE");
        f.replace("engine/src/lib.rs", "let count=1", "let r#count=1");
        f.replace("engine/src/lib.rs", "sink(count)", "sink(r#count)");
        f.replace(
            "mods/alpha/horizon-mixin.toml",
            "name=\"count\"",
            "name=\"r#count\"",
        );
        reject(&f, "E_UNSUPPORTED_MACRO_SITE");
    }
}
#[test]
fn macro_identity_checks_reject_shadowing_but_allow_absolute_standard_paths() {
    for declaration in [
        "macro_rules! vec { ($($t:tt)*) => { return 7 }; }",
        "use other::vec;",
        "use other::*;",
    ] {
        let f = fixture("let _=vec![1]; sink(1)");
        f.append("engine/src/lib.rs", declaration);
        reject(&f, "E_UNSUPPORTED_MACRO");
    }
    let f = fixture("let _=::std::vec![1]; sink(1)");
    f.append(
        "engine/src/lib.rs",
        "macro_rules! vec { ($($t:tt)*) => { return 7 }; }",
    );
    f.run("check");
    let f = fixture("let _=std::vec![1]; sink(1)");
    f.append("engine/src/lib.rs", "use other as std;");
    reject(&f, "E_UNSUPPORTED_MACRO");
    let f = fixture("let _=::std::vec![1]; sink(1)");
    fs::write(
        f.root.join("driver/Cargo.toml"),
        "[package]\nname='fixture-driver'\nversion='0.1.0'\nedition='2024'\n",
    )
    .unwrap();
    f.append(
        "engine/Cargo.toml",
        "\n[dependencies]\nstd={package='fixture-driver',path='../driver'}\n",
    );
    reject(&f, "E_UNSUPPORTED_MACRO");
}

#[test]
fn return_hooks_wrap_macro_tail_values() {
    for (body, ty, helper, expected) in [
        (
            "format!(\"a\")",
            "String",
            "mut value:String)->String { value.push('!'); value }",
            "\"a!\"",
        ),
        (
            "vec![1]",
            "Vec<i32>",
            "mut value:Vec<i32>)->Vec<i32> { value.push(2); value }",
            "vec![1,2]",
        ),
        (
            "matches!(1,1)",
            "bool",
            "value:bool)->bool { !value }",
            "false",
        ),
    ] {
        let f = fixture(body);
        f.replace(
            "engine/src/lib.rs",
            "pub fn run()->i32",
            &format!("pub fn run()->{ty}"),
        );
        f.replace(
            "mods/alpha/horizon-mixin.toml",
            "ty=\"i32\"",
            &format!("ty=\"{ty}\""),
        );
        fs::write(
            f.mods.join("alpha/return.rs"),
            format!("fn modify({helper}"),
        )
        .unwrap();
        fs::write(
            f.root.join("driver/src/lib.rs"),
            format!("#[test] fn tail_value() {{ assert_eq!(fixture_game::run(),{expected}); }}"),
        )
        .unwrap();
        f.run("check");
    }
}

#[test]
fn hidden_state_constructor_literals_fail_during_planning() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("original");
    let mods = temp.path().join("mods");
    copy(&repo().join("fixtures/effects"), &root);
    copy(&repo().join("fixtures/effects-mods"), &mods);
    let f = Fixture {
        out: temp.path().join("stage"),
        _temp: temp,
        root,
        mods,
    };
    f.replace(
        "game/src/lib.rs",
        "Self {health}",
        "vec![Self {health}].pop().unwrap()",
    );
    reject(&f, "E_UNSUPPORTED_MACRO_SITE");
}
