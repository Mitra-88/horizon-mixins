use std::{cell::RefCell, rc::Rc};
pub type Trace = Rc<RefCell<Vec<&'static str>>>;
pub fn mark(trace: &Trace, event: &'static str) { trace.borrow_mut().push(event); }
pub struct Token { trace: Trace, event: &'static str }
impl Drop for Token { fn drop(&mut self) { mark(&self.trace, self.event); } }
pub struct Receiver { trace: Trace }
impl Drop for Receiver { fn drop(&mut self) { mark(&self.trace, "receiver-drop"); } }
impl Receiver { pub fn produce(&self) -> Token { mark(&self.trace,"call"); Token { trace: self.trace.clone(), event:"result-drop" } } }
pub fn receiver(trace:&Trace) -> Receiver { mark(trace,"receiver"); Receiver {trace: trace.clone()} }
pub fn number(trace:&Trace) -> i32 { mark(trace,"number"); 2 }
pub fn add(a:i32,b:i32)->i32 { a+b }
pub fn expression(trace:&Trace)->i32 { let value = add(number(trace),3)*2; value }
pub fn nested(trace:&Trace)->i32 { add(number(trace),3) }
pub fn short(trace:&Trace, enabled:bool)->bool { enabled && number(trace)>0 }
pub fn before(trace:&Trace,cancel:bool)->i32 { let _guard = Token { trace:trace.clone(),event:"guard-drop" }; let value=number(trace); mark(trace,"body"); value }
pub fn discarded(trace:&Trace,cancel:bool)->i32 { receiver(trace).produce(); mark(trace,"body"); 5 }
pub fn assigned(trace:&Trace,cancel:bool)->i32 { let mut value: i32=number(trace); mark(trace,"between"); value=number(trace); mark(trace,"body"); value }
pub fn after_owned(_trace:&Trace)->String { let mut text=String::from("hello"); text }
pub mod state {
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicUsize,Ordering};
    pub static INITIALIZATIONS:AtomicUsize=AtomicUsize::new(0);
    pub fn initial_counter()->usize { INITIALIZATIONS.fetch_add(1,Ordering::SeqCst); 7 }
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Entity { health:i32 }
    impl Entity {
        pub fn new(health:i32)->Self { Self {health} }
        pub fn with_health(self,health:i32)->Self { Self {health,..self} }
        pub fn tick(&mut self) { self.health+=1; }
        pub fn health(&self)->i32 {self.health}
    }
}
