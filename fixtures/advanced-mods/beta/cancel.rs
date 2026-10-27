fn cancel(input: i32, log: &mut Vec<&'static str>) -> std::ops::ControlFlow<i32> {
    log.push("cancel-beta");
    if input == 1 { return std::ops::ControlFlow::Break(11); }
    std::ops::ControlFlow::Continue(())
}
