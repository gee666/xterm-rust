// Licensed under MIT. Portable raw-value contracts also exercised in real realms.
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use xterm_core::input::{
    keyboard_event_source::KeyboardEventSource,
    kitty_keyboard::{
        self,
        lookup::{Hint, HostValue, Table, Value},
        EventType,
    },
};

type Trace = Rc<RefCell<Vec<String>>>;
struct Raw {
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
        assert_eq!(hint, Hint::Default);
        self.trace.borrow_mut().push("convert:default".into());
        if self.fail {
            return Err("original conversion marker");
        }
        Ok(vec![0xd800])
    }
}
struct Event {
    table: Table,
    value: fn(Trace) -> Value<&'static str>,
    trace: Trace,
    cached_functional: bool,
    fail_lookup: bool,
}
impl Event {
    fn new(table: Table) -> Self {
        Self {
            table,
            value: |trace| Value::Host(Box::new(Raw { trace, fail: false })),
            trace: Trace::default(),
            cached_functional: false,
            fail_lookup: false,
        }
    }
}
impl KeyboardEventSource for Event {
    type Error = &'static str;
    fn kitty_lookup(&self, table: Table, _: &[u16]) -> Result<Value<Self::Error>, Self::Error> {
        let entry = format!("lookup:{table:?}");
        let repeated = self.trace.borrow().contains(&entry);
        self.trace.borrow_mut().push(entry);
        if table != self.table || (self.cached_functional && repeated) {
            return Ok(Value::Undefined);
        }
        if self.fail_lookup {
            return Err("original lookup marker");
        }
        Ok((self.value)(self.trace.clone()))
    }
    fn key_code(&self) -> Result<f64, Self::Error> {
        Ok(0.0)
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        Ok(Cow::Borrowed(&[112, 114, 111, 98, 101]))
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        Ok(Cow::Borrowed(&[]))
    }
    fn shift_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn ctrl_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn alt_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn meta_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

#[test]
fn each_raw_map_uses_default_hint_and_preserves_surrogates() {
    for (table, expected) in [
        (Table::Letter, vec![27, 91, 0xd800]),
        (Table::Ss3, vec![27, 79, 0xd800]),
        (Table::Tilde, vec![27, 91, 0xd800, 126]),
        (Table::Functional, vec![27, 91, 0xd800, 117]),
    ] {
        let event = Event::new(table);
        let result = kitty_keyboard::try_evaluate(&event, 8, EventType::Press, false).unwrap();
        assert!(result.cancel);
        assert_eq!(result.key, Some(expected));
        assert_eq!(
            event.trace.borrow().last().map(String::as_str),
            Some("convert:default")
        );
    }
}

#[test]
fn functional_value_survives_the_later_lookup_before_conversion() {
    let event = Event {
        cached_functional: true,
        ..Event::new(Table::Functional)
    };
    let result = kitty_keyboard::try_evaluate(&event, 8, EventType::Press, false).unwrap();
    assert_eq!(result.key, Some(vec![27, 91, 0xd800, 117]));
    assert_eq!(
        *event.trace.borrow(),
        [
            "lookup:Letter",
            "lookup:Ss3",
            "lookup:Tilde",
            "lookup:Functional",
            "lookup:Functional",
            "convert:default"
        ]
    );
}

#[test]
fn special_key_equality_does_not_coerce_a_host_object() {
    let primitive = Event {
        value: |_| Value::Number(13.0),
        ..Event::new(Table::Functional)
    };
    let result = kitty_keyboard::try_evaluate(&primitive, 2, EventType::Release, false).unwrap();
    assert!(!result.cancel);
    assert_eq!(result.key, None);
    let object = Event::new(Table::Functional);
    let result = kitty_keyboard::try_evaluate(&object, 2, EventType::Release, false).unwrap();
    assert!(result.cancel);
    assert_eq!(result.key, Some(vec![27, 91, 0xd800, 59, 49, 58, 51, 117]));
}

#[test]
fn failures_preserve_the_original_lookup_or_conversion_marker() {
    for table in [Table::Letter, Table::Ss3, Table::Tilde, Table::Functional] {
        let event = Event {
            fail_lookup: true,
            ..Event::new(table)
        };
        assert_eq!(
            kitty_keyboard::try_evaluate(&event, 8, EventType::Press, false).unwrap_err(),
            "original lookup marker"
        );
        assert!(!event.trace.borrow().iter().any(|s| s == "convert:default"));
        let event = Event {
            value: |trace| Value::Host(Box::new(Raw { trace, fail: true })),
            ..Event::new(table)
        };
        assert_eq!(
            kitty_keyboard::try_evaluate(&event, 8, EventType::Press, false).unwrap_err(),
            "original conversion marker"
        );
    }
}
