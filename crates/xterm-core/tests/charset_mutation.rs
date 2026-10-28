//! Mutable object semantics that an immutable table handle cannot preserve.
use std::collections::BTreeMap;
use xterm_core::charset::{builtin_charset, Charset, CharsetService, CharsetValue};

#[test]
fn edits_are_visible_through_selected_saved_and_cloned_handles() {
    let map = Charset::new(BTreeMap::from([(u16::from(b'q'), vec![0x2500])]));
    let mut service = CharsetService::default();
    service.set_g_charset(1, Some(map.clone())).unwrap();
    service.set_g_level(1);
    let saved = service.charsets().to_vec();
    let clone = service.clone();
    map.set(u16::from(b'q'), Some(vec![0x58, 0x59])).unwrap();
    for handle in [
        service.charset.as_ref(),
        clone.charset.as_ref(),
        saved[1].as_ref(),
    ] {
        let handle = handle.unwrap();
        assert!(Charset::ptr_eq(handle, &map));
        assert_eq!(handle.get(u16::from(b'q')).unwrap(), Some(vec![0x58, 0x59]));
        assert_eq!(handle.get_first(u16::from(b'q')).unwrap(), Some(0x58));
    }
    service.reset();
    assert_eq!(
        clone.charset.unwrap().get_first(u16::from(b'q')).unwrap(),
        Some(0x58)
    );
}

#[test]
fn empty_undefined_and_absent_properties_remain_distinguishable() {
    let map = Charset::default();
    map.set(1, Some(Vec::new())).unwrap();
    map.set(2, None).unwrap();
    assert_eq!(map.get_first(1).unwrap(), None);
    assert_eq!(map.get_first(2).unwrap(), None);
    assert_eq!(map.get_first(3).unwrap(), None);
    let snapshot = map.snapshot().unwrap();
    assert_eq!(snapshot.get(&1), Some(&CharsetValue::Text(Vec::new())));
    assert_eq!(snapshot.get(&2), Some(&CharsetValue::Undefined));
    assert!(!snapshot.contains_key(&3));
    assert!(map.remove(2).unwrap());
    assert!(!map.remove(2).unwrap());
    assert!(!map.snapshot().unwrap().contains_key(&2));
    map.set(1, Some(vec![0])).unwrap();
    assert_eq!(map.get_first(1).unwrap(), Some(0));
}

#[test]
fn borrowed_results_are_owned_snapshots_not_hidden_live_guards() {
    let map = Charset::new(BTreeMap::from([(1, vec![0xd800, 0xdc00])]));
    let first = map.get_first(1).unwrap();
    let text = map.get(1).unwrap();
    let snapshot = map.snapshot().unwrap();
    map.set(1, Some(vec![0xdc00])).unwrap();
    assert_eq!(first, Some(0xd800));
    assert_eq!(text, Some(vec![0xd800, 0xdc00]));
    assert_eq!(
        snapshot.get(&1),
        Some(&CharsetValue::Text(vec![0xd800, 0xdc00]))
    );
    assert_eq!(map.get_first(1).unwrap(), Some(0xdc00));
}

#[test]
fn builtin_aliases_and_repeated_lookups_share_edits() {
    let finnish = builtin_charset('C').unwrap();
    let alias = builtin_charset('5').unwrap();
    assert!(Charset::ptr_eq(&finnish, &alias));
    let key = u16::from(b'!');
    assert_eq!(finnish.get(key).unwrap(), None);
    finnish.set(key, Some(vec![0x2603])).unwrap();
    assert_eq!(alias.get_first(key).unwrap(), Some(0x2603));
    assert_eq!(
        builtin_charset('C').unwrap().get_first(key).unwrap(),
        Some(0x2603)
    );
    assert!(finnish.remove(key).unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_threads_have_separate_builtin_realms() {
    let map = builtin_charset('C').unwrap();
    let key = u16::from(b'?');
    assert_eq!(map.get_first(key).unwrap(), None);
    map.set(key, Some(vec![0x2603])).unwrap();
    let child = std::thread::spawn(move || builtin_charset('C').unwrap().get_first(key).unwrap());
    assert_eq!(child.join().unwrap(), None);
    assert!(map.remove(key).unwrap());
}
