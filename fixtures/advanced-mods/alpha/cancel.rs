fn cancel(input: i32, log: &mut Vec<&'static str>) -> std::ops::ControlFlow<i32> {
    log.push("cancel-alpha");
    if input == 0 { return std::ops::ControlFlow::Break(7); }
    std::ops::ControlFlow::Continue(())
}
