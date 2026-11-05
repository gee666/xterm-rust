// Licensed under MIT. CoreBrowserTerminal._inputEvent getter and state ordering.
#[path = "support/keyboard_dispatch.rs"]
mod support;
use std::{borrow::Cow, cell::RefCell};
use support::Host;
use xterm_web::input::keyboard_dispatch::*;

type Read<'a, T> = Box<dyn Fn() -> Result<T, &'static str> + 'a>;
struct Input<'a> {
    reads: RefCell<Vec<&'static str>>,
    truthy: bool,
    insert: bool,
    composed: Read<'a, bool>,
    data: Read<'a, Vec<u16>>,
}
impl Default for Input<'_> {
    fn default() -> Self {
        Self {
            reads: RefCell::new(Vec::new()),
            truthy: true,
            insert: true,
            composed: Box::new(|| Ok(false)),
            data: Box::new(|| Ok(vec![0xd800, 0x61])),
        }
    }
}
impl InputEventSource<&'static str> for Input<'_> {
    fn has_data(&self) -> Result<bool, &'static str> {
        self.reads.borrow_mut().push("data guard");
        Ok(self.truthy)
    }
    fn insert_text(&self) -> Result<bool, &'static str> {
        self.reads.borrow_mut().push("inputType");
        Ok(self.insert)
    }
    fn composed(&self) -> Result<bool, &'static str> {
        self.reads.borrow_mut().push("composed");
        (self.composed)()
    }
}
impl TextInputEventSource<&'static str> for Input<'_> {
    fn data(&self) -> Result<Cow<'_, [u16]>, &'static str> {
        self.reads.borrow_mut().push("data payload");
        (self.data)().map(Cow::Owned)
    }
}

fn key(key: &'static [u16], kind: KeyEventKind) -> DispatchKeyEvent<'static> {
    DispatchKeyEvent {
        key: xterm_core::input::keyboard::KeyboardEvent {
            key,
            ..Default::default()
        },
        kind,
        repeat: false,
        alt_graph: false,
        char_code: Some(97),
        which: Some(97),
    }
}

#[test]
fn immutable_input_remains_compatible_with_the_shared_trace_adapter() {
    assert_eq!(
        support::row(
            &KeyboardDispatch::default(),
            &Host::default(),
            "I\t0\t\t\t0\t0\t-\t-\t1536\t0\t24\t0\t0\t61\t1"
        )
        .unwrap(),
        "1\t0\tdata:1:61"
    );
}

#[test]
fn rejected_input_short_circuits_before_unused_getters() {
    for (truthy, insert, expected) in [
        (false, true, vec!["data guard"]),
        (true, false, vec!["data guard", "inputType"]),
    ] {
        let input = Input {
            truthy,
            insert,
            composed: Box::new(|| Err("unused composed")),
            data: Box::new(|| Err("unused data")),
            ..Default::default()
        };
        let host = Host::default();
        assert!(!KeyboardDispatch::default()
            .input_source(&input, &host)
            .unwrap());
        assert_eq!(*input.reads.borrow(), expected);
        assert!(host.actions.borrow().is_empty());
    }
}

#[test]
fn admitted_input_rereads_payload_and_preserves_empty_or_unpaired_utf16() {
    for data in [vec![], vec![0xd800, 0x61]] {
        let input = Input {
            data: Box::new(|| Ok(data.clone())),
            ..Default::default()
        };
        let host = Host::default();
        assert!(KeyboardDispatch::default()
            .input_source(&input, &host)
            .unwrap());
        assert_eq!(
            *input.reads.borrow(),
            ["data guard", "inputType", "composed", "data payload"]
        );
        let hex = data
            .iter()
            .map(|unit| format!("{unit:x}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(*host.actions.borrow(), [format!("data:1:{hex}")]);
    }
}

#[test]
fn composed_getter_reentry_is_seen_before_keydown_guard() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    let input = Input {
        composed: Box::new(|| {
            dispatch.key_down(&key(&[97], KeyEventKind::Down), &host)?;
            Ok(true)
        }),
        data: Box::new(|| Err("suppressed payload")),
        ..Default::default()
    };
    assert!(!dispatch.input_source(&input, &host).unwrap());
    assert!(dispatch.state().key_down_seen);
    assert_eq!(
        *input.reads.borrow(),
        ["data guard", "inputType", "composed"]
    );
}

#[test]
fn duplicate_keypress_and_screen_reader_guard_do_not_reread_data() {
    for screen_reader_mode in [false, true] {
        let dispatch = KeyboardDispatch::default();
        let host = Host::default();
        if screen_reader_mode {
            host.snapshot.set(KeyboardSnapshot {
                screen_reader_mode,
                ..Default::default()
            });
        } else {
            assert!(dispatch
                .key_press(&key(&[97], KeyEventKind::Press), &host)
                .unwrap());
        }
        let input = Input {
            data: Box::new(|| Err("suppressed payload")),
            ..Default::default()
        };
        assert!(!dispatch.input_source(&input, &host).unwrap());
        assert_eq!(
            *input.reads.borrow(),
            ["data guard", "inputType", "composed"]
        );
    }
}

#[test]
fn dead_key_is_cleared_before_second_data_getter_even_if_it_throws() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    dispatch
        .key_down(&key(&[68, 101, 97, 100], KeyEventKind::Down), &host)
        .unwrap();
    assert!(dispatch.state().unprocessed_dead_key);
    let input = Input {
        data: Box::new(|| {
            assert!(!dispatch.state().unprocessed_dead_key);
            Err("second data getter")
        }),
        ..Default::default()
    };
    assert_eq!(
        dispatch.input_source(&input, &host),
        Err("second data getter")
    );
    assert!(!dispatch.state().unprocessed_dead_key);
}
