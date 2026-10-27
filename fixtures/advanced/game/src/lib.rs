use std::{cell::RefCell, rc::Rc};
pub type Trace = Rc<RefCell<Vec<&'static str>>>;
pub fn calculate(input: i32, log: &mut Vec<&'static str>) -> i32 {
    log.push("body");
    if input < 0 { return -input; }
    input * 2
}
pub fn exits(input: i32, evaluations: &mut usize) -> i32 {
    *evaluations += 1;
    if input == 0 { return 5; }
    match input { 1 => 6, _ => { return 7; } }
}
pub fn unit(early: bool, log: &mut Vec<&'static str>) {
    log.push("body");
    if early { return; }
    log.push("end");
}
pub fn borrowed(input: &i32) -> &i32 { input }
pub fn owned(input: String) -> String { input }
struct Guard { trace: Trace }
impl Drop for Guard { fn drop(&mut self) { self.trace.borrow_mut().push("local-drop"); } }
pub fn drop_return(trace: Trace) -> i32 {
    let _guard = Guard { trace: trace.clone() };
    41
}
fn make(evaluations: &mut usize, log: &mut Vec<&'static str>) -> String {
    *evaluations += 1;
    log.push("arg0");
    String::from("s")
}
fn next(evaluations: &mut usize, log: &mut Vec<&'static str>) -> i32 {
    *evaluations += 1;
    log.push("arg1");
    *evaluations as i32
}
fn sink(value: String, number: i32, log: &mut Vec<&'static str>) -> String {
    log.push("sink");
    value + &number.to_string()
}
pub fn arguments(evaluations: &mut usize, log: &mut Vec<&'static str>) -> String {
    sink(make(evaluations, log), next(evaluations, log), log)
}
pub struct Counter { total: i32 }
impl Counter {
    pub fn new() -> Self { Self { total: 0 } }
    fn apply(&mut self, value: i32) -> i32 { self.total += value; self.total }
    fn apply_traced(&mut self, value: i32, trace: Trace) -> i32 {
        trace.borrow_mut().push("original");
        self.apply(value)
    }
    pub fn wrapped_self(&mut self, value: i32, trace: Trace) -> i32 {
        self.apply_traced(value, trace)
    }
}
pub fn method_argument(counter: &mut Counter, evaluations: &mut usize, log: &mut Vec<&'static str>) -> i32 {
    counter.apply(next(evaluations, log))
}
pub fn constants() -> i32 { 3 + 3 + (-2) }
pub fn text() -> &'static str { "two words" }
pub fn flag() -> bool { true }
pub fn character() -> char { 'x' }
pub fn float() -> f64 { 1.5f64 }
pub fn hex() -> u32 { 0x10u32 }
pub fn pattern(value: i32) -> i32 { match value { 3 => 3, _ => 3 } }
pub fn array() -> [i32; 2] { [3; 2] }
fn make_owned(evaluations: &mut usize, trace: &Trace) -> String {
    *evaluations += 1;
    trace.borrow_mut().push("argument");
    String::from("s")
}
fn original_owned(value: String, trace: Trace) -> String {
    trace.borrow_mut().push("original");
    value + "o"
}
pub fn wrapping(evaluations: &mut usize, trace: Trace) -> String {
    original_owned(make_owned(evaluations, &trace), trace)
}
pub fn wrapping_method(counter: &mut Counter, value: i32, trace: Trace) -> i32 {
    counter.apply_traced(value, trace)
}
fn never_called() -> i32 { panic!("operation should be skipped"); }
pub fn skipping() -> i32 { never_called() }
fn base(value: i32) -> i32 { value * 3 }
pub fn combined(log: &mut Vec<&'static str>) -> i32 { base(2) }
pub fn nested_wrap(value: i32) -> i32 { 1 + base(value) }
