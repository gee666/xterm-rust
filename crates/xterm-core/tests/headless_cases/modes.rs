use super::support::*;
use xterm_core::headless::{ModesSnapshot, MouseTrackingMode};
#[test]
fn defaults() {
    let term = terminal(80, 24);
    assert_eq!(
        term.modes().unwrap(),
        ModesSnapshot {
            application_cursor_keys_mode: false,
            application_keypad_mode: false,
            bracketed_paste_mode: false,
            insert_mode: false,
            mouse_tracking_mode: MouseTrackingMode::None,
            origin_mode: false,
            reverse_wraparound_mode: false,
            send_focus_mode: false,
            show_cursor: true,
            synchronized_output_mode: false,
            win32_input_mode: false,
            wraparound_mode: true,
        }
    );
}
fn check(sequence: u32, get: fn(&ModesSnapshot) -> bool) {
    let term = terminal(80, 24);
    term.write(&format!("\x1b[?{sequence}h")).unwrap();
    assert!(get(&term.modes().unwrap()));
    term.write(&format!("\x1b[?{sequence}l")).unwrap();
    assert!(!get(&term.modes().unwrap()));
}
#[test]
fn application_cursor_keys_mode() {
    check(1, |m| m.application_cursor_keys_mode);
}
#[test]
fn application_keypad_mode() {
    check(66, |m| m.application_keypad_mode);
}
#[test]
fn bracketed_paste_mode() {
    check(2004, |m| m.bracketed_paste_mode);
}
#[test]
fn insert_mode() {
    let term = terminal(80, 24);
    term.write("\x1b[4h").unwrap();
    assert!(term.modes().unwrap().insert_mode);
    term.write("\x1b[4l").unwrap();
    assert!(!term.modes().unwrap().insert_mode);
}
#[test]
fn mouse_tracking_mode() {
    let term = terminal(80, 24);
    for (mode, expected) in [
        (9, MouseTrackingMode::X10),
        (1000, MouseTrackingMode::Vt200),
        (1002, MouseTrackingMode::Drag),
        (1003, MouseTrackingMode::Any),
    ] {
        term.write(&format!("\x1b[?{mode}h")).unwrap();
        assert_eq!(term.modes().unwrap().mouse_tracking_mode, expected);
        term.write(&format!("\x1b[?{mode}l")).unwrap();
        assert_eq!(
            term.modes().unwrap().mouse_tracking_mode,
            MouseTrackingMode::None
        );
    }
}
#[test]
fn origin_mode() {
    check(6, |m| m.origin_mode);
}
#[test]
fn reverse_wraparound_mode() {
    check(45, |m| m.reverse_wraparound_mode);
}
#[test]
fn send_focus_mode() {
    check(1004, |m| m.send_focus_mode);
}
#[test]
fn wraparound_mode() {
    check(7, |m| m.wraparound_mode);
}
