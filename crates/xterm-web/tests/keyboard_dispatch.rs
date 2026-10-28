// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. CoreBrowserTerminal source paths and callback ordering.
#[path = "support/keyboard_dispatch.rs"]
mod support;

use support::Host;
use xterm_core::input::keyboard::KeyboardEvent;
use xterm_web::input::keyboard_dispatch::*;

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

#[test]
fn source_custom_handler_veto_precedes_composition_and_all_effects() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    let down = event(&[], 77, KeyEventKind::Down);
    let press = event(&[], 77, KeyEventKind::Press);
    // Terminal.test.ts attachCustomKeyEventHandler's six original returns:
    // no handler, accepting handler, rejecting handler, for down and press.
    host.has_custom.set(false);
    assert_eq!(dispatch.key_down(&down, &host), Ok(Some(true)));
    assert_eq!(dispatch.key_press(&press, &host), Ok(true));
    host.has_custom.set(true);
    assert_eq!(dispatch.key_down(&down, &host), Ok(Some(true)));
    assert_eq!(dispatch.key_press(&press, &host), Ok(true));
    host.accepted.set(false);
    host.actions.borrow_mut().clear();
    assert_eq!(dispatch.key_down(&down, &host), Ok(Some(false)));
    assert_eq!(log(&host), ["custom"]);
    assert!(dispatch.state().key_down_seen);
    assert!(!dispatch.state().key_down_handled);
    host.actions.borrow_mut().clear();
    assert_eq!(dispatch.key_press(&press, &host), Ok(false));
    assert_eq!(log(&host), ["custom"]);
}

#[test]
fn screen_reader_handled_state_suppresses_following_keypress() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    host.snapshot.set(KeyboardSnapshot {
        screen_reader_mode: true,
        ..host.snapshot.get()
    });
    assert_eq!(
        dispatch.key_down(&event(&[97], 65, KeyEventKind::Down), &host),
        Ok(None)
    );
    assert_eq!(
        log(&host),
        [
            "custom",
            "composition",
            "cursor",
            "key:61",
            "show",
            "data:1:61"
        ]
    );
    assert!(dispatch.state().key_down_handled);
    host.actions.borrow_mut().clear();
    assert_eq!(
        dispatch.key_press(&event(&[97], 97, KeyEventKind::Press), &host),
        Ok(false)
    );
    assert!(log(&host).is_empty());
    assert_eq!(
        dispatch.input(
            TextInputEvent {
                data: Some(&[97]),
                insert_text: true,
                composed: false
            },
            &host
        ),
        Ok(false)
    );
}

#[test]
fn dead_key_and_uppercase_routes_preserve_partial_state() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    let dead: Vec<_> = "Dead".encode_utf16().collect();
    assert_eq!(
        dispatch.key_down(&event(&dead, 0, KeyEventKind::Down), &host),
        Ok(Some(true))
    );
    assert!(dispatch.state().unprocessed_dead_key);
    host.actions.borrow_mut().clear();
    assert_eq!(
        dispatch.key_down(&event(&[65], 65, KeyEventKind::Down), &host),
        Ok(Some(true))
    );
    assert!(
        dispatch.state().unprocessed_dead_key,
        "uppercase workaround precedes clearing dead key"
    );
    assert!(!has_data(&host));
    assert_eq!(
        dispatch.key_down(&event(&[97], 65, KeyEventKind::Down), &host),
        Ok(Some(true))
    );
    assert!(!dispatch.state().unprocessed_dead_key);
    assert!(!has_data(&host));
    assert_eq!(
        dispatch.key_press(&event(&[97], 97, KeyEventKind::Press), &host),
        Ok(true)
    );
    assert!(dispatch.state().key_press_handled);
    assert_eq!(
        dispatch.input(
            TextInputEvent {
                data: Some(&[97]),
                insert_text: true,
                composed: false
            },
            &host
        ),
        Ok(false)
    );
}

#[test]
fn enter_clears_textarea_before_key_and_can_cancel_twice() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    let enter: Vec<_> = "Enter".encode_utf16().collect();
    assert_eq!(
        dispatch.key_down(&event(&enter, 13, KeyEventKind::Down), &host),
        Ok(Some(false))
    );
    assert_eq!(
        log(&host),
        [
            "custom",
            "composition",
            "cursor",
            "cancel",
            "clear",
            "key:d",
            "show",
            "data:1:d",
            "cancel"
        ]
    );
}

#[test]
fn composition_rejection_scrolls_only_when_required_and_mac_option_bypasses_it() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    host.composition.set(false);
    let mut snapshot = host.snapshot.get();
    snapshot.scroll_on_user_input = true;
    snapshot.ybase = 40;
    host.snapshot.set(snapshot);
    assert_eq!(
        dispatch.key_down(&event(&[97], 65, KeyEventKind::Down), &host),
        Ok(Some(false))
    );
    assert_eq!(log(&host), ["custom", "composition", "bottom"]);
    snapshot.is_mac = true;
    snapshot.modes.is_mac = true;
    snapshot.modes.mac_option_is_meta = true;
    host.snapshot.set(snapshot);
    host.actions.borrow_mut().clear();
    let mut down = event(&[97], 65, KeyEventKind::Down);
    down.key.alt_key = true;
    dispatch.key_down(&down, &host).unwrap();
    assert!(!log(&host).contains(&"composition".into()));
    assert!(has_data(&host));
}

#[test]
fn source_mac_and_windows_altgraph_printable_keypress_cases() {
    for windows in [false, true] {
        let dispatch = KeyboardDispatch::default();
        let host = Host::default();
        let mut snapshot = host.snapshot.get();
        snapshot.is_mac = !windows;
        snapshot.modes.is_mac = !windows;
        snapshot.is_windows = windows;
        host.snapshot.set(snapshot);
        for code in [64, 92, 124] {
            for firefox in [false, true] {
                host.actions.borrow_mut().clear();
                let mut press = event(&[], if firefox { 0 } else { code }, KeyEventKind::Press);
                press.key.alt_key = true;
                press.key.ctrl_key = windows;
                press.char_code = firefox.then_some(code);
                assert_eq!(dispatch.key_press(&press, &host), Ok(true));
                assert!(log(&host).contains(&format!("key:{code:x}")));
            }
        }
    }
}

#[test]
fn charcode_which_zero_absence_and_utf16_modulo_are_not_interchangeable() {
    for (char_code, which, key_code, expected) in [
        (None, None, 13, Some(13)),
        (Some(0), None, 65, Some(65)),
        (None, Some(97), 0, Some(97)),
        (Some(0), Some(97), 0, None),
        (None, Some(0), 65, None),
        (Some(0xd800), Some(0), 0, Some(0xd800)),
        (Some(u32::MAX), None, 0, Some(0xffff)),
        (Some(65536), None, 0, Some(0)),
    ] {
        let dispatch = KeyboardDispatch::default();
        let host = Host::default();
        let mut press = event(&[], key_code, KeyEventKind::Press);
        press.char_code = char_code;
        press.which = which;
        assert_eq!(dispatch.key_press(&press, &host), Ok(expected.is_some()));
        if let Some(code) = expected {
            assert!(log(&host).contains(&format!("data:1:{code:x}")));
        } else {
            assert!(!has_data(&host));
        }
    }
}

#[test]
fn input_composed_seen_and_keyup_veto_order() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    host.accepted.set(false);
    dispatch
        .key_down(&event(&[], 0, KeyEventKind::Down), &host)
        .unwrap();
    let input = TextInputEvent {
        data: Some(&[0xd800, 0xde00]),
        insert_text: true,
        composed: true,
    };
    assert_eq!(dispatch.input(input, &host), Ok(false));
    host.actions.borrow_mut().clear();
    dispatch
        .key_up(&event(&[], 65, KeyEventKind::Up), &host)
        .unwrap();
    assert!(!dispatch.state().key_down_seen);
    assert_eq!(log(&host), ["custom"]);
    assert_eq!(dispatch.input(input, &host), Ok(true));
    assert!(log(&host).contains(&"data:1:d800,de00".into()));
}

#[test]
fn shared_trace_adapter_runs_ordered_enter() {
    let dispatch = KeyboardDispatch::default();
    let host = Host::default();
    assert_eq!(
        support::row(
            &dispatch,
            &host,
            "D\t13\t45,6e,74,65,72\t45,6e,74,65,72\t0\t0\t-\t-\t1536\t0\t24\t40\t0\t-\t0"
        )
        .unwrap(),
        "0\t2\tcustom|composition|cursor|cancel|clear|key:d|show|data:1:d|cancel"
    );
}
