#[cfg(test)]
mod tests{
    use scoped_game::*;
    fn trace()->Trace{std::rc::Rc::new(std::cell::RefCell::new(Vec::new()))}
    #[test]
    fn captured_locals_have_stable_shadow_aware_identities(){
        let mut log=Vec::new();assert_eq!(locals(1,&mut log),10);
        assert_eq!(log,["alpha","beta","tick","inner","tick","after","tick"]);
        let mut log=Vec::new();assert_eq!(inferred(&mut log),3);
    }
    #[test]
    fn return_capture_is_visible_at_all_exits(){
        for (early,expected) in [(true,8),(false,10)]{let mut log=Vec::new();assert_eq!(return_local(early,&mut log),expected);assert_eq!(log,["return"]);}
    }
    #[test]
    fn variable_loads_and_stores_preserve_identity_and_compose(){
        let mut log=Vec::new();assert_eq!(variables(&mut log),81);
        assert_eq!(log,["init-alpha","init-beta","store","load","return"]);
        assert_eq!(parameter(2),16);
    }
    #[test]
    fn slices_limit_counts_and_can_capture_visible_locals(){
        let mut log=Vec::new();assert_eq!(sliced(&mut log),34);
        assert_eq!(log,["begin","capture-first","end"]);
    }
    #[test]
    fn public_bridges_preserve_privacy_borrowing_and_dependency_direction(){
        let mut entity=state::Entity::new();
        assert_eq!(*entity.__horizon_mixins_alpha_get_health(),2);
        entity.__horizon_mixins_alpha_set_health(7);
        *entity.__horizon_mixins_alpha_get_health_mut()+=1;
        assert_eq!(*entity.__horizon_mixins_alpha_inc(2),10);
        assert_eq!(*entity.__horizon_mixins_beta_health(),10);
        assert_eq!(entity.__horizon_mixins_beta_read_trait(),10);
        assert_eq!(entity.__horizon_mixins_alpha_get_name(),"entity");
        let entity=state::Entity::__horizon_mixins_alpha_create(19);
        assert_eq!(*entity.__horizon_mixins_beta_health(),19);
        assert_eq!(*state::pair().__horizon_mixins_beta_tuple_field(),8);
    }
    #[test]
    fn argument_tuples_compose_with_single_argument_hooks_wrappers_and_capture(){
        let trace=trace();let mut evaluations=0;
        assert_eq!(jointly(&mut evaluations,trace.clone()),"sabw26!");assert_eq!(evaluations,2);
        assert_eq!(*trace.borrow(),["arg0","arg1","tuple-alpha","tuple-beta","wrap","original"]);
    }
    #[test]
    fn argument_tuples_preserve_method_and_mutable_argument_reborrows(){
        let trace=trace();let mut evaluations=0;let mut entity=state::Entity::new();
        assert_eq!(method(&mut entity,&mut evaluations,trace.clone()),10);assert_eq!(evaluations,1);
        let mut number=0;assert_eq!(reborrow(&mut number),2);assert_eq!(number,3);
    }
    #[test]
    fn tuples_preserve_owned_and_temporary_destruction(){
        let trace=trace();assert_eq!(owned_drop(trace.clone()),2);
        assert_eq!(*trace.borrow(),["tuple","original","token-drop"]);
        trace.borrow_mut().clear();assert_eq!(temporary(&trace),4);
        assert_eq!(*trace.borrow(),["value","later","tuple","token-drop"]);
    }
}
