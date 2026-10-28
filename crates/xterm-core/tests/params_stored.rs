// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Source Params.ts storage plus SixelHandler.ts58 reads beyond logical length.
use xterm_core::parser::params::Params;

#[test]
fn allocated_slots_are_zero_even_before_the_first_parameter() {
    let params = Params::with_capacities(2, 0).unwrap();
    assert!(params.is_empty());
    assert_eq!(params.stored_value(0), Some(0));
    assert_eq!(params.stored_value(1), Some(0));
    assert_eq!(params.stored_value(2), None);
    assert_eq!(params.stored_value(usize::MAX), None);
    assert_eq!(Params::with_capacities(0, 0).unwrap().stored_value(0), None);
}

#[test]
fn sixel_transparency_observes_stale_second_slot_after_zdm_reset() {
    let mut params = Params::default();
    params.add_param(0).unwrap();
    params.add_param(1).unwrap();
    params.reset_zdm().unwrap();
    assert_eq!(params.values(), [0]);
    // The source tests the backing slot, not `length`, for this exact value.
    assert_eq!(params.stored_value(1), Some(1));
    assert_eq!(params.values().get(1), None);
    let snapshot = params.clone();
    params.add_param(0).unwrap();
    assert_eq!(params.stored_value(1), Some(0));
    assert_eq!(snapshot.stored_value(1), Some(1));
    params.reset();
    assert_eq!(params.stored_value(1), Some(0));
}

#[test]
fn reset_preserves_nontransparent_sentinels_and_clone_storage() {
    let mut params = Params::with_capacities(3, 1).unwrap();
    params.add_param(7).unwrap();
    params.add_param(-1).unwrap();
    params.reset();
    assert!(params.values().is_empty());
    assert_eq!(params.stored_value(0), Some(7));
    assert_eq!(params.stored_value(1), Some(-1));
    assert_eq!(params.stored_value(2), Some(0));
    let clone = params.clone();
    params.reset_zdm().unwrap();
    assert_eq!(params.stored_value(0), Some(0));
    assert_eq!(clone.stored_value(0), Some(7));
}
