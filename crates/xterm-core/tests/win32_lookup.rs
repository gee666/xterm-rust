// Licensed under MIT. Raw value, source phase and native intrinsic regressions.
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use xterm_core::input::{
    keyboard_event_source::KeyboardEventSource,
    win32_input_mode::{
        lookup::{Hint, HostValue, Table, Value},
        Win32InputMode, Win32KeyboardEvent,
    },
};

type Trace = Rc<RefCell<Vec<String>>>;
struct Event {
    value: fn() -> Value<&'static str>,
    trace: Trace,
    fail_lookup: Option<Table>,
    fail_coercion: Option<Table>,
    raw: bool,
}
impl Default for Event {
    fn default() -> Self {
        Self {
            value: || Value::Undefined,
            trace: Trace::default(),
            fail_lookup: None,
            fail_coercion: None,
            raw: false,
        }
    }
}
struct Raw {
    table: Table,
    trace: Trace,
    fail: bool,
}
impl HostValue<&'static str> for Raw {
    fn is_undefined(&self) -> bool {
        false
    }
    fn is_truthy(&self) -> bool {
        true
    }
    fn number(&self) -> Option<f64> {
        None
    }
    fn to_utf16(&self, hint: Hint) -> Result<Vec<u16>, &'static str> {
        assert_eq!(hint, Hint::String);
        assert!(self.trace.borrow().iter().any(|read| read == "shift"));
        self.trace
            .borrow_mut()
            .push(format!("coerce:{:?}", self.table));
        if self.fail {
            return Err("original coercion marker");
        }
        Ok(format!("{:?}", self.table).encode_utf16().collect())
    }
}
impl KeyboardEventSource for Event {
    type Error = &'static str;
    fn win32_lookup(&self, table: Table, _: &[u16]) -> Result<Value<Self::Error>, Self::Error> {
        self.trace.borrow_mut().push(format!("lookup:{table:?}"));
        if self.fail_lookup == Some(table) {
            return Err("original lookup marker");
        }
        Ok(if self.raw {
            Value::Host(Box::new(Raw {
                table,
                trace: self.trace.clone(),
                fail: self.fail_coercion == Some(table),
            }))
        } else {
            (self.value)()
        })
    }
    fn key_code(&self) -> Result<f64, Self::Error> {
        self.trace.borrow_mut().push("keyCode".into());
        Ok(65.0)
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        self.trace.borrow_mut().push("code".into());
        Ok(Cow::Owned("probe".encode_utf16().collect()))
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        self.trace.borrow_mut().push("key".into());
        Ok(Cow::Owned("probe".encode_utf16().collect()))
    }
    fn shift_key(&self) -> Result<bool, Self::Error> {
        self.trace.borrow_mut().push("shift".into());
        Ok(false)
    }
    fn ctrl_key(&self) -> Result<bool, Self::Error> {
        self.trace.borrow_mut().push("ctrl".into());
        Ok(false)
    }
    fn alt_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn meta_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
fn text(event: &Event) -> Vec<u16> {
    Win32InputMode
        .try_evaluate_keyboard_event(event, true)
        .unwrap()
        .key
        .unwrap()
}

#[test]
fn false_scan_values_but_only_undefined_virtual_and_control_values_fall_back() {
    type Case = (
        fn() -> Value<&'static str>,
        &'static str,
        &'static str,
        &'static str,
    );
    let cases: &[Case] = &[
        (|| Value::Undefined, "65", "0", "0"),
        (|| Value::Null, "null", "0", "null"),
        (|| Value::Boolean(false), "false", "0", "false"),
        (|| Value::Boolean(true), "true", "true", "true"),
        (|| Value::Number(-0.0), "0", "0", "0"),
        (|| Value::Number(f64::NAN), "NaN", "0", "NaN"),
        (
            || Value::Number(f64::INFINITY),
            "Infinity",
            "Infinity",
            "Infinity",
        ),
        (
            || Value::Number(f64::NEG_INFINITY),
            "-Infinity",
            "-Infinity",
            "-Infinity",
        ),
        (|| Value::Text(vec![]), "", "0", ""),
        (|| Value::Text(vec![97]), "a", "a", "a"),
    ];
    for (value, vk, sc, uc) in cases {
        let event = Event {
            value: *value,
            ..Event::default()
        };
        assert_eq!(
            text(&event),
            format!("\x1b[{vk};{sc};{uc};1;0;1_")
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            event
                .trace
                .borrow()
                .iter()
                .filter(|x| *x == "keyCode")
                .count(),
            usize::from(*vk == "65")
        );
    }
}

#[test]
fn inherited_text_does_not_replace_lone_surrogates() {
    let event = Event {
        value: || Value::Text(vec![0xd800, 0xdfff]),
        ..Event::default()
    };
    let mut expected = vec![
        27, 91, 0xd800, 0xdfff, 59, 0xd800, 0xdfff, 59, 0xd800, 0xdfff,
    ];
    expected.extend(";1;0;1_".encode_utf16());
    assert_eq!(text(&event), expected);
}

#[test]
fn raw_values_coerce_only_after_all_field_reads_and_in_template_order() {
    let event = Event {
        raw: true,
        ..Event::default()
    };
    assert_eq!(
        text(&event),
        "\x1b[VirtualKey;ScanCode;ControlChar;1;0;1_"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        *event.trace.borrow(),
        [
            "code",
            "lookup:VirtualKey",
            "code",
            "lookup:ScanCode",
            "ctrl",
            "key",
            "lookup:ControlChar",
            "shift",
            "ctrl",
            "code",
            "coerce:VirtualKey",
            "coerce:ScanCode",
            "coerce:ControlChar"
        ]
    );
}

#[test]
fn lookup_and_coercion_failures_stop_at_the_source_stage() {
    for (index, table) in [Table::VirtualKey, Table::ScanCode, Table::ControlChar]
        .into_iter()
        .enumerate()
    {
        for coercion in [false, true] {
            let event = Event {
                raw: true,
                fail_lookup: (!coercion).then_some(table),
                fail_coercion: coercion.then_some(table),
                ..Event::default()
            };
            let result = Win32InputMode.try_evaluate_keyboard_event(&event, true);
            assert_eq!(
                result.unwrap_err(),
                if coercion {
                    "original coercion marker"
                } else {
                    "original lookup marker"
                }
            );
            let trace = event.trace.borrow();
            assert_eq!(trace.iter().any(|s| s == "shift"), coercion);
            assert_eq!(
                trace.iter().filter(|s| s.starts_with("coerce:")).count(),
                if coercion { index + 1 } else { 0 }
            );
        }
    }
}

#[test]
fn native_intrinsic_names_use_the_documented_v8_representation() {
    for (name, value) in [
        ("toString", "function toString() { [native code] }"),
        ("constructor", "function Object() { [native code] }"),
        ("__proto__", "[object Object]"),
    ] {
        let units: Vec<_> = name.encode_utf16().collect();
        let event = Win32KeyboardEvent {
            key: &units,
            code: &units,
            ..Win32KeyboardEvent::default()
        };
        assert_eq!(
            Win32InputMode
                .evaluate_keyboard_event(&event, false)
                .key
                .unwrap(),
            format!("\x1b[{value};{value};{value};0;0;1_")
                .encode_utf16()
                .collect::<Vec<_>>()
        );
    }
}
