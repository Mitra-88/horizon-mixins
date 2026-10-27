use std::{cell::RefCell, rc::Rc};
pub mod state;
use state::Entity;
pub type Trace = Rc<RefCell<Vec<&'static str>>>;
fn tick(log: &mut Vec<&'static str>) {
    log.push("tick");
}
pub fn locals(input: i32, log: &mut Vec<&'static str>) -> i32 {
    let mut value: i32 = input;
    {
        ({
            fn capture(value: &mut i32, log: &mut Vec<&'static str>) {
                *value += 2;
                log.push("alpha");
            }
            capture
        })(&mut value, &mut *log);
        ({
            fn capture(value: &mut i32, log: &mut Vec<&'static str>) {
                *value += 3;
                log.push("beta");
            }
            capture
        })(&mut value, &mut *log);
        tick(log);
    };
    {
        let value: i32 = 100;
        {
            ({
                fn value(value: &i32, log: &mut Vec<&'static str>) {
                    assert_eq!(* value, 100);
                    log.push("inner");
                }
                value
            })(&value, &mut *log);
            tick(log);
        };
        let _ = value;
    }
    {
        ({
            fn capture(value: &mut i32, log: &mut Vec<&'static str>) {
                *value += 4;
                log.push("after");
            }
            capture
        })(&mut value, &mut *log);
        tick(log);
    };
    value
}
pub fn inferred(log: &mut Vec<&'static str>) -> i32 {
    let mut n = 1;
    {
        ({
            fn capture(value: &mut i32) {
                *value += 2;
            }
            capture
        })(&mut n);
        tick(log);
    };
    n
}
pub fn return_local(early: bool, log: &mut Vec<&'static str>) -> i32 {
    let extra: i32 = 3;
    if early {
        return ({
            fn modify(value: i32, extra: &i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("return");
                value + *extra
            }
            modify
        })(5, &extra, &mut *log);
    }
    let _ = &log;
    ({
        fn modify(value: i32, extra: &i32, log: &mut Vec<&'static str>) -> i32 {
            log.push("return");
            value + *extra
        }
        modify
    })(7, &extra, &mut *log)
}
pub fn variables(log: &mut Vec<&'static str>) -> i32 {
    let mut health: i32 = ({
        fn modify(value: i32, log: &mut Vec<&'static str>) -> i32 {
            log.push("init-beta");
            value * 2
        }
        modify
    })(
        ({
            fn modify(value: i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("init-alpha");
                value + 10
            }
            modify
        })(1, &mut *log),
        &mut *log,
    );
    health = ({
        fn modify(value: i32, log: &mut Vec<&'static str>) -> i32 {
            log.push("store");
            value + 3
        }
        modify
    })(health + 2, &mut *log);
    ({
        fn modify(value: i32, health: &i32, log: &mut Vec<&'static str>) -> i32 {
            log.push("return");
            value + *health
        }
        modify
    })(
        ({
            fn modify(value: i32, log: &mut Vec<&'static str>) -> i32 {
                log.push("load");
                value * 2
            }
            modify
        })(health, &mut *log),
        &health,
        &mut *log,
    )
}
pub fn parameter(mut count: i32) -> i32 {
    count = ({
        fn modify(value: i32) -> i32 {
            value + 5
        }
        modify
    })(count + 1);
    ({
        fn modify(value: i32) -> i32 {
            value * 2
        }
        modify
    })(count)
}
fn consume(value: i32) -> i32 {
    value * 2
}
fn begin(log: &mut Vec<&'static str>) {
    log.push("begin");
}
fn end(log: &mut Vec<&'static str>) {
    log.push("end");
}
pub fn sliced(log: &mut Vec<&'static str>) -> i32 {
    let first = consume(1);
    begin(log);
    let middle = {
        ({
            fn capture(first: &i32, log: &mut Vec<&'static str>) {
                assert_eq!(* first, 2);
                log.push("capture-first");
            }
            capture
        })(&first, &mut *log);
        consume(
            ({
                fn modify(value: i32) -> i32 {
                    let _ = consume(0);
                    value + 10
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
    };
    end(log);
    let last = consume(3);
    first + middle + last
}
fn make(evaluations: &mut usize, trace: &Trace) -> String {
    *evaluations += 1;
    trace.borrow_mut().push("arg0");
    String::from("s")
}
fn next(evaluations: &mut usize, trace: &Trace) -> i32 {
    *evaluations += 1;
    trace.borrow_mut().push("arg1");
    *evaluations as i32
}
fn combine(value: String, number: i32, trace: Trace) -> String {
    trace.borrow_mut().push("original");
    value + &number.to_string()
}
pub fn jointly(evaluations: &mut usize, trace: Trace) -> String {
    let bonus: i32 = 10;
    (|
        (
            __horizon_mixins_bundle_arg0,
            __horizon_mixins_bundle_arg1,
            __horizon_mixins_bundle_arg2,
        ): (String, i32, Trace),
    | -> String {
        (|
            __horizon_mixins_arg0: String,
            __horizon_mixins_arg1: i32,
            __horizon_mixins_arg2: Trace|
        ({
            fn around(
                value: String,
                number: i32,
                trace: Trace,
                original: impl FnOnce(String, i32, Trace) -> String,
            ) -> String {
                trace.borrow_mut().push("wrap");
                original(value + "w", number, trace) + "!"
            }
            around
        })(
            __horizon_mixins_arg0,
            __horizon_mixins_arg1,
            __horizon_mixins_arg2,
            |
                __horizon_mixins_arg0: String,
                __horizon_mixins_arg1: i32,
                __horizon_mixins_arg2: Trace|
            combine(__horizon_mixins_arg0, __horizon_mixins_arg1, __horizon_mixins_arg2),
        ))(
            __horizon_mixins_bundle_arg0,
            __horizon_mixins_bundle_arg1,
            __horizon_mixins_bundle_arg2,
        )
    })(
        ({
            fn modify(args: (String, i32, Trace)) -> (String, i32, Trace) {
                let (value, number, trace) = args;
                trace.borrow_mut().push("tuple-beta");
                (value + "b", number * 2, trace)
            }
            modify
        })(
            ({
                fn modify(
                    args: (String, i32, Trace),
                    bonus: &i32,
                ) -> (String, i32, Trace) {
                    let (value, number, trace) = args;
                    trace.borrow_mut().push("tuple-alpha");
                    (value + "a", number + *bonus, trace)
                }
                modify
            })(
                (
                    make(evaluations, &trace),
                    ({
                        fn modify(value: i32) -> i32 {
                            value + 1
                        }
                        modify
                    })(next(evaluations, &trace)),
                    trace,
                ),
                &bonus,
            ),
        ),
    )
}
pub fn method(entity: &mut Entity, evaluations: &mut usize, trace: Trace) -> i32 {
    (|
        __horizon_mixins_bundle_receiver: &mut Entity,
        (__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1): (i32, Trace),
    | -> i32 {
        __horizon_mixins_bundle_receiver
            .add_traced(__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1)
    })(
        &mut *(entity),
        ({
            fn modify(args: (i32, Trace)) -> (i32, Trace) {
                let (number, trace) = args;
                (number * 2, trace)
            }
            modify
        })(
            ({
                fn modify(args: (i32, Trace)) -> (i32, Trace) {
                    let (number, trace) = args;
                    (number + 3, trace)
                }
                modify
            })((next(evaluations, &trace), trace)),
        ),
    )
}
fn add_to(pointer: &mut i32, number: i32) -> i32 {
    *pointer += number;
    *pointer
}
pub fn reborrow(pointer: &mut i32) -> i32 {
    let result = (|
        (__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1): (&mut i32, i32),
    | -> i32 {
        add_to(__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1)
    })(
        ({
            fn modify(args: (&mut i32, i32)) -> (&mut i32, i32) {
                (args.0, args.1 + 1)
            }
            modify
        })((pointer, 1)),
    );
    *pointer += 1;
    result
}
struct Token(Trace);
impl Drop for Token {
    fn drop(&mut self) {
        self.0.borrow_mut().push("token-drop");
    }
}
fn consume_token(token: Token, number: i32) -> i32 {
    token.0.borrow_mut().push("original");
    number
}
pub fn owned_drop(trace: Trace) -> i32 {
    (|(__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1): (Token, i32)| -> i32 {
        consume_token(__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1)
    })(
        ({
            fn modify(args: (Token, i32)) -> (Token, i32) {
                args.0.0.borrow_mut().push("tuple");
                (args.0, args.1 + 1)
            }
            modify
        })((Token(trace), 1)),
    )
}
impl Token {
    fn value(&self) -> i32 {
        self.0.borrow_mut().push("value");
        1
    }
}
fn later(trace: &Trace) -> i32 {
    trace.borrow_mut().push("later");
    2
}
fn add(a: i32, b: i32) -> i32 {
    a + b
}
pub fn temporary(trace: &Trace) -> i32 {
    (|(__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1): (i32, i32)| -> i32 {
        add(__horizon_mixins_bundle_arg0, __horizon_mixins_bundle_arg1)
    })(
        ({
            fn modify(args: (i32, i32), trace: &Trace) -> (i32, i32) {
                trace.borrow_mut().push("tuple");
                (args.0 + 1, args.1)
            }
            modify
        })((Token(trace.clone()).value(), later(trace)), trace),
    )
}
