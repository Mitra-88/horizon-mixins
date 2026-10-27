fn around(value: String, trace: Trace, original: impl FnOnce(String, Trace) -> String) -> String {
    trace.borrow_mut().push("alpha-enter");
    let retained = trace.clone();
    let output = original(value + "a", trace);
    retained.borrow_mut().push("alpha-exit");
    output + "A"
}
