// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. See LICENSE.
// Four service tests below retain CharsetService.test.ts assertions.

use std::collections::BTreeMap;
use xterm_core::charset::{
    builtin_charset, Charset, CharsetError, CharsetService, DEFAULT_CHARSET,
};

fn same_map(actual: &Option<Charset>, expected: &Option<Charset>) -> bool {
    match (actual, expected) {
        (None, None) => true,
        (Some(actual), Some(expected)) => Charset::ptr_eq(actual, expected),
        _ => false,
    }
}

#[test]
fn should_not_update_active_charset_when_designating_an_inactive_glevel() {
    let mut service = CharsetService::default();
    service.set_g_charset(1, builtin_charset('0')).unwrap();
    assert_eq!(service.glevel, 0);
    assert!(service.charset.is_none());
}

#[test]
fn should_expose_the_designated_charset_after_set_g_level() {
    let mut service = CharsetService::default();
    service.set_g_charset(1, builtin_charset('0')).unwrap();
    service.set_g_level(1);
    assert!(same_map(&service.charset, &builtin_charset('0')));
}

#[test]
fn should_update_active_charset_when_designating_the_current_glevel() {
    let mut service = CharsetService::default();
    service.set_g_level(1);
    service.set_g_charset(1, builtin_charset('0')).unwrap();
    assert!(same_map(&service.charset, &builtin_charset('0')));
}

#[test]
fn should_reset_glevel_charsets_and_active_charset() {
    let mut service = CharsetService::default();
    service.set_g_charset(1, builtin_charset('0')).unwrap();
    service.set_g_level(1);
    service.reset();
    assert_eq!(service.glevel, 0);
    assert!(service.charsets().is_empty());
    assert!(service.charset.is_none());
}

fn hex_unit(value: &str) -> u16 {
    u16::from_str_radix(value, 16).unwrap()
}

fn designator(value: &str) -> char {
    char::from_u32(u32::from(hex_unit(value))).unwrap()
}

#[test]
fn every_ascii_mapping_matches_javascript() {
    let mut rows = 0;
    for row in include_str!("fixtures/charset-maps.tsv").lines() {
        let columns: Vec<_> = row.split('\t').collect();
        assert_eq!(columns.len(), 3);
        let id = designator(columns[0]);
        let input = hex_unit(columns[1]);
        let charset = builtin_charset(id);
        let actual = charset.as_ref().and_then(|map| map.get(input).unwrap());
        let expected =
            (columns[2] != "-").then(|| columns[2].split(',').map(hex_unit).collect::<Vec<_>>());
        assert_eq!(actual, expected, "map {id:?}, input {input:x}");
        rows += 1;
    }
    assert_eq!(rows, 2176);
}

const DESIGNATORS: &str = "04567=ABCEHKQRYZ";

fn canonical(map: &Option<Charset>) -> Option<char> {
    map.as_ref().map(|map| {
        DESIGNATORS
            .chars()
            .find(|&id| builtin_charset(id).is_some_and(|builtin| Charset::ptr_eq(map, &builtin)))
            .unwrap()
    })
}

fn expected_id(value: &str) -> Option<char> {
    (value != "-").then(|| designator(value))
}

#[test]
fn seeded_designation_trace_matches_javascript() {
    let mut service = CharsetService::default();
    let mut rows = 0;
    for (index, row) in include_str!("fixtures/charset-service.tsv")
        .lines()
        .enumerate()
    {
        let columns: Vec<_> = row.split('\t').collect();
        assert_eq!(columns.len(), 6, "row {index}");
        match columns[0] {
            "s" => service
                .set_g_charset(
                    columns[1].parse().unwrap(),
                    expected_id(columns[2]).and_then(builtin_charset),
                )
                .unwrap(),
            "l" => service.set_g_level(columns[1].parse().unwrap()),
            "r" => service.reset(),
            op => panic!("unknown operation {op} at row {index}"),
        }
        assert_eq!(service.glevel, columns[3].parse().unwrap(), "row {index}");
        assert_eq!(
            canonical(&service.charset),
            expected_id(columns[4]),
            "row {index}"
        );
        let expected: Vec<_> = if columns[5].is_empty() {
            Vec::new()
        } else {
            columns[5].split(',').map(expected_id).collect()
        };
        let actual: Vec<_> = service.charsets().iter().map(canonical).collect();
        assert_eq!(actual, expected, "designations at row {index}");
        rows += 1;
    }
    assert_eq!(rows, 4107);
}

#[test]
fn aliases_share_identity_and_undefined_sets_have_no_map() {
    assert!(DEFAULT_CHARSET.is_none());
    assert!(builtin_charset('B').is_none());
    assert!(builtin_charset('X').is_none());
    assert!(builtin_charset('\u{1f600}').is_none());
    for (a, b) in [('C', '5'), ('E', '6'), ('H', '7')] {
        assert!(same_map(&builtin_charset(a), &builtin_charset(b)));
    }
    assert!(!same_map(&builtin_charset('A'), &builtin_charset('4')));
    for id in DESIGNATORS.chars() {
        if let Some(map) = builtin_charset(id) {
            assert!(map.snapshot().unwrap().keys().all(|&unit| unit < 128));
        }
    }
}

#[test]
fn custom_maps_preserve_utf16_empty_values_and_identity() {
    let charset = Charset::new(BTreeMap::from([
        (0x005b, vec![0x0069, 0x006a]),
        (0x0078, vec![]),
        (0xd800, vec![0xdc00, 0xd800]),
    ]));
    let mut service = CharsetService::default();
    service.set_g_charset(0, Some(charset.clone())).unwrap();
    assert!(Charset::ptr_eq(service.charset.as_ref().unwrap(), &charset));
    assert_eq!(
        service.charset.as_ref().unwrap().get(0xd800).unwrap(),
        Some(vec![0xdc00, 0xd800])
    );
    assert_eq!(
        service.charset.as_ref().unwrap().get(0x0078).unwrap(),
        Some(vec![])
    );
    assert_eq!(
        service.charset.as_ref().unwrap().get(0x005b).unwrap(),
        Some(vec![0x0069, 0x006a])
    );
    let clone = service.clone();
    assert!(same_map(&service.charset, &clone.charset));
    service.reset();
    assert!(Charset::ptr_eq(clone.charset.as_ref().unwrap(), &charset));
}

#[test]
fn direct_array_edits_do_not_refresh_the_active_charset() {
    let mut service = CharsetService::default();
    service.set_g_charset(0, builtin_charset('0')).unwrap();
    service.charsets_mut()[0] = builtin_charset('4');
    assert!(same_map(&service.charset, &builtin_charset('0')));
    service.set_g_level(0);
    assert!(same_map(&service.charset, &builtin_charset('4')));
    service.charsets_mut().clear();
    assert!(same_map(&service.charset, &builtin_charset('4')));
    service.set_g_level(0);
    assert!(service.charset.is_none());
}

#[test]
fn failed_growth_leaves_the_service_unchanged() {
    let mut service = CharsetService::default();
    service.set_g_charset(0, builtin_charset('0')).unwrap();
    assert!(matches!(
        service.set_g_charset(usize::MAX, builtin_charset('4')),
        Err(CharsetError::LevelOverflow)
    ));
    assert!(matches!(
        service.set_g_charset(usize::MAX - 1, builtin_charset('4')),
        Err(CharsetError::Allocation(_))
    ));
    assert_eq!(service.glevel, 0);
    assert_eq!(service.charsets().len(), 1);
    assert!(same_map(&service.charset, &builtin_charset('0')));
    service.set_g_level(usize::MAX);
    assert_eq!(service.glevel, usize::MAX);
    assert!(service.charset.is_none());
}
