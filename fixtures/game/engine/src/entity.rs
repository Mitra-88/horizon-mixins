pub struct Entity {
    velocity: i32,
    log: Vec<&'static str>,
}
impl Entity {
    pub fn new() -> Self { Self { velocity: 2, log: Vec::new() } }
    pub fn log(&self) -> &[&'static str] { &self.log }
    pub fn tick(&mut self) -> i32 { self.velocity }
}
pub trait Marker { fn marked(&self) -> i32; }
impl Marker for Entity {
    fn marked(&self) -> i32 { self.velocity }
}
pub fn advance(entity: &mut Entity) -> i32 {
    entity.velocity += 1;
    entity.velocity
}
