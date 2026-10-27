pub fn consume(value: i32) -> i32 { value * 3 }
fn next(evaluations: &mut i32, log: &mut Vec<&'static str>) -> i32 {
    *evaluations += 1;
    log.push("argument");
    *evaluations
}
pub fn compute(evaluations: &mut i32, log: &mut Vec<&'static str>) -> i32 {
    let first = consume(next(evaluations, log));
    let second = consume(next(evaluations, log));
    first + second
}
struct Receiver(i32);
impl Receiver {
    fn take(self, arg: i32) -> i32 { self.0 + arg }
}
fn make_receiver(evaluations: &mut i32) -> Receiver {
    *evaluations += 1;
    Receiver(*evaluations)
}
pub fn method_compute(evaluations: &mut i32, log: &mut Vec<&'static str>) -> i32 {
    make_receiver(evaluations).take(next(evaluations, log))
}
