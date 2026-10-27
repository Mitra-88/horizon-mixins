use std::{cell::RefCell, rc::Rc};
pub type Trace = Rc<RefCell<Vec<&'static str>>>;
pub fn mark(trace: &Trace, event: &'static str) {
    trace.borrow_mut().push(event);
}
pub struct Token {
    trace: Trace,
    event: &'static str,
}
impl Drop for Token {
    fn drop(&mut self) {
        mark(&self.trace, self.event);
    }
}
pub struct Receiver {
    trace: Trace,
}
impl Drop for Receiver {
    fn drop(&mut self) {
        mark(&self.trace, "receiver-drop");
    }
}
impl Receiver {
    pub fn produce(&self) -> Token {
        mark(&self.trace, "call");
        Token {
            trace: self.trace.clone(),
            event: "result-drop",
        }
    }
}
pub fn receiver(trace: &Trace) -> Receiver {
    mark(trace, "receiver");
    Receiver { trace: trace.clone() }
}
pub fn number(trace: &Trace) -> i32 {
    mark(trace, "number");
    2
}
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
pub fn expression(trace: &Trace) -> i32 {
    let value = ({
        fn modify(value: i32, trace: &Trace) -> i32 {
            mark(trace, "expr-beta");
            value * 2
        }
        modify
    })(
        ({
            fn modify(value: i32, trace: &Trace) -> i32 {
                mark(trace, "expr-alpha");
                value + 1
            }
            modify
        })(add(number(trace), 3), trace),
        trace,
    ) * 2;
    value
}
pub fn nested(trace: &Trace) -> i32 {
    add(
        ({
            fn __horizon_mixins_after(
                mut __horizon_mixins_after_value: i32,
                __horizon_mixins_context_1: &Trace,
            ) -> i32 {
                fn after(value: &mut i32, trace: &Trace) {
                    mark(trace, "after-number");
                    *value += 4;
                }
                after(&mut __horizon_mixins_after_value, __horizon_mixins_context_1);
                __horizon_mixins_after_value
            }
            __horizon_mixins_after
        })(number(trace), trace),
        3,
    )
}
pub fn short(trace: &Trace, enabled: bool) -> bool {
    enabled
        && ({
            fn modify(value: bool, trace: &Trace) -> bool {
                mark(trace, "boolean");
                !value
            }
            modify
        })(number(trace) > 0, trace)
}
pub fn before(trace: &Trace, cancel: bool) -> i32 {
    let _guard = Token {
        trace: trace.clone(),
        event: "guard-drop",
    };
    let value = {
        if let ::core::ops::ControlFlow::Break(__horizon_mixins_cancel) = {
            fn before(cancel: bool, trace: &Trace) -> ::core::ops::ControlFlow<i32> {
                mark(trace, "before");
                if cancel {
                    ::core::ops::ControlFlow::Break(41)
                } else {
                    ::core::ops::ControlFlow::Continue(())
                }
            }
            before(cancel, trace)
        } {
            return ({
                fn result(value: i32) -> i32 {
                    value + 1
                }
                result
            })(__horizon_mixins_cancel);
        }
        number(trace)
    };
    mark(trace, "body");
    ({
        fn result(value: i32) -> i32 {
            value + 1
        }
        result
    })(value)
}
pub fn discarded(trace: &Trace, cancel: bool) -> i32 {
    match ({
        fn __horizon_mixins_after(
            __horizon_mixins_after_value: Token,
            __horizon_mixins_context_1: bool,
            __horizon_mixins_context_2: &Trace,
        ) -> ::core::ops::ControlFlow<i32, Token> {
            fn after(
                result: &Token,
                cancel: bool,
                trace: &Trace,
            ) -> ::core::ops::ControlFlow<i32> {
                let _ = result;
                mark(trace, "after-alpha");
                if cancel {
                    ::core::ops::ControlFlow::Break(41)
                } else {
                    ::core::ops::ControlFlow::Continue(())
                }
            }
            if let ::core::ops::ControlFlow::Break(value) = after(
                &__horizon_mixins_after_value,
                __horizon_mixins_context_1,
                __horizon_mixins_context_2,
            ) {
                return ::core::ops::ControlFlow::Break(value);
            }
            ::core::ops::ControlFlow::Continue(__horizon_mixins_after_value)
        }
        __horizon_mixins_after
    })(receiver(trace).produce(), cancel, trace) {
        ::core::ops::ControlFlow::Break(__horizon_mixins_cancel) => {
            return __horizon_mixins_cancel;
        }
        ::core::ops::ControlFlow::Continue(__horizon_mixins_after_next) => {
            match ({
                fn __horizon_mixins_after(
                    __horizon_mixins_after_value: Token,
                    __horizon_mixins_context_0: &Trace,
                ) -> ::core::ops::ControlFlow<i32, Token> {
                    fn after(trace: &Trace) {
                        mark(trace, "after-beta");
                    }
                    after(__horizon_mixins_context_0);
                    ::core::ops::ControlFlow::Continue(__horizon_mixins_after_value)
                }
                __horizon_mixins_after
            })(__horizon_mixins_after_next, trace) {
                ::core::ops::ControlFlow::Break(__horizon_mixins_cancel) => {
                    return __horizon_mixins_cancel;
                }
                ::core::ops::ControlFlow::Continue(__horizon_mixins_after_next) => {
                    let _ = __horizon_mixins_after_next;
                }
            }
        }
    };
    mark(trace, "body");
    5
}
pub fn assigned(trace: &Trace, cancel: bool) -> i32 {
    let mut value: i32 = number(trace);
    if let ::core::ops::ControlFlow::Break(__horizon_mixins_cancel) = ({
        fn after(
            value: &i32,
            cancel: bool,
            trace: &Trace,
        ) -> ::core::ops::ControlFlow<i32> {
            mark(trace, "assigned-first");
            if cancel {
                ::core::ops::ControlFlow::Break(*value + 100)
            } else {
                ::core::ops::ControlFlow::Continue(())
            }
        }
        after
    })(&value, cancel, trace) {
        return __horizon_mixins_cancel;
    }
    mark(trace, "between");
    value = number(trace);
    ({
        fn after(value: &mut i32, trace: &Trace) {
            mark(trace, "assigned-second");
            *value += 10;
        }
        after
    })(&mut value, trace);
    mark(trace, "body");
    value
}
pub fn after_owned(_trace: &Trace) -> String {
    let mut text = String::from("hello");
    ({
        fn after(text: &mut String) {
            text.push_str("!");
        }
        after
    })(&mut text);
    text
}
pub mod state {
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicUsize, Ordering};
    pub static INITIALIZATIONS: AtomicUsize = AtomicUsize::new(0);
    pub fn initial_counter() -> usize {
        INITIALIZATIONS.fetch_add(1, Ordering::SeqCst);
        7
    }
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Entity {
        health: i32,
        #[serde(default = "__horizon_mixins_alpha_history_init")]
        __horizon_mixins_alpha_history: Vec<i32>,
        #[serde(default = "__horizon_mixins_beta_counter_init")]
        __horizon_mixins_beta_counter: usize,
    }
    impl Entity {
        pub fn new(health: i32) -> Self {
            Self {
                health,
                __horizon_mixins_alpha_history: crate::state::__horizon_mixins_alpha_history_init(),
                __horizon_mixins_beta_counter: crate::state::__horizon_mixins_beta_counter_init(),
            }
        }
        pub fn with_health(self, health: i32) -> Self {
            Self { health, ..self }
        }
        pub fn tick(&mut self) {
            {
                fn tick(entity: &mut Entity) {
                    let health = entity.health;
                    entity.__horizon_mixins_alpha_history_mut().push(health);
                }
                tick(self)
            };
            self.health += 1;
        }
        pub fn health(&self) -> i32 {
            self.health
        }
    }
    pub(crate) fn __horizon_mixins_alpha_history_init() -> Vec<i32> {
        ::core::default::Default::default()
    }
    impl Entity {
        pub fn __horizon_mixins_alpha_history(&self) -> &Vec<i32> {
            &self.__horizon_mixins_alpha_history
        }
        pub fn __horizon_mixins_alpha_history_mut(&mut self) -> &mut Vec<i32> {
            &mut self.__horizon_mixins_alpha_history
        }
    }
    pub(crate) fn __horizon_mixins_beta_counter_init() -> usize {
        initial_counter()
    }
    impl Entity {
        pub fn __horizon_mixins_beta_counter(&self) -> &usize {
            &self.__horizon_mixins_beta_counter
        }
        pub fn __horizon_mixins_beta_counter_mut(&mut self) -> &mut usize {
            &mut self.__horizon_mixins_beta_counter
        }
    }
}
