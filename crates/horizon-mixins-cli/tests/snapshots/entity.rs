pub struct Entity {
    velocity: i32,
    log: Vec<&'static str>,
}
impl Entity {
    pub fn new() -> Self {
        Self {
            velocity: 2,
            log: Vec::new(),
        }
    }
    pub fn log(&self) -> &[&'static str] {
        &self.log
    }
    pub fn tick(&mut self) -> i32 {
        {
            fn tick(entity: &mut Entity) {
                entity.velocity += 1;
            }
            tick(&mut *self)
        };
        self.velocity
    }
}
pub trait Marker {
    fn marked(&self) -> i32;
}
impl Marker for Entity {
    fn marked(&self) -> i32 {
        {
            fn observe(entity: &Entity) {
                let _ = entity.velocity;
            }
            observe(self)
        };
        self.velocity
    }
}
pub fn advance(entity: &mut Entity) -> i32 {
    {
        fn head(entity: &mut Entity) {
            entity.log.push("alpha");
            entity.velocity *= 2;
        }
        head(&mut *entity)
    };
    {
        fn head(entity: &mut Entity) {
            entity.log.push("beta");
            entity.velocity += 3;
        }
        head(&mut *entity)
    };
    entity.velocity += 1;
    entity.velocity
}
