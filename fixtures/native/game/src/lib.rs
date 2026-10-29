//! A game with two named hook points, listed in ../native-hooks.json.
pub mod hooks {
    #[derive(Default, serde::Serialize)]
    pub struct Game {
        pub score: i64,
        pub log: Vec<String>,
    }
    impl Game {
        pub fn tick(&mut self) {
            self.native_tick();
            self.score += 1;
        }
        /// Hook point "tick" v1. Does nothing in the game itself.
        fn native_tick(&mut self) {}
        /// Hook point "frame" v1.
        pub fn native_frame(&mut self) {}
    }
}
