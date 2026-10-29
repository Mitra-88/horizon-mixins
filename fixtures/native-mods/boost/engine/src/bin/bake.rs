//! The post-build step: the launcher runs it after the game compiles; horizon-mixins never does.
fn main() {
    println!("baked {}", boost_engine::boost(0));
}
