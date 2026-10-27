// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Assertions ported from Unicode addon providers and tests.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::event::EmitError::ListenerArrayCleared as DispatchFailure;
use xterm_core::unicode::{UnicodeCharProperties as Properties, UnicodeVersionProvider};
use xterm_core::unicode11::UnicodeV11;
use xterm_core::unicode_addons::{Unicode11Addon, UnicodeGraphemesAddon};
use xterm_core::unicode_graphemes::UnicodeGraphemeProvider;
use xterm_core::unicode_properties as uc;
use xterm_core::unicode_service::{UnicodeService, UnicodeServiceError};
use xterm_core::unicode_v6::UnicodeV6;

fn provider(name: &str) -> Rc<dyn UnicodeVersionProvider> {
    if name == "11" {
        Rc::new(UnicodeV11::new())
    } else {
        let value = UnicodeGraphemeProvider::new(name.starts_with('g'));
        value.ambiguous_chars_are_wide.set(name.ends_with('w'));
        Rc::new(value)
    }
}

#[test]
fn exhaustive_codepoint_width_and_property_oracle() {
    let preceding = [0, 2, 4, 6, 0xffffffff, 0x80000000];
    let mut name = "";
    let mut active = provider("11");
    let mut end_previous = 0;
    let mut compared = 0;
    for line in include_str!("fixtures/unicode-addons-ranges.tsv").lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if name != fields[0] {
            if !name.is_empty() {
                assert_eq!(end_previous, 0x110000);
            }
            name = fields[0];
            active = provider(name);
            end_previous = 0;
        }
        let start = u32::from_str_radix(fields[1], 16).unwrap();
        let end = u32::from_str_radix(fields[2], 16).unwrap();
        assert_eq!(start, end_previous);
        assert!(end > start && end <= 0x110000);
        let values: Vec<u32> = fields[3..]
            .iter()
            .map(|text| text.parse().unwrap())
            .collect();
        assert_eq!(values.len(), 8);
        for cp in start..end {
            assert_eq!(uc::get_info(cp), values[0], "info U+{cp:x}");
            assert_eq!(
                u32::from(active.wcwidth(cp).columns()),
                values[1],
                "width {name} U+{cp:x}"
            );
            for (index, before) in preceding.iter().enumerate() {
                assert_eq!(
                    active
                        .char_properties(cp, Properties::from_raw(*before))
                        .raw(),
                    values[index + 2],
                    "property {name} U+{cp:x} before {before:x}"
                );
            }
            compared += 1;
        }
        end_previous = end;
    }
    assert_eq!(end_previous, 0x110000);
    assert_eq!(compared, 5 * 0x110000);
    assert_eq!(uc::get_info(0x110000), 0);
    assert_eq!(uc::get_info(u32::MAX), 0);
    for name in ["11", "nn", "nw", "gn", "gw"] {
        let provider = provider(name);
        for cp in [0x110000, u32::MAX] {
            assert_eq!(provider.wcwidth(cp).columns(), 1);
        }
    }
}

#[test]
fn state_helpers_and_standalone_widths_match_javascript() {
    for line in include_str!("fixtures/unicode-join.tsv").lines() {
        let values: Vec<i32> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(uc::should_join(values[0], values[1] as u32), values[2]);
        assert_eq!(
            uc::should_join_backwards(values[0] as u32, values[1]),
            values[3]
        );
    }
    for line in include_str!("fixtures/unicode-width-helpers.tsv").lines() {
        let values: Vec<u32> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(uc::info_to_width_info(values[0]), values[1]);
        assert_eq!(u32::from(uc::info_to_width(values[0], false)), values[2]);
        assert_eq!(u32::from(uc::info_to_width(values[0], true)), values[3]);
    }
}

fn units(text: &str) -> Vec<u16> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split(',')
        .map(|v| u16::from_str_radix(v, 16).unwrap())
        .collect()
}

#[test]
fn utf16_column_helpers_match_javascript() {
    for line in include_str!("fixtures/unicode-columns.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let input = units(fields[0]);
        let column = match fields[2] {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            other => other.parse().unwrap(),
        };
        let wide = fields[3] == "1";
        assert_eq!(
            uc::str_width(&input, wide),
            fields[4].parse::<usize>().unwrap()
        );
        assert_eq!(
            uc::column_to_index_in_context(&input, fields[1].parse().unwrap(), column, wide),
            fields[5].parse::<usize>().unwrap(),
            "{line}"
        );
    }
}

#[test]
fn seeded_multi_codepoint_property_traces() {
    let mut count = 0;
    for (case, line) in include_str!("fixtures/unicode-addon-traces.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = line.split('\t').collect();
        let provider = provider(fields[0]);
        let points: Vec<_> = fields[1]
            .split(',')
            .map(|v| u32::from_str_radix(v, 16).unwrap())
            .collect();
        let expected: Vec<u32> = fields[2].split(',').map(|v| v.parse().unwrap()).collect();
        assert_eq!(points.len(), 64);
        assert_eq!(expected.len(), points.len());
        let mut preceding = Properties::default();
        for (step, (cp, expected)) in points.iter().zip(expected).enumerate() {
            preceding = provider.char_properties(*cp, preceding);
            assert_eq!(
                preceding.raw(),
                expected,
                "seed15cafe case {case} step {step}"
            );
            count += 1;
        }
    }
    assert_eq!(count, 81_920);
}

fn new_service() -> Rc<UnicodeService> {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    service
}

#[test]
fn original_addon_version_and_emoji_width_assertions() {
    let service = new_service();
    let unicode11 = Unicode11Addon::new();
    unicode11.activate(&service).unwrap();
    assert!(service.versions().contains(&"11".to_owned()));
    assert_eq!(service.active_version(), "6");
    service.set_active_version("11").unwrap();
    assert_eq!(
        service
            .get_string_cell_width("🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣")
            .unwrap(),
        20
    );
    unicode11.dispose();
    assert_eq!(service.active_version(), "11");
    // Keep the original grapheme suite's exact independent registration list.
    let service = new_service();
    let addon = UnicodeGraphemesAddon::new();
    addon.activate(Rc::clone(&service)).unwrap();
    assert_eq!(service.versions(), ["6", "15", "15-graphemes"]);
    assert_eq!(service.active_version(), "15-graphemes");
    for line in include_str!("fixtures/unicode-grapheme-strings.tsv").lines() {
        let (input, expected) = line.split_once('\t').unwrap();
        assert_eq!(
            service.get_string_cell_width_utf16(&units(input)).unwrap(),
            expected.parse::<i64>().unwrap(),
            "{input}"
        );
    }
    addon.dispose().unwrap();
    assert_eq!(service.active_version(), "6");
}

#[test]
fn activation_disposal_and_reentrant_events() {
    let service = new_service();
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&events);
    let weak = Rc::downgrade(&service);
    service.on_change().subscribe(move |version| {
        let service = weak.upgrade().unwrap();
        assert_eq!(*version, service.active_version());
        output.borrow_mut().push(version.clone());
    });
    let addon = UnicodeGraphemesAddon::new();
    addon.dispose().unwrap();
    addon.activate(Rc::clone(&service)).unwrap();
    addon.dispose().unwrap();
    addon.dispose().unwrap();
    assert_eq!(&*events.borrow(), &["15-graphemes", "6", "6"]);
    addon.activate(Rc::clone(&service)).unwrap();
    addon.activate(Rc::clone(&service)).unwrap();
    addon.dispose().unwrap();
    assert_eq!(service.active_version(), "15-graphemes");
    let other = new_service();
    addon.activate(Rc::clone(&other)).unwrap();
    addon.dispose().unwrap();
    assert_eq!(other.active_version(), "6");
    assert_eq!(service.active_version(), "15-graphemes");
    let provider = Rc::new(UnicodeGraphemeProvider::default());
    other.register(provider.clone()).unwrap();
    other.set_active_version("15-graphemes").unwrap();
    assert_eq!(other.wcwidth(0xa1).unwrap().columns(), 1);
    provider.ambiguous_chars_are_wide.set(true);
    assert_eq!(other.wcwidth(0xa1).unwrap().columns(), 2);
}

#[test]
fn callback_selection_and_errors_preserve_source_order() {
    let service = new_service();
    let weak = Rc::downgrade(&service);
    let once = Cell::new(false);
    service.on_change().subscribe(move |version| {
        if version == "15-graphemes" && !once.replace(true) {
            weak.upgrade().unwrap().set_active_version("6").unwrap();
        }
    });
    let addon = UnicodeGraphemesAddon::new();
    addon.activate(Rc::clone(&service)).unwrap();
    assert_eq!(service.active_version(), "6");
    let failing = new_service();
    failing.on_change().try_subscribe(|_| Err(DispatchFailure));
    let addon = UnicodeGraphemesAddon::new();
    assert_eq!(
        addon.activate(Rc::clone(&failing)),
        Err(UnicodeServiceError::Event(DispatchFailure))
    );
    assert_eq!(failing.active_version(), "15-graphemes");
    assert_eq!(
        addon.dispose(),
        Err(UnicodeServiceError::Event(DispatchFailure))
    );
    assert_eq!(failing.active_version(), "6");
    let callback_service = new_service();
    let failure = xterm_core::event::EmitError::callback(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "Unicode addon callback payload",
    ));
    let original = failure.clone();
    callback_service
        .on_change()
        .try_subscribe(move |_| Err(failure.clone()));
    let callback_addon = UnicodeGraphemesAddon::new();
    let error = callback_addon
        .activate(Rc::clone(&callback_service))
        .unwrap_err();
    let UnicodeServiceError::Event(actual) = error else {
        panic!("lost callback error")
    };
    assert_eq!(actual, original);
    let payload = actual.callback_ref::<std::io::Error>().unwrap();
    assert_eq!(payload.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(payload.to_string(), "Unicode addon callback payload");
    assert_eq!(callback_service.active_version(), "15-graphemes");
    let empty = Rc::new(UnicodeService::new());
    empty.on_change().try_subscribe(|_| Err(DispatchFailure));
    let addon = UnicodeGraphemesAddon::new();
    assert!(matches!(
        addon.activate(Rc::clone(&empty)),
        Err(UnicodeServiceError::Event(_))
    ));
    assert_eq!(empty.versions(), ["15"]);
    assert_eq!(empty.active_version(), "15");
    assert_eq!(
        addon.dispose(),
        Err(UnicodeServiceError::UnknownVersion(String::new()))
    );
}
