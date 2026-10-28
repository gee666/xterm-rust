// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::decoration::OverviewPosition;
use xterm_web::renderer::color_zone_store::{
    ColorZone, ColorZoneOptions, ColorZonePadding, ColorZoneStore,
};
fn options(color: &str, position: Option<OverviewPosition>) -> ColorZoneOptions {
    ColorZoneOptions {
        color: color.encode_utf16().collect(),
        position,
    }
}
fn store() -> ColorZoneStore {
    let mut store = ColorZoneStore::new();
    store.set_padding(Rc::new(RefCell::new(ColorZonePadding::uniform(1.0))));
    store
}
fn values(store: &mut ColorZoneStore) -> Vec<ColorZone> {
    store
        .zones()
        .unwrap()
        .borrow()
        .iter()
        .map(|zone| zone.borrow().clone())
        .collect()
}
#[test]
fn should_merge_adjacent_zones() {
    let mut store = store();
    let options = options("red", Some(OverviewPosition::Full));
    store.add(0.0, Some(&options)).unwrap();
    store.add(1.0, Some(&options)).unwrap();
    assert_eq!(
        values(&mut store),
        [ColorZone {
            color: "red".encode_utf16().collect(),
            position: Some(OverviewPosition::Full),
            start_buffer_line: 0.0,
            end_buffer_line: 1.0
        }]
    );
}
#[test]
fn should_not_merge_non_adjacent_zones() {
    let mut store = store();
    let options = options("red", Some(OverviewPosition::Full));
    store.add(0.0, Some(&options)).unwrap();
    store.add(2.0, Some(&options)).unwrap();
    assert_eq!(
        values(&mut store),
        [
            ColorZone {
                color: "red".encode_utf16().collect(),
                position: Some(OverviewPosition::Full),
                start_buffer_line: 0.0,
                end_buffer_line: 0.0
            },
            ColorZone {
                color: "red".encode_utf16().collect(),
                position: Some(OverviewPosition::Full),
                start_buffer_line: 2.0,
                end_buffer_line: 2.0
            }
        ]
    );
}
#[test]
fn should_reuse_zone_objects() {
    let mut store = store();
    let options = options("red", Some(OverviewPosition::Full));
    store.add(0.0, Some(&options)).unwrap();
    let first = store.zones().unwrap().borrow()[0].clone();
    store.clear().unwrap();
    store.add(1.0, Some(&options)).unwrap();
    assert!(Rc::ptr_eq(&first, &store.zones().unwrap().borrow()[0]));
}
#[test]
fn first_matching_zone_extends_without_coalescing_later_zones() {
    let mut store = store();
    let options = options("red", None);
    store.add(0.0, Some(&options)).unwrap();
    store.add(2.0, Some(&options)).unwrap();
    store.add(1.0, Some(&options)).unwrap();
    let zones = values(&mut store);
    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0].end_buffer_line, 1.0);
    assert_eq!(zones[1].start_buffer_line, 2.0);
    store
        .add(
            2.0,
            Some(&ColorZoneOptions {
                color: "red".encode_utf16().collect(),
                position: Some(OverviewPosition::Full),
            }),
        )
        .unwrap();
}
#[test]
fn missing_settings_padding_and_mutable_padding_identity() {
    let mut store = ColorZoneStore::new();
    store.add(0.0, None).unwrap();
    assert!(values(&mut store).is_empty());
    let padding = Rc::new(RefCell::new(ColorZonePadding {
        full: None,
        ..ColorZonePadding::default()
    }));
    store.set_padding(padding.clone());
    let red = options("red", None);
    store.add(0.0, Some(&red)).unwrap();
    store.add(1.0, Some(&red)).unwrap();
    assert_eq!(values(&mut store).len(), 2);
    padding.borrow_mut().full = Some(1.0);
    store.add(-1.0, Some(&red)).unwrap();
    assert_eq!(values(&mut store)[0].start_buffer_line, -1.0);
}
#[test]
fn getter_trims_pool_and_list_identity_and_lone_surrogates_survive() {
    let mut store = store();
    let malformed = ColorZoneOptions {
        color: vec![0xd800],
        position: None,
    };
    store.add(0.0, Some(&malformed)).unwrap();
    let list = store.zones().unwrap();
    let old = list.borrow()[0].clone();
    store.clear().unwrap();
    assert!(Rc::ptr_eq(&list, &store.zones().unwrap()));
    store.add(1.0, Some(&malformed)).unwrap();
    let new = list.borrow()[0].clone();
    assert!(!Rc::ptr_eq(&old, &new));
    assert_eq!(new.borrow().color, [0xd800]);
}
#[test]
fn all_positions_distinguish_absence_from_full_and_color_identity() {
    let mut store = store();
    for position in [
        None,
        Some(OverviewPosition::Full),
        Some(OverviewPosition::Left),
        Some(OverviewPosition::Center),
        Some(OverviewPosition::Right),
    ] {
        store.add(1.0, Some(&options("red", position))).unwrap();
        store.add(1.0, Some(&options("blue", position))).unwrap();
    }
    assert_eq!(values(&mut store).len(), 10);
}
#[test]
fn fractional_nan_negative_padding_and_nonfinite_lines_follow_js() {
    let mut store = ColorZoneStore::new();
    let red = options("red", None);
    let padding = Rc::new(RefCell::new(ColorZonePadding::uniform(0.5)));
    store.set_padding(padding.clone());
    store.add(0.0, Some(&red)).unwrap();
    store.add(0.5, Some(&red)).unwrap();
    assert_eq!(values(&mut store).len(), 1);
    padding.borrow_mut().full = Some(-1.0);
    store.add(0.25, Some(&red)).unwrap(); // Intersection ignores negative padding.
    assert_eq!(values(&mut store).len(), 1);
    padding.borrow_mut().full = Some(f64::NAN);
    store.add(1.0, Some(&red)).unwrap();
    store.add(f64::INFINITY, Some(&red)).unwrap();
    store.add(f64::NAN, Some(&red)).unwrap();
    let zones = values(&mut store);
    assert_eq!(zones.len(), 4);
    assert!(zones[3].start_buffer_line.is_nan());
    assert!(zones[2].start_buffer_line.is_infinite());
}
#[test]
fn external_zone_mutations_remain_visible_to_merging() {
    let mut store = store();
    let red = options("red", None);
    store.add(0.0, Some(&red)).unwrap();
    let zones = store.zones().unwrap();
    zones.borrow()[0].borrow_mut().color = "blue".encode_utf16().collect();
    store.add(0.0, Some(&red)).unwrap();
    assert_eq!(zones.borrow().len(), 2);
    zones.borrow()[0].borrow_mut().start_buffer_line = -10.0;
    store.add(-9.0, Some(&options("blue", None))).unwrap();
    assert_eq!(zones.borrow().len(), 2);
}
#[test]
fn borrowed_external_objects_are_recoverable_errors_not_panics() {
    let mut store = store();
    let red = options("red", None);
    store.add(0.0, Some(&red)).unwrap();
    let list = store.zones().unwrap();
    let borrow = list.borrow();
    assert!(store.clear().is_err());
    drop(borrow);
    let zone = list.borrow()[0].clone();
    let borrow = zone.borrow_mut();
    assert!(store.add(1.0, Some(&red)).is_err());
    drop(borrow);
    store.add(1.0, Some(&red)).unwrap();
}
