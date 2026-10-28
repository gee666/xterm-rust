// Licensed under MIT. Reentrant and failing host callback contracts.
#[path = "support/keyboard_dispatch.rs"]
mod support;
use std::cell::Cell;
use support::Host;
use xterm_core::input::keyboard::KeyboardEvent;
use xterm_web::input::keyboard_dispatch::*;
// Test scaffolding repeats the small event literal to keep callback fixtures local.
fn event(key: &[u16], key_code: u32, kind: KeyEventKind) -> DispatchKeyEvent<'_> {
    DispatchKeyEvent {
        key: KeyboardEvent {
            key,
            key_code,
            ..KeyboardEvent::default()
        },
        kind,
        repeat: false,
        alt_graph: false,
        char_code: None,
        which: None,
    }
}
fn log(host: &Host) -> Vec<String> {
    host.actions.borrow().clone()
}
fn has_data(host: &Host) -> bool {
    log(host).iter().any(|s| s.starts_with("data:"))
}
struct CallbackHost<'a> {
    base: Host,
    dispatch: &'a KeyboardDispatch,
    fail_key: bool,
    fail_data: bool,
    reenter: Cell<bool>,
}
impl KeyboardReadHost for CallbackHost<'_> {
    type Error = &'static str;
    fn snapshot(&self) -> Result<KeyboardSnapshot, Self::Error> {
        self.base.snapshot()
    }
}
impl KeyboardHost for CallbackHost<'_> {
    fn custom_key_handler(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        self.base.custom_key_handler(event)
    }
    fn composition_key_down(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        self.base.composition_key_down(event)
    }
    fn perform(
        &self,
        event: Option<&dyn KeyEventSource<Error = Self::Error>>,
        action: KeyboardAction<'_>,
    ) -> Result<(), Self::Error> {
        self.base.perform(event, action)?;
        if self.fail_data && matches!(action, KeyboardAction::Data { .. }) {
            return Err("data failed");
        }
        if matches!(action, KeyboardAction::OnKey(_)) {
            if self.fail_key {
                return Err("onKey failed");
            }
            if self.reenter.replace(false) {
                self.dispatch.key_up(&super_event(), self)?;
                self.base.snapshot.set(KeyboardSnapshot {
                    screen_reader_mode: true,
                    ..self.base.snapshot.get()
                });
            }
        }
        Ok(())
    }
}
fn super_event() -> DispatchKeyEvent<'static> {
    event(&[97], 65, KeyEventKind::Up)
}

#[test]
fn callback_error_stops_effects_and_reentry_observes_live_state() {
    for fail_key in [false, true] {
        let dispatch = KeyboardDispatch::default();
        let host = CallbackHost {
            base: Host::default(),
            dispatch: &dispatch,
            fail_key,
            fail_data: false,
            reenter: Cell::new(true),
        };
        let result = dispatch.key_down(&event(&[97], 65, KeyEventKind::Down), &host);
        if fail_key {
            assert_eq!(result, Err("onKey failed"));
            assert!(!has_data(&host.base));
            assert!(!log(&host.base).contains(&"show".into()));
            assert!(!dispatch.state().key_down_handled);
        } else {
            assert_eq!(result, Ok(None));
            assert!(
                !dispatch.state().key_down_seen,
                "nested keyup commits before outer completion"
            );
            assert!(
                dispatch.state().key_down_handled,
                "live screen-reader option changed in callback"
            );
            assert_eq!(
                log(&host.base),
                [
                    "custom",
                    "composition",
                    "cursor",
                    "key:61",
                    "custom",
                    "focus",
                    "cursor",
                    "show",
                    "data:1:61"
                ]
            );
        }
    }
}

#[test]
fn input_and_keypress_commit_dead_state_on_opposite_sides_of_data_callback() {
    let dispatch = KeyboardDispatch::default();
    let host = CallbackHost {
        base: Host::default(),
        dispatch: &dispatch,
        fail_key: false,
        fail_data: true,
        reenter: Cell::new(false),
    };
    let dead: Vec<_> = "Dead".encode_utf16().collect();
    dispatch
        .key_down(&event(&dead, 0, KeyEventKind::Down), &host)
        .unwrap();
    assert!(dispatch.state().unprocessed_dead_key);
    assert_eq!(
        dispatch.key_press(&event(&[97], 97, KeyEventKind::Press), &host),
        Err("data failed")
    );
    assert!(dispatch.state().unprocessed_dead_key);
    assert!(!dispatch.state().key_press_handled);
    assert_eq!(
        dispatch.input(
            TextInputEvent {
                data: Some(&[97]),
                insert_text: true,
                composed: false
            },
            &host
        ),
        Err("data failed")
    );
    assert!(!dispatch.state().unprocessed_dead_key);
}

#[test]
fn shared_trace_adapter_after_callback_sequence() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    assert!(support::row(
        &dispatch,
        &host,
        "U\t65\t61\t\t0\t0\t-\t-\t1536\t0\t24\t0\t0\t-\t0"
    )
    .unwrap()
    .contains("focus|cursor"));
}
