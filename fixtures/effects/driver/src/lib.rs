#[cfg(test)]
mod tests {
    use effects_game::*;
    fn trace()->Trace { Default::default() }
    #[test] fn expression_once_and_ordered() {
        let t=trace(); assert_eq!(expression(&t),24); assert_eq!(*t.borrow(),vec!["number","expr-alpha","expr-beta"]);
        t.borrow_mut().clear(); assert_eq!(nested(&t),9); assert_eq!(*t.borrow(),vec!["number","after-number"]);
        t.borrow_mut().clear(); assert!(!short(&t,false)); assert!(t.borrow().is_empty()); assert!(!short(&t,true)); assert_eq!(*t.borrow(),vec!["number","boolean"]);
    }
    #[test] fn before_cancels_before_arguments_and_return_modifies_break() {
        let t=trace(); assert_eq!(before(&t,true),42); assert_eq!(*t.borrow(),vec!["before","guard-drop"]);
        t.borrow_mut().clear(); assert_eq!(before(&t,false),3); assert_eq!(*t.borrow(),vec!["before","number","body","guard-drop"]);
    }
    #[test] fn after_call_drop_and_cancel_order() {
        let t=trace(); assert_eq!(discarded(&t,false),5); assert_eq!(*t.borrow(),vec!["receiver","call","after-alpha","after-beta","result-drop","receiver-drop","body"]);
        t.borrow_mut().clear(); assert_eq!(discarded(&t,true),41); assert_eq!(*t.borrow(),vec!["receiver","call","after-alpha","result-drop","receiver-drop"]);
    }
    #[test] fn after_assignment_has_new_binding_and_store_is_complete() {
        let t=trace(); assert_eq!(assigned(&t,false),12); assert_eq!(*t.borrow(),vec!["number","assigned-first","between","number","assigned-second","body"]);
        t.borrow_mut().clear(); assert_eq!(assigned(&t,true),102); assert_eq!(*t.borrow(),vec!["number","assigned-first"]);
        assert_eq!(after_owned(&t),"hello!");
    }
    #[test] fn state_lives_with_instances_and_roundtrips() {
        use state::{Entity,INITIALIZATIONS}; use std::sync::atomic::Ordering;
        let before=INITIALIZATIONS.load(Ordering::SeqCst);
        let mut a=Entity::new(1); let b=Entity::new(5);
        assert_eq!(INITIALIZATIONS.load(Ordering::SeqCst)-before,2);
        a.tick(); a.tick(); assert_eq!(a.__horizon_mixins_alpha_history(),&vec![1,2]); assert!(b.__horizon_mixins_alpha_history().is_empty());
        *a.__horizon_mixins_beta_counter_mut()=99;
        let a=a.with_health(8); assert_eq!(*a.__horizon_mixins_beta_counter(),99); assert_eq!(INITIALIZATIONS.load(Ordering::SeqCst)-before,2);
        let clone=a.clone(); assert_eq!(clone.__horizon_mixins_alpha_history(),a.__horizon_mixins_alpha_history());
        let json=serde_json::to_string(&a).unwrap(); let restored:Entity=serde_json::from_str(&json).unwrap(); assert_eq!(*restored.__horizon_mixins_beta_counter(),99);
        let old:Entity=serde_json::from_str(r#"{"health":10}"#).unwrap(); assert_eq!(*old.__horizon_mixins_beta_counter(),7); assert!(old.__horizon_mixins_alpha_history().is_empty());
    }
}
