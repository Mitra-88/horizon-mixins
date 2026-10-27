use std::{cell::RefCell, ops::ControlFlow, rc::Rc};
type Log = Rc<RefCell<Vec<&'static str>>>;
struct Token(Log, &'static str);
impl Drop for Token {
    fn drop(&mut self) {
        self.0.borrow_mut().push(self.1);
    }
}
fn make(log: &Log, name: &'static str) -> Token {
    Token(log.clone(), name)
}
fn call(log: &Log, _arg: &Token) -> Token {
    log.borrow_mut().push("call");
    make(log, "result")
}
fn observed(value: Token) -> Token {
    value.0.borrow_mut().push("after");
    value
}
fn cancel(value: Token) -> ControlFlow<i32, Token> {
    value.0.borrow_mut().push("after");
    ControlFlow::Continue(value)
}
fn original(log: &Log) {
    call(log, &make(log, "arg"));
    log.borrow_mut().push("tail");
}
fn modified(log: &Log) {
    ({
        fn adapter(value: Token) -> Token {
            observed(value)
        }
        adapter
    })(call(log, &make(log, "arg")));
    log.borrow_mut().push("tail");
}
fn cancellable(log: &Log) -> i32 {
    match cancel(call(log, &make(log, "arg"))) {
        ControlFlow::Break(v) => return v,
        ControlFlow::Continue(value) => {
            drop(value);
        }
    };
    log.borrow_mut().push("tail");
    0
}
#[test]
fn drop_order() {
    let logs: Vec<_> = (0..3).map(|_| Rc::new(RefCell::new(Vec::new()))).collect();
    original(&logs[0]);
    modified(&logs[1]);
    cancellable(&logs[2]);
    for log in &logs[1..] {
        let filtered: Vec<_> = log
            .borrow()
            .iter()
            .filter(|s| **s != "after")
            .copied()
            .collect();
        assert_eq!(*logs[0].borrow(), filtered);
    }
    assert_eq!(
        *logs[1].borrow(),
        vec!["call", "after", "result", "arg", "tail"]
    );
}
#[test]
fn expression_once_and_state() {
    let mut calls = 0;
    let v = ({
        fn modify(v: bool) -> bool {
            !v
        }
        modify
    })({
        calls += 1;
        3 > 2
    });
    assert!(!v);
    assert_eq!(calls, 1);
    struct Owner {
        value: i32,
        state: Vec<i32>,
    }
    fn new() -> Owner {
        Owner {
            value: 1,
            state: Default::default(),
        }
    }
    let mut a = new();
    let b = new();
    a.state.push(a.value);
    assert!(b.state.is_empty());
}
