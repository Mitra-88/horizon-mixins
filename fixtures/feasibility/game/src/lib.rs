mod private_game {
    pub struct Entity { velocity: i32 }
    impl Entity {
        pub fn new() -> Self { Self { velocity: 1 } }
        pub fn tick(&mut self) -> i32 {
            // This is the bridge shape the transformer will generate.
            fn injected(entity: &mut Entity) { entity.velocity += 4; }
            injected(&mut *self);
            self.velocity *= 2;
            self.velocity
        }
    }
}
pub use private_game::Entity;
