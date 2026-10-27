#[test]
fn same_crate_private_bridge_and_reborrow() {
    let mut entity = spike_game::Entity::new();
    assert_eq!(entity.tick(), 10);
    assert_eq!(entity.tick(), 28);
}
