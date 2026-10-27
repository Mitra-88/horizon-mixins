#[cfg(test)]
mod tests {
    use advanced_game::*;
    fn trace() -> Trace { std::rc::Rc::new(std::cell::RefCell::new(Vec::new())) }
    #[test]
    fn cancellation_order_and_return_composition() {
        for (input, expected, log_expected) in [
            (0, 20, vec!["head-first", "cancel-alpha", "return-alpha", "return-beta"]),
            (1, 28, vec!["head-first", "cancel-alpha", "cancel-beta", "return-alpha", "return-beta"]),
            (2, 14, vec!["head-first", "cancel-alpha", "cancel-beta", "head-last", "body", "return-alpha", "return-beta"]),
            (-2, 10, vec!["head-first", "cancel-alpha", "cancel-beta", "head-last", "body", "return-alpha", "return-beta"]),
        ] {
            let mut log = Vec::new();
            assert_eq!(calculate(input, &mut log), expected);
            assert_eq!(log, log_expected);
        }
    }
    #[test]
    fn explicit_implicit_and_unit_returns_execute_once() {
        for (input, result) in [(0, 6), (1, 7), (2, 8)] {
            let mut evaluations = 0;
            assert_eq!(exits(input, &mut evaluations), result);
            assert_eq!(evaluations, 1);
        }
        for early in [true, false] {
            let mut log = Vec::new(); unit(early, &mut log);
            assert_eq!(log, if early { vec!["body", "return"] } else { vec!["body", "end", "return"] });
        }
    }
    #[test]
    fn return_modifiers_preserve_ownership_borrows_and_local_drop_order() {
        let input = 17;
        assert!(std::ptr::eq(borrowed(&input), &input));
        assert_eq!(owned(String::from("owned")), "owned!");
        let log = trace();
        assert_eq!(drop_return(log.clone()), 42);
        assert_eq!(*log.borrow(), ["return", "local-drop"]);
    }
    #[test]
    fn argument_modifiers_chain_owned_values_and_preserve_evaluation_order() {
        let mut evaluations = 0; let mut log = Vec::new();
        assert_eq!(arguments(&mut evaluations, &mut log), "sab12");
        assert_eq!(evaluations, 2);
        assert_eq!(log, ["arg0", "modify-alpha", "modify-beta", "arg1", "sink"]);
        let mut counter = Counter::new(); let mut evaluations = 0; let mut log = Vec::new();
        assert_eq!(method_argument(&mut counter, &mut evaluations, &mut log), 11);
        assert_eq!(evaluations, 1);
    }
    #[test]
    fn constant_modifiers_preserve_literal_identity_and_skip_patterns() {
        assert_eq!(constants(), 28);
        assert_eq!(text(), "rewritten");
        assert!(!flag()); assert_eq!(character(), 'y');
        assert_eq!(float(), 3.0); assert_eq!(hex(), 17);
        assert_eq!((pattern(3), pattern(0)), (3, 4));
        assert_eq!(array(), [4, 4]);
    }
    #[test]
    fn owned_operation_wrappers_chain_and_evaluate_arguments_once() {
        let log = trace(); let mut evaluations = 0;
        assert_eq!(wrapping(&mut evaluations, log.clone()), "saboBA");
        assert_eq!(evaluations, 1);
        assert_eq!(*log.borrow(), ["argument", "alpha-enter", "beta-enter", "original", "beta-exit", "alpha-exit"]);
    }
    #[test]
    fn method_wrappers_reborrow_private_receivers_including_self() {
        for use_self in [false, true] {
            let log = trace(); let mut counter = Counter::new();
            let result = if use_self { counter.wrapped_self(2, log.clone()) } else { wrapping_method(&mut counter, 2, log.clone()) };
            assert_eq!(result, 116);
            assert_eq!(*log.borrow(), ["alpha-enter", "beta-enter", "original", "beta-exit", "alpha-exit"]);
        }
    }
    #[test]
    fn wrappers_can_skip_and_all_modifier_families_compose() {
        assert_eq!(skipping(), 19);
        let mut log = Vec::new();
        assert_eq!(combined(&mut log), 48);
        assert_eq!(log, ["before"]);
        assert_eq!(nested_wrap(2), 18);
    }
}
