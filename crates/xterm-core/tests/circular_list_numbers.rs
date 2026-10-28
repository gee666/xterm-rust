//! Nonintegral/NaN array properties are valid source Number-index operations.
use xterm_core::circular_list::CircularList;

#[test]
fn fraction_and_subnormal_properties_survive_ring_advances() {
    let list = CircularList::new(5).unwrap();
    let tiny = f64::from_bits(1);
    list.set_number(0.5, Some(11)).unwrap();
    list.set_number(-0.5, Some(12)).unwrap();
    list.set_number(tiny, Some(13)).unwrap();
    assert_eq!(list.get_number(0.5), Some(11));
    assert_eq!(list.get_number(-0.5), Some(12));
    assert_eq!(list.get_number(tiny), Some(13));
    assert_eq!(list.len(), 0);
    for value in 1..=6 {
        list.push(value).unwrap();
    }
    assert_eq!(list.get_number(-0.5), Some(11));
    assert_eq!(list.get_number(0.5), None);
    assert_eq!(list.get_number(tiny), Some(2));
    for value in 7..=10 {
        list.push(value).unwrap();
    }
    assert_eq!(list.get_number(tiny), Some(13));
}

#[test]
fn nan_and_either_infinity_use_the_same_array_property() {
    let list = CircularList::new(5).unwrap();
    list.set_number(f64::NAN, Some(77)).unwrap();
    for index in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(0x7ff8_0000_0000_0001),
    ] {
        assert_eq!(list.get_number(index), Some(77));
    }
    list.set_number(f64::NEG_INFINITY, Some(88)).unwrap();
    assert_eq!(list.get_number(f64::NAN), Some(88));
    list.set_number(f64::NAN, None).unwrap();
    assert_eq!(list.get_number(f64::INFINITY), None);
    list.set_number(-0.0, Some(9)).unwrap();
    assert_eq!(list.get(0), Some(9));
}

#[test]
fn raw_length_preserves_named_properties_but_capacity_rebuild_discards_them() {
    let list = CircularList::new(5).unwrap();
    list.set_number(0.5, Some(11)).unwrap();
    list.set_number(f64::NAN, Some(77)).unwrap();
    list.set_signed(-1, Some(99)).unwrap();
    list.set_length(1);
    list.set(0, Some(3)).unwrap();
    list.set_max_length(5).unwrap();
    assert_eq!(list.get_number(0.5), Some(11));
    assert_eq!(list.get_number(f64::INFINITY), Some(77));
    list.set_max_length(6).unwrap();
    assert_eq!(list.get(0), Some(3));
    assert_eq!(list.get_number(0.5), None);
    assert_eq!(list.get_number(f64::NAN), None);
    assert_eq!(list.get_signed(-1), None);
}
