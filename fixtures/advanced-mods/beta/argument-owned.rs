fn modify(value: String, log: &mut Vec<&'static str>) -> String {
    log.push("modify-beta"); value + "b"
}
