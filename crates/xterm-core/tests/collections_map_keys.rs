use xterm_core::multi_key_map::{FourKeyMap, TwoKeyMap};

#[test]
fn all_four_positions_remain_independent_with_number_string_aliases() {
    let mut map = FourKeyMap::new();
    let base = [1, 2, 3, 4];
    map.set(1, 2, 3, 4, "base".to_owned());
    for position in 0..4 {
        let mut keys = base;
        keys[position] = 8;
        map.set(keys[0], keys[1], keys[2], keys[3], position.to_string());
    }
    assert_eq!(map.get("1", "2", "3", "4"), Some(&"base".to_owned()));
    for position in 0..4 {
        let mut keys = base.map(|key| key.to_string());
        keys[position] = "8".into();
        assert_eq!(
            map.get(
                keys[0].clone(),
                keys[1].clone(),
                keys[2].clone(),
                keys[3].clone()
            ),
            Some(&position.to_string())
        );
    }
    map.clear();
    assert_eq!(map.get(1, 2, 3, 4), None);
}

#[test]
fn raw_utf16_property_keys_preserve_lone_surrogates_and_embedded_zero() {
    let mut map = TwoKeyMap::new();
    for key in [
        vec![],
        vec![0],
        vec![0xd800],
        vec![0xdc00],
        vec![0xd83d, 0xde00],
        vec![0x0061, 0, 0x0062],
    ] {
        map.set(key.clone(), "key", key.len());
        assert_eq!(map.get(key.as_slice(), "key"), Some(&key.len()));
    }
    assert_ne!(map.get(&[0xd800][..], "key"), map.get("�", "key"));
    map.set("😀", "key", 99);
    assert_eq!(map.get(&[0xd83d, 0xde00][..], "key"), Some(&99));
}

#[test]
fn extreme_finite_float_keys_use_ecmascript_not_rust_display() {
    let mut map = TwoKeyMap::new();
    for (number, text) in [
        (f64::MAX, "1.7976931348623157e+308"),
        (f64::MIN_POSITIVE, "2.2250738585072014e-308"),
        (f64::from_bits(1), "5e-324"),
        (9007199254740991.0, "9007199254740991"),
        (9007199254740992.0, "9007199254740992"),
        (-0.0, "0"),
    ] {
        map.set(number, "key", text);
        assert_eq!(map.get(text, "key"), Some(&text));
    }
}
