#[cfg(test)]
mod tests {
    use fixture_game::{calls, entity::{advance, Entity, Marker}};
    #[test]
    fn two_independent_mods_access_private_fields_in_priority_order() {
        let mut entity = Entity::new();
        assert_eq!(advance(&mut entity), 8);
        assert_eq!(entity.log(), &["alpha", "beta"]);
        assert_eq!(entity.tick(), 9);
        assert_eq!(entity.marked(), 9);
    }
    #[test]
    fn before_call_keeps_result_and_evaluates_arguments_once() {
        let mut evaluations = 0;
        let mut log = Vec::new();
        assert_eq!(calls::compute(&mut evaluations, &mut log), 9);
        assert_eq!(evaluations, 2);
        assert_eq!(log, ["argument", "alpha-before", "beta-before", "argument"]);
    }
    #[test]
    fn method_receiver_and_argument_evaluate_once() {
        let mut evaluations = 0;
        let mut log = Vec::new();
        assert_eq!(calls::method_compute(&mut evaluations, &mut log), 3);
        assert_eq!(evaluations, 2);
        assert_eq!(log, ["alpha-before", "argument"]);
    }
    #[test]
    fn overwrite_preserves_signature_and_return_type() {
        assert_eq!(fixture_game::overwrite::value(6), 42);
        assert_eq!(fixture_game::inline::leaf::identity(7), 7);
    }
}
