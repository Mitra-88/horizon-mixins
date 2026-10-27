use std::{cell::RefCell,rc::Rc};
pub mod state;
use state::Entity;
pub type Trace=Rc<RefCell<Vec<&'static str>>>;
fn tick(log:&mut Vec<&'static str>){log.push("tick");}
pub fn locals(input:i32,log:&mut Vec<&'static str>)->i32 {
    let mut value:i32=input;
    tick(log);
    {let value:i32=100;tick(log);let _=value;}
    tick(log);
    value
}
pub fn inferred(log:&mut Vec<&'static str>)->i32 {let mut n=1;tick(log);n}
pub fn return_local(early:bool,log:&mut Vec<&'static str>)->i32 {
    let extra:i32=3;
    if early{return 5;}
    let _=&log;
    7
}
pub fn variables(log:&mut Vec<&'static str>)->i32 {
    let mut health:i32=1;
    health=health+2;
    health
}
pub fn parameter(mut count:i32)->i32 {count=count+1;count}
fn consume(value:i32)->i32{value*2}
fn begin(log:&mut Vec<&'static str>){log.push("begin");}
fn end(log:&mut Vec<&'static str>){log.push("end");}
pub fn sliced(log:&mut Vec<&'static str>)->i32 {
    let first=consume(1);
    begin(log);
    let middle=consume(2);
    end(log);
    let last=consume(3);
    first+middle+last
}
fn make(evaluations:&mut usize,trace:&Trace)->String{*evaluations+=1;trace.borrow_mut().push("arg0");String::from("s")}
fn next(evaluations:&mut usize,trace:&Trace)->i32{*evaluations+=1;trace.borrow_mut().push("arg1");*evaluations as i32}
fn combine(value:String,number:i32,trace:Trace)->String{trace.borrow_mut().push("original");value+&number.to_string()}
pub fn jointly(evaluations:&mut usize,trace:Trace)->String{
    let bonus:i32=10;
    combine(make(evaluations,&trace),next(evaluations,&trace),trace)
}
pub fn method(entity:&mut Entity,evaluations:&mut usize,trace:Trace)->i32{
    entity.add_traced(next(evaluations,&trace),trace)
}
fn add_to(pointer:&mut i32,number:i32)->i32{*pointer+=number;*pointer}
pub fn reborrow(pointer:&mut i32)->i32{
    let result=add_to(pointer,1);
    *pointer+=1;
    result
}
struct Token(Trace);
impl Drop for Token{fn drop(&mut self){self.0.borrow_mut().push("token-drop");}}
fn consume_token(token:Token,number:i32)->i32{token.0.borrow_mut().push("original");number}
pub fn owned_drop(trace:Trace)->i32{consume_token(Token(trace),1)}
impl Token{fn value(&self)->i32{self.0.borrow_mut().push("value");1}}
fn later(trace:&Trace)->i32{trace.borrow_mut().push("later");2}
fn add(a:i32,b:i32)->i32{a+b}
pub fn temporary(trace:&Trace)->i32{add(Token(trace.clone()).value(),later(trace))}
