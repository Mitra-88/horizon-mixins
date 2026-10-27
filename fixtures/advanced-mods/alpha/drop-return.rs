fn modify(value: i32, trace: &Trace) -> i32 { trace.borrow_mut().push("return"); value + 1 }
