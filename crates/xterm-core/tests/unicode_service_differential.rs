// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. UnicodeService UTF-16 operation oracle.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use xterm_core::lifecycle::Disposable;
use xterm_core::unicode::{
    UnicodeCharProperties as Properties, UnicodeCharWidth as Width, UnicodeVersionProvider,
};
use xterm_core::unicode_service::UnicodeService;
use xterm_core::unicode_v6::UnicodeV6;

struct TraceProvider(Rc<RefCell<Vec<String>>>);

impl UnicodeVersionProvider for TraceProvider {
    fn version(&self) -> &str {
        "trace"
    }

    fn wcwidth(&self, codepoint: u32) -> Width {
        self.0.borrow_mut().push(format!("w:{codepoint}"));
        match codepoint % 3 {
            0 => Width::Zero,
            1 => Width::Narrow,
            _ => Width::Wide,
        }
    }

    fn char_properties(&self, codepoint: u32, preceding: Properties) -> Properties {
        self.0
            .borrow_mut()
            .push(format!("p:{codepoint}:{}", preceding.raw()));
        let state = ((preceding.raw() >> 3) + codepoint) & 0xffffff;
        Properties::new(
            state,
            codepoint % 4,
            codepoint & 1 == 0 && preceding.raw() != 0,
        )
    }
}

#[test]
fn seeded_utf16_traces_match_javascript() {
    let v6 = UnicodeService::new();
    v6.register(Rc::new(UnicodeV6::new())).unwrap();
    let custom = UnicodeService::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    custom
        .register(Rc::new(TraceProvider(Rc::clone(&log))))
        .unwrap();
    let mut count = 0;
    for (index, row) in include_str!("fixtures/unicode-service-widths.tsv")
        .lines()
        .enumerate()
    {
        let values: Vec<_> = row.split('\t').collect();
        assert_eq!(values.len(), 4);
        let units: Vec<_> = if values[0].is_empty() {
            Vec::new()
        } else {
            values[0]
                .split(',')
                .map(|value| u16::from_str_radix(value, 16).unwrap())
                .collect()
        };
        let expected_v6: i64 = values[1].parse().unwrap();
        let expected_custom: i64 = values[2].parse().unwrap();
        let message = format!("trace {index}: {units:x?}, seed 0x71de63a1");
        assert_eq!(
            v6.get_string_cell_width_utf16(&units).unwrap(),
            expected_v6,
            "{message}"
        );
        if let Ok(text) = String::from_utf16(&units) {
            assert_eq!(
                v6.get_string_cell_width(&text).unwrap(),
                expected_v6,
                "{message}"
            );
        }
        log.borrow_mut().clear();
        assert_eq!(
            custom.get_string_cell_width_utf16(&units).unwrap(),
            expected_custom,
            "{message}"
        );
        assert_eq!(log.borrow().join(","), values[3], "{message}");
        count += 1;
    }
    assert_eq!(count, 4106);
}

#[test]
fn custom_provider_call_order_matches_source() {
    let service = UnicodeService::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    service
        .register(Rc::new(TraceProvider(Rc::clone(&log))))
        .unwrap();
    service
        .get_string_cell_width_utf16(&[0xd800, 0x301, 0xd800])
        .unwrap();
    assert_eq!(*log.borrow(), ["w:769", "p:55296:0", "w:55296"]);
    log.borrow_mut().clear();
    service
        .get_string_cell_width_utf16(&[0xd800, 0xd800, 0xdc00])
        .unwrap();
    assert_eq!(*log.borrow(), ["w:55296", "p:55296:0", "p:56320:442368"]);
}

struct ShrinkingProvider;
impl UnicodeVersionProvider for ShrinkingProvider {
    fn version(&self) -> &str {
        "shrinking"
    }
    fn wcwidth(&self, _: u32) -> Width {
        Width::Zero
    }
    fn char_properties(&self, code: u32, preceding: Properties) -> Properties {
        if code == 65 {
            Properties::new(0, 3, false)
        } else {
            Properties::new(0, 0, preceding.raw() != 0)
        }
    }
}

#[test]
fn joined_width_can_decrease() {
    let service = UnicodeService::new();
    service.register(Rc::new(ShrinkingProvider)).unwrap();
    assert_eq!(service.get_string_cell_width("A").unwrap(), 3);
    assert_eq!(service.get_string_cell_width("AB").unwrap(), 0);
    assert_eq!(service.get_string_cell_width("ABC").unwrap(), 0);
}

struct RegistryProvider {
    name: String,
    width: Width,
}

impl UnicodeVersionProvider for RegistryProvider {
    fn version(&self) -> &str {
        &self.name
    }
    fn wcwidth(&self, _: u32) -> Width {
        self.width
    }
    fn char_properties(&self, _: u32, _: Properties) -> Properties {
        Properties::new(0, u32::from(self.width.columns()), false)
    }
}

fn decode_name(value: &str) -> String {
    if value == "~" {
        return String::new();
    }
    let units: Vec<_> = value
        .split(',')
        .map(|unit| u16::from_str_radix(unit, 16).unwrap())
        .collect();
    String::from_utf16(&units).unwrap()
}

fn decode_names(value: &str) -> Vec<String> {
    if value == "-" {
        Vec::new()
    } else {
        value.split(';').map(decode_name).collect()
    }
}

#[test]
fn registry_operation_traces_match_javascript() {
    let service = UnicodeService::new();
    let notes = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&notes);
    let _listener = service
        .on_change()
        .subscribe(move |version| output.borrow_mut().push(version.clone()));
    let mut count = 0;
    for (index, row) in include_str!("fixtures/unicode-service-registry.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 8, "row {index}");
        let name = decode_name(fields[1]);
        let width = match fields[2] {
            "0" => Width::Zero,
            "1" => Width::Narrow,
            "2" => Width::Wide,
            other => panic!("unexpected width {other}"),
        };
        let result = match fields[0] {
            "r" => service.register(Rc::new(RegistryProvider { name, width })),
            "s" => service.set_active_version(&name),
            "d" => {
                service.dispose();
                Ok(())
            }
            other => panic!("unexpected operation {other}"),
        };
        if fields[6] == "-" {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().to_string(), fields[6], "row {index}");
        }
        assert_eq!(
            service.versions(),
            decode_names(fields[3]),
            "versions row {index}"
        );
        assert_eq!(
            service.active_version(),
            decode_name(fields[4]),
            "active row {index}"
        );
        assert_eq!(
            service.wcwidth(65).unwrap().columns(),
            fields[5].parse::<u8>().unwrap(),
            "width row {index}"
        );
        assert_eq!(
            *notes.borrow(),
            decode_names(fields[7]),
            "events row {index}"
        );
        count += 1;
    }
    assert_eq!(count, 531);
}

struct SwitchingProvider(Weak<UnicodeService>);
impl UnicodeVersionProvider for SwitchingProvider {
    fn version(&self) -> &str {
        "switching"
    }
    fn wcwidth(&self, _: u32) -> Width {
        Width::Narrow
    }
    fn char_properties(&self, _: u32, _: Properties) -> Properties {
        self.0.upgrade().unwrap().set_active_version("6").unwrap();
        Properties::new(0, 2, false)
    }
}

#[test]
fn provider_callbacks_can_switch_active_version_without_borrow_panics() {
    let service = Rc::new(UnicodeService::new());
    service
        .register(Rc::new(SwitchingProvider(Rc::downgrade(&service))))
        .unwrap();
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    assert_eq!(service.get_string_cell_width("ab").unwrap(), 3);
    assert_eq!(service.active_version(), "6");
}
