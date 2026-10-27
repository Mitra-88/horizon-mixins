use std::{cell::RefCell, rc::Rc};
pub type Trace = Rc<RefCell<Vec<&'static str>>>;
pub fn calculate(input: i32, log: &mut Vec<&'static str>) -> i32 {
    {
        fn first(log: &mut Vec<&'static str>) {
            log.push("head-first");
        }
        first(&mut *log)
    };
    if let ::core::ops::ControlFlow::Break(__horizon_mixins_return) = {
        fn cancel(
            input: i32,
            log: &mut Vec<&'static str>,
        ) -> std::ops::ControlFlow<i32> {
            log.push("cancel-alpha");
            if input == 0 {
                return std::ops::ControlFlow::Break(7);
            }
            std::ops::ControlFlow::Continue(())
        }
        cancel(input, &mut *log)
    } {
        return ({
            fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("return-beta");
                value * 2
            }
            on_return
        })(
            ({
                fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                    log.push("return-alpha");
                    value + 3
                }
                on_return
            })(__horizon_mixins_return, &mut *log),
            &mut *log,
        );
    }
    if let ::core::ops::ControlFlow::Break(__horizon_mixins_return) = {
        fn cancel(
            input: i32,
            log: &mut Vec<&'static str>,
        ) -> std::ops::ControlFlow<i32> {
            log.push("cancel-beta");
            if input == 1 {
                return std::ops::ControlFlow::Break(11);
            }
            std::ops::ControlFlow::Continue(())
        }
        cancel(input, &mut *log)
    } {
        return ({
            fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("return-beta");
                value * 2
            }
            on_return
        })(
            ({
                fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                    log.push("return-alpha");
                    value + 3
                }
                on_return
            })(__horizon_mixins_return, &mut *log),
            &mut *log,
        );
    }
    {
        fn last(log: &mut Vec<&'static str>) {
            log.push("head-last");
        }
        last(&mut *log)
    };
    log.push("body");
    if input < 0 {
        return ({
            fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("return-beta");
                value * 2
            }
            on_return
        })(
            ({
                fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                    log.push("return-alpha");
                    value + 3
                }
                on_return
            })(-input, &mut *log),
            &mut *log,
        );
    }
    ({
        fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
            log.push("return-beta");
            value * 2
        }
        on_return
    })(
        ({
            fn on_return(value: i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("return-alpha");
                value + 3
            }
            on_return
        })(input * 2, &mut *log),
        &mut *log,
    )
}
pub fn exits(input: i32, evaluations: &mut usize) -> i32 {
    *evaluations += 1;
    if input == 0 {
        return ({
            fn modify(value: i32) -> i32 {
                if value == 5 {
                    return 6;
                }
                value + 1
            }
            modify
        })(5);
    }
    ({
        fn modify(value: i32) -> i32 {
            if value == 5 {
                return 6;
            }
            value + 1
        }
        modify
    })(
        match input {
            1 => 6,
            _ => {
                return ({
                    fn modify(value: i32) -> i32 {
                        if value == 5 {
                            return 6;
                        }
                        value + 1
                    }
                    modify
                })(7);
            }
        },
    )
}
pub fn unit(early: bool, log: &mut Vec<&'static str>) {
    log.push("body");
    if early {
        return ({
            fn on_return(value: (), log: &mut Vec<&'static str>) {
                let _ = value;
                log.push("return");
            }
            on_return
        })((), &mut *log);
    }
    log.push("end");
    ({
        fn on_return(value: (), log: &mut Vec<&'static str>) {
            let _ = value;
            log.push("return");
        }
        on_return
    })((), &mut *log)
}
pub fn borrowed(input: &i32) -> &i32 {
    ({
        fn modify(value: &i32) -> &i32 {
            value
        }
        modify
    })(input)
}
pub fn owned(input: String) -> String {
    ({
        fn modify(value: String) -> String {
            value + "!"
        }
        modify
    })(input)
}
struct Guard {
    trace: Trace,
}
impl Drop for Guard {
    fn drop(&mut self) {
        self.trace.borrow_mut().push("local-drop");
    }
}
pub fn drop_return(trace: Trace) -> i32 {
    let _guard = Guard { trace: trace.clone() };
    ({
        fn modify(value: i32, trace: &Trace) -> i32 {
            trace.borrow_mut().push("return");
            value + 1
        }
        modify
    })(41, &trace)
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
    sink(
        ({
            fn modify(value: String, log: &mut Vec<&'static str>) -> String {
                log.push("modify-beta");
                value + "b"
            }
            modify
        })(
            ({
                fn modify(value: String, log: &mut Vec<&'static str>) -> String {
                    log.push("modify-alpha");
                    value + "a"
                }
                modify
            })(make(evaluations, log), &mut *log),
            &mut *log,
        ),
        ({
            fn modify(value: i32) -> i32 {
                value + 10
            }
            modify
        })(next(evaluations, log)),
        log,
    )
}
pub struct Counter {
    total: i32,
}
impl Counter {
    pub fn new() -> Self {
        Self { total: 0 }
    }
    fn apply(&mut self, value: i32) -> i32 {
        self.total += value;
        self.total
    }
    fn apply_traced(&mut self, value: i32, trace: Trace) -> i32 {
        trace.borrow_mut().push("original");
        self.apply(value)
    }
    pub fn wrapped_self(&mut self, value: i32, trace: Trace) -> i32 {
        (|
            __horizon_mixins_arg0: &mut Counter,
            __horizon_mixins_arg1: i32,
            __horizon_mixins_arg2: Trace|
        ({
            fn around(
                counter: &mut Counter,
                value: i32,
                trace: Trace,
                original: impl FnOnce(&mut Counter, i32, Trace) -> i32,
            ) -> i32 {
                trace.borrow_mut().push("alpha-enter");
                let retained = trace.clone();
                let output = original(counter, value + 1, trace);
                retained.borrow_mut().push("alpha-exit");
                output + 100
            }
            around
        })(
            __horizon_mixins_arg0,
            __horizon_mixins_arg1,
            __horizon_mixins_arg2,
            |
                __horizon_mixins_arg0: &mut Counter,
                __horizon_mixins_arg1: i32,
                __horizon_mixins_arg2: Trace|
            ({
                fn around(
                    counter: &mut Counter,
                    value: i32,
                    trace: Trace,
                    original: impl FnOnce(&mut Counter, i32, Trace) -> i32,
                ) -> i32 {
                    trace.borrow_mut().push("beta-enter");
                    let retained = trace.clone();
                    let output = original(counter, value * 2, trace);
                    retained.borrow_mut().push("beta-exit");
                    output + 10
                }
                around
            })(
                __horizon_mixins_arg0,
                __horizon_mixins_arg1,
                __horizon_mixins_arg2,
                |
                    __horizon_mixins_arg0: &mut Counter,
                    __horizon_mixins_arg1: i32,
                    __horizon_mixins_arg2: Trace|
                {
                    __horizon_mixins_arg0
                        .apply_traced(__horizon_mixins_arg1, __horizon_mixins_arg2)
                },
            ),
        ))(&mut *(self), value, trace)
    }
}
pub fn method_argument(
    counter: &mut Counter,
    evaluations: &mut usize,
    log: &mut Vec<&'static str>,
) -> i32 {
    counter
        .apply(
            ({
                fn modify(value: i32) -> i32 {
                    value + 10
                }
                modify
            })(next(evaluations, log)),
        )
}
pub fn constants() -> i32 {
    3
        + ({
            fn modify(value: i32) -> i32 {
                let _ = 3;
                value * 2
            }
            modify
        })(
            ({
                fn modify(value: i32) -> i32 {
                    let _ = 3;
                    value + 10
                }
                modify
            })(3),
        )
        + (({
            fn modify(value: i32) -> i32 {
                value + 1
            }
            modify
        })(-2))
}
pub fn text() -> &'static str {
    ({
        fn modify(value: &'static str) -> &'static str {
            if value == "two words" { "rewritten" } else { value }
        }
        modify
    })("two words")
}
pub fn flag() -> bool {
    ({
        fn modify(value: bool) -> bool {
            !value
        }
        modify
    })(true)
}
pub fn character() -> char {
    ({
        fn modify(value: char) -> char {
            if value == 'x' { 'y' } else { value }
        }
        modify
    })('x')
}
pub fn float() -> f64 {
    ({
        fn modify(value: f64) -> f64 {
            value * 2.0
        }
        modify
    })(1.5f64)
}
pub fn hex() -> u32 {
    ({
        fn modify(value: u32) -> u32 {
            value + 1
        }
        modify
    })(0x10u32)
}
pub fn pattern(value: i32) -> i32 {
    match value {
        3 => 3,
        _ => {
            ({
                fn modify(value: i32) -> i32 {
                    value + 1
                }
                modify
            })(3)
        }
    }
}
pub fn array() -> [i32; 2] {
    [({
        fn modify(value: i32) -> i32 {
            value + 1
        }
        modify
    })(3); 2]
}
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
    (|__horizon_mixins_arg0: String, __horizon_mixins_arg1: Trace| ({
        fn around(
            value: String,
            trace: Trace,
            original: impl FnOnce(String, Trace) -> String,
        ) -> String {
            trace.borrow_mut().push("alpha-enter");
            let retained = trace.clone();
            let output = original(value + "a", trace);
            retained.borrow_mut().push("alpha-exit");
            output + "A"
        }
        around
    })(
        __horizon_mixins_arg0,
        __horizon_mixins_arg1,
        |__horizon_mixins_arg0: String, __horizon_mixins_arg1: Trace| ({
            fn around(
                value: String,
                trace: Trace,
                original: impl FnOnce(String, Trace) -> String,
            ) -> String {
                trace.borrow_mut().push("beta-enter");
                let retained = trace.clone();
                let output = original(value + "b", trace);
                retained.borrow_mut().push("beta-exit");
                output + "B"
            }
            around
        })(
            __horizon_mixins_arg0,
            __horizon_mixins_arg1,
            |__horizon_mixins_arg0: String, __horizon_mixins_arg1: Trace| original_owned(
                __horizon_mixins_arg0,
                __horizon_mixins_arg1,
            ),
        ),
    ))(make_owned(evaluations, &trace), trace)
}
pub fn wrapping_method(counter: &mut Counter, value: i32, trace: Trace) -> i32 {
    (|
        __horizon_mixins_arg0: &mut Counter,
        __horizon_mixins_arg1: i32,
        __horizon_mixins_arg2: Trace|
    ({
        fn around(
            counter: &mut Counter,
            value: i32,
            trace: Trace,
            original: impl FnOnce(&mut Counter, i32, Trace) -> i32,
        ) -> i32 {
            trace.borrow_mut().push("alpha-enter");
            let retained = trace.clone();
            let output = original(counter, value + 1, trace);
            retained.borrow_mut().push("alpha-exit");
            output + 100
        }
        around
    })(
        __horizon_mixins_arg0,
        __horizon_mixins_arg1,
        __horizon_mixins_arg2,
        |
            __horizon_mixins_arg0: &mut Counter,
            __horizon_mixins_arg1: i32,
            __horizon_mixins_arg2: Trace|
        ({
            fn around(
                counter: &mut Counter,
                value: i32,
                trace: Trace,
                original: impl FnOnce(&mut Counter, i32, Trace) -> i32,
            ) -> i32 {
                trace.borrow_mut().push("beta-enter");
                let retained = trace.clone();
                let output = original(counter, value * 2, trace);
                retained.borrow_mut().push("beta-exit");
                output + 10
            }
            around
        })(
            __horizon_mixins_arg0,
            __horizon_mixins_arg1,
            __horizon_mixins_arg2,
            |
                __horizon_mixins_arg0: &mut Counter,
                __horizon_mixins_arg1: i32,
                __horizon_mixins_arg2: Trace|
            {
                __horizon_mixins_arg0
                    .apply_traced(__horizon_mixins_arg1, __horizon_mixins_arg2)
            },
        ),
    ))(&mut *(counter), value, trace)
}
fn never_called() -> i32 {
    panic!("operation should be skipped");
}
pub fn skipping() -> i32 {
    (|| ({
        fn around(original: impl FnOnce() -> i32) -> i32 {
            let _ = original;
            19
        }
        around
    })(|| never_called()))()
}
fn base(value: i32) -> i32 {
    value * 3
}
pub fn combined(log: &mut Vec<&'static str>) -> i32 {
    ({
        fn modify(value: i32) -> i32 {
            value + 1
        }
        modify
    })({
        {
            fn before(log: &mut Vec<&'static str>) {
                log.push("before");
            }
            before(&mut *log)
        };
        (|__horizon_mixins_arg0: i32| ({
            fn around(value: i32, original: impl FnOnce(i32) -> i32) -> i32 {
                original(value * 2) + 5
            }
            around
        })(
            __horizon_mixins_arg0,
            |__horizon_mixins_arg0: i32| base(__horizon_mixins_arg0),
        ))(
            ({
                fn modify(value: i32) -> i32 {
                    value + 4
                }
                modify
            })(
                ({
                    fn modify(value: i32) -> i32 {
                        value + 1
                    }
                    modify
                })(2),
            ),
        )
    })
}
pub fn nested_wrap(value: i32) -> i32 {
    1
        + (|__horizon_mixins_arg0: i32| ({
            fn around(value: i32, original: impl FnOnce(i32) -> i32) -> i32 {
                original(value * 2) + 5
            }
            around
        })(
            __horizon_mixins_arg0,
            |__horizon_mixins_arg0: i32| base(__horizon_mixins_arg0),
        ))(value)
}
