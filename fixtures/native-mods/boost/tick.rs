fn tick(game: &mut Game) {
    // Bundle data is staged under <workspace>/native-data/<mod id>/.
    const GREETING: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../native-data/boost/greeting.txt"));
    const BLOB: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../native-data/boost/nested/blob.bin"));
    game.score = boost_engine::boost(game.score);
    game.log.push(String::from_utf8_lossy(GREETING).trim().to_string());
    game.log.push(format!("blob {} bytes", BLOB.len()));
}
