fn around(counter: &mut Counter, value: i32, trace: Trace, original: impl FnOnce(&mut Counter, i32, Trace) -> i32) -> i32 {
    trace.borrow_mut().push("beta-enter");
    let retained = trace.clone();
    let output = original(counter, value * 2, trace);
    retained.borrow_mut().push("beta-exit");
    output + 10
}
