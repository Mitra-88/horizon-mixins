fn around(value: String, trace: Trace, original: impl FnOnce(String, Trace) -> String) -> String {
    trace.borrow_mut().push("beta-enter");
    let retained = trace.clone();
    let output = original(value + "b", trace);
    retained.borrow_mut().push("beta-exit");
    output + "B"
}
