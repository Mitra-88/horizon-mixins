use crate::Trace;
#[derive(Debug)]
pub struct Entity{health:i32,name:String}
pub trait Value{fn value(&self)->i32;}
impl Value for Entity{fn value(&self)->i32{self.health}}
impl Entity{
    pub fn new()->Self{Self{health:2,name:String::from("entity")}}
    fn inc(&mut self,amount:i32)->&i32{self.health+=amount;&self.health}
    fn create(health:i32)->Self{Self{health,name:String::from("created")}}
    pub(crate) fn add_traced(&mut self,value:i32,trace:Trace)->i32{
        trace.borrow_mut().push("original");self.health+=value;self.health
    }
}
pub struct Pair(i32);
pub fn pair()->Pair{Pair(8)}
