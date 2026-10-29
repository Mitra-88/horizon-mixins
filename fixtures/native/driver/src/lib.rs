#[cfg(test)]
mod tests {
    use native_game::hooks::Game;
    #[test]
    fn named_hook_calls_the_bundle_crate_and_reads_bundle_data() {
        let mut game = Game::default();
        game.tick();
        // The helper adds the crate's bonus (from the crate's own data/table.json), then the tick adds 1.
        assert_eq!(game.score, 41);
        assert_eq!(game.log, ["hello from boost", "blob 4 bytes"]);
        assert_eq!(serde_json::to_value(&game).unwrap()["score"], 41);
    }
}
