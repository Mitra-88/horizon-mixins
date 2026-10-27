fn before(log: &mut Vec<&'static str>) {
    // This call must never become another mod's selector match.
    let _ = consume(0);
    log.push("alpha-before");
}
