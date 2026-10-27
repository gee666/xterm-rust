//! Directed gaps from docs/acceptance/charset.md, separate from the original tests.

use std::collections::BTreeMap;
use std::sync::Arc;
use xterm_core::charset::{builtin_charset, Charset, CharsetError, CharsetService};

fn same_map(a: &Option<Charset>, b: &Option<Charset>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

#[test]
fn builtin_identity_is_stable_and_distinct_except_declared_aliases() {
    let ids = "04567=ACEHKQRYZ";
    for a in ids.chars() {
        assert!(same_map(&builtin_charset(a), &builtin_charset(a)));
        for b in ids.chars() {
            let aliases = [('C', '5'), ('E', '6'), ('H', '7')];
            let expected = a == b || aliases.contains(&(a, b)) || aliases.contains(&(b, a));
            assert_eq!(same_map(&builtin_charset(a), &builtin_charset(b)), expected);
        }
    }
    for id in ['\0', 'X', '\u{e9}', '\u{ffff}', '\u{1f600}'] {
        assert!(builtin_charset(id).is_none(), "unknown designator {id:?}");
    }
}

#[test]
fn initial_state_and_g0_through_g3_invocation_do_not_grow_slots() {
    let mut service = CharsetService::default();
    assert_eq!(service.glevel, 0);
    assert!(service.charset.is_none());
    assert!(service.charsets().is_empty());
    for g in 0..4 {
        service.set_g_level(g);
        assert_eq!(service.glevel, g);
        assert!(service.charset.is_none());
        assert!(service.charsets().is_empty());
        service.set_g_charset(g, builtin_charset('0')).unwrap();
        assert!(same_map(&service.charset, &builtin_charset('0')));
        assert_eq!(service.charsets().len(), g + 1);
        service.charsets_mut().clear();
    }
}

#[test]
fn sparse_designation_and_lower_overwrite_preserve_higher_slots() {
    let mut service = CharsetService::default();
    service.set_g_charset(16, builtin_charset('C')).unwrap();
    assert_eq!(service.charsets().len(), 17);
    assert!(service.charsets()[..16].iter().all(Option::is_none));
    service.set_g_charset(0, builtin_charset('4')).unwrap();
    service.set_g_charset(0, builtin_charset('4')).unwrap();
    assert_eq!(service.charsets().len(), 17);
    assert!(same_map(&service.charsets()[0], &builtin_charset('4')));
    assert!(same_map(&service.charsets()[16], &builtin_charset('5')));
    assert!(same_map(&service.charset, &builtin_charset('4')));
    service.set_g_level(16);
    service.set_g_charset(16, None).unwrap();
    assert!(service.charset.is_none());
    assert_eq!(service.charsets().len(), 17);
}

#[test]
fn direct_field_array_growth_and_truncation_keep_active_snapshot_until_invocation() {
    let mut service = CharsetService::default();
    service.set_g_charset(0, builtin_charset('0')).unwrap();
    service.glevel = 3;
    assert!(same_map(&service.charset, &builtin_charset('0')));
    service.charset = builtin_charset('4');
    assert_eq!(service.glevel, 3);
    assert!(same_map(&service.charsets()[0], &builtin_charset('0')));
    service.charsets_mut().push(builtin_charset('A'));
    service.charsets_mut().resize(4, None);
    service.charsets_mut()[3] = builtin_charset('K');
    assert!(same_map(&service.charset, &builtin_charset('4')));
    service.set_g_level(3);
    assert!(same_map(&service.charset, &builtin_charset('K')));
    service.charsets_mut().truncate(1);
    assert!(same_map(&service.charset, &builtin_charset('K')));
    service.set_g_level(3);
    assert!(service.charset.is_none());
    assert_eq!(service.charsets().len(), 1);
    for _ in 0..2 {
        service.reset();
        assert_eq!(service.glevel, 0);
        assert!(service.charset.is_none());
        assert!(service.charsets().is_empty());
    }
    service.set_g_charset(0, builtin_charset('A')).unwrap();
    assert!(same_map(&service.charset, &builtin_charset('A')));
    service.set_g_level(0);
    assert!(same_map(&service.charset, &builtin_charset('A')));
}

#[test]
fn custom_maps_preserve_all_raw_utf16_value_boundaries() {
    let entries = BTreeMap::from([
        (0, vec![0]),
        (1, vec![0, 0x0061]),
        (2, vec![0xd83d, 0xde00]),
        (3, vec![0xd800]),
        (4, vec![0xdc00]),
        (0xd800, vec![0xffff]),
        (0xffff, vec![]),
    ]);
    let map = Arc::new(entries.clone());
    let mut service = CharsetService::default();
    service.set_g_charset(0, Some(Arc::clone(&map))).unwrap();
    assert_eq!(service.charset.as_deref(), Some(&entries));
    let clone = service.clone();
    service.reset();
    assert_eq!(clone.charset.as_deref(), Some(&entries));
    assert!(Arc::ptr_eq(clone.charset.as_ref().unwrap(), &map));
}

#[test]
fn both_growth_failures_preserve_every_slot_and_allow_recovery() {
    let mut service = CharsetService::default();
    for (g, id) in ['0', '4', 'A', 'K'].into_iter().enumerate() {
        service.set_g_charset(g, builtin_charset(id)).unwrap();
    }
    service.set_g_level(2);
    let before = service.clone();
    for g in [usize::MAX, usize::MAX - 1] {
        let result = service.set_g_charset(g, builtin_charset('Y'));
        if g == usize::MAX {
            assert!(matches!(result, Err(CharsetError::LevelOverflow)));
        } else {
            assert!(matches!(result, Err(CharsetError::Allocation(_))));
        }
        assert_eq!(service.glevel, before.glevel);
        assert!(same_map(&service.charset, &before.charset));
        assert_eq!(service.charsets().len(), before.charsets().len());
        for (actual, expected) in service.charsets().iter().zip(before.charsets()) {
            assert!(same_map(actual, expected));
        }
    }
    service.set_g_charset(4, builtin_charset('Y')).unwrap();
    service.set_g_level(4);
    assert!(same_map(&service.charset, &builtin_charset('Y')));
    service.reset();
    assert!(service.charsets().is_empty());
    service.set_g_charset(0, builtin_charset('Q')).unwrap();
    assert!(same_map(&service.charset, &builtin_charset('Q')));
}
