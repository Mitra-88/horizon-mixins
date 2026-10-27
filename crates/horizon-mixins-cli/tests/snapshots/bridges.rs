use crate::Trace;
#[derive(Debug)]
pub struct Entity {
    health: i32,
    name: String,
}
pub trait Value {
    fn value(&self) -> i32;
}
impl Value for Entity {
    fn value(&self) -> i32 {
        self.health
    }
}
impl Entity {
    pub fn new() -> Self {
        Self {
            health: 2,
            name: String::from("entity"),
        }
    }
    fn inc(&mut self, amount: i32) -> &i32 {
        self.health += amount;
        &self.health
    }
    fn create(health: i32) -> Self {
        Self {
            health,
            name: String::from("created"),
        }
    }
    pub(crate) fn add_traced(&mut self, value: i32, trace: Trace) -> i32 {
        trace.borrow_mut().push("original");
        self.health += value;
        self.health
    }
}
pub struct Pair(i32);
pub fn pair() -> Pair {
    Pair(8)
}
impl Entity {
    pub fn __horizon_mixins_alpha_get_health(&self) -> &i32 {
        &self.health
    }
}
impl Entity {
    pub fn __horizon_mixins_alpha_get_health_mut(&mut self) -> &mut i32 {
        &mut self.health
    }
}
impl Entity {
    pub fn __horizon_mixins_alpha_get_name(&self) -> &String {
        &self.name
    }
}
impl Entity {
    pub fn __horizon_mixins_alpha_set_health(&mut self, value: i32) {
        self.health = value;
    }
}
impl Entity {
    pub fn __horizon_mixins_beta_health(&self) -> &i32 {
        &self.health
    }
}
impl Entity {
    pub fn __horizon_mixins_beta_read_trait(&self) -> i32 {
        <Entity as Value>::value(self)
    }
}
impl Entity {
    pub fn __horizon_mixins_alpha_create(health: i32) -> Self {
        <Entity>::create(health)
    }
}
impl Entity {
    pub fn __horizon_mixins_alpha_inc(&mut self, amount: i32) -> &i32 {
        <Entity>::inc(self, amount)
    }
}
impl Pair {
    pub fn __horizon_mixins_beta_tuple_field(&self) -> &i32 {
        &self.0
    }
}
