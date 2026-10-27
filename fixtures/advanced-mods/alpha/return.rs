fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
    log.push("return-alpha"); value + 3
}
