//! Raw keypress values must not be coerced before source guards.
#[path = "support/keyboard_dispatch.rs"]
mod support;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};
use support::Host;
use xterm_core::input::keyboard_event_source::KeyboardEventSource;
use xterm_web::input::keyboard_dispatch::*;

type Number = EventNumber<&'static str>;
struct RawEvent {
    codes: RefCell<Vec<Number>>,
    which: Number,
    fallback: Number,
    alt: bool,
}
impl RawEvent {
    fn new(codes: Vec<Number>) -> Self {
        Self {
            codes: RefCell::new(codes),
            which: Number::Null,
            fallback: Number::Number(66.0),
            alt: false,
        }
    }
}
impl KeyboardEventSource for RawEvent {
    type Error = &'static str;
    fn key_code(&self) -> Result<f64, Self::Error> {
        Err("numeric encoder keyCode must not be read")
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        Ok(Cow::Borrowed(&[]))
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        Ok(Cow::Borrowed(&[]))
    }
    fn shift_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn alt_key(&self) -> Result<bool, Self::Error> {
        Ok(self.alt)
    }
    fn ctrl_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn meta_key(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
impl KeyEventSource for RawEvent {
    fn repeat(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn char_code(&self) -> Result<Number, Self::Error> {
        if self.codes.borrow().is_empty() {
            return Err("extra charCode read");
        }
        Ok(self.codes.borrow_mut().remove(0))
    }
    fn which(&self) -> Result<Number, Self::Error> {
        Ok(self.which.clone())
    }
    fn key_press_code(&self) -> Result<Number, Self::Error> {
        Ok(self.fallback.clone())
    }
    fn is_key_press(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }
    fn alt_graph(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
fn raw(truthy: bool, convert: impl Fn() -> Result<f64, &'static str> + 'static) -> Number {
    Number::Other {
        truthy,
        convert: Rc::new(convert),
    }
}
fn dead(dispatch: &KeyboardDispatch, host: &Host) {
    support::row(
        dispatch,
        host,
        "D\t0\t44,65,61,64\t\t0\t0\t-\t-\t1536\t0\t24\t0\t0\t\t0",
    )
    .unwrap();
    assert!(dispatch.state().unprocessed_dead_key);
    host.actions.borrow_mut().clear();
}

#[test]
fn string_zero_is_truthy_and_converts_to_nul_after_admission() {
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let value = raw(true, move || {
        seen.set(seen.get() + 1);
        Ok(0.0)
    });
    assert!(value.truthy());
    assert!(!value.is_zero());
    let event = RawEvent::new(vec![value.clone(), value]);
    let host = Host::default();
    assert!(KeyboardDispatch::default()
        .key_press_source(&event, &host)
        .unwrap());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        *host.actions.borrow(),
        ["custom", "key:0", "show", "data:1:0"]
    );
}

#[test]
fn modifier_guard_does_not_convert_a_truthy_raw_payload() {
    let value = raw(true, || Err("must not convert"));
    let mut event = RawEvent::new(vec![value.clone(), value]);
    event.alt = true;
    let host = Host::default();
    assert!(!KeyboardDispatch::default()
        .key_press_source(&event, &host)
        .unwrap());
    assert_eq!(*host.actions.borrow(), ["custom"]);
}

#[test]
fn changed_second_read_is_retested_for_truthiness_without_using_first_value() {
    let event = RawEvent::new(vec![
        raw(true, || Err("first read must not convert")),
        Number::Number(0.0),
    ]);
    let host = Host::default();
    assert!(!KeyboardDispatch::default()
        .key_press_source(&event, &host)
        .unwrap());
    assert_eq!(*host.actions.borrow(), ["custom"]);
}

#[test]
fn null_which_fallback_retains_raw_keycode_until_conversion() {
    let mut event = RawEvent::new(vec![Number::Number(0.0)]);
    event.fallback = raw(true, || Ok(66.0));
    let host = Host::default();
    assert!(KeyboardDispatch::default()
        .key_press_source(&event, &host)
        .unwrap());
    assert_eq!(
        *host.actions.borrow(),
        ["custom", "key:42", "show", "data:1:42"]
    );
}

#[test]
fn conversion_throw_precedes_key_effects_and_keeps_dead_key_state() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    dead(&dispatch, &host);
    let value = raw(true, || Err("conversion identity"));
    let event = RawEvent::new(vec![value.clone(), value]);
    assert_eq!(
        dispatch.key_press_source(&event, &host),
        Err("conversion identity")
    );
    assert!(dispatch.state().unprocessed_dead_key);
    assert!(!dispatch.state().key_press_handled);
    assert_eq!(*host.actions.borrow(), ["custom"]);
}

#[test]
fn conversion_can_reenter_input_before_outer_key_and_data_effects() {
    let dispatch = Rc::new(KeyboardDispatch::default());
    let host = Rc::new(Host::default());
    dead(&dispatch, &host);
    let nested = dispatch.clone();
    let nested_host = host.clone();
    let value = raw(true, move || {
        assert!(!nested.state().key_press_handled);
        assert!(nested.state().unprocessed_dead_key);
        nested.input(
            TextInputEvent {
                data: Some(&[110]),
                insert_text: true,
                composed: false,
            },
            nested_host.as_ref(),
        )?;
        Ok(65.0)
    });
    let event = RawEvent::new(vec![value.clone(), value]);
    assert!(dispatch.key_press_source(&event, host.as_ref()).unwrap());
    assert_eq!(
        *host.actions.borrow(),
        ["custom", "data:1:6e", "key:41", "show", "data:1:41"]
    );
    assert!(dispatch.state().key_press_handled);
    assert!(!dispatch.state().unprocessed_dead_key);
}
