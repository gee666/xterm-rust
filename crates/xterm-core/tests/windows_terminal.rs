use xterm_core::{
    options::OptionValue,
    terminal::{CoreTerminal, TerminalEvent},
};
fn pty(backend: &str, build: OptionValue) -> OptionValue {
    OptionValue::property_map([
        ("backend", OptionValue::from(backend)),
        ("buildNumber", build),
    ])
}
fn wrapped(t: &CoreTerminal, row: usize) -> bool {
    let state = t.state();
    let buffer = state.active_buffer();
    let buffer = buffer.borrow();
    let line = buffer.lines.get(buffer.ybase + row).unwrap();
    let value = line.borrow().is_wrapped;
    value
}
fn set_pty(t: &CoreTerminal, value: OptionValue) {
    let options = t.state().options.clone();
    options.set("windowsPty", value).unwrap();
}
#[test]
fn conpty_build_boundaries_control_actual_linefeed_wrapping() {
    for (backend, build, enabled) in [
        ("conpty", OptionValue::Number(21375.0), true),
        ("conpty", OptionValue::Number(21376.0), false),
        ("winpty", OptionValue::Number(1.0), false),
        ("conpty", OptionValue::Undefined, false),
        ("conpty", OptionValue::Null, true),
        ("conpty", OptionValue::Number(f64::NAN), false),
        ("conpty", OptionValue::from("21375"), true),
    ] {
        let mut terminal = CoreTerminal::new(5, 3, 10).unwrap();
        set_pty(&terminal, pty(backend, build));
        terminal.write("abcde\n").unwrap();
        assert_eq!(wrapped(&terminal, 1), enabled, "{backend}");
        assert!(terminal.drain_events().contains(&TerminalEvent::LineFeed));
    }
}
#[test]
fn csi_h_updates_previous_location_before_cursor_position_changes() {
    let mut terminal = CoreTerminal::new(5, 3, 10).unwrap();
    terminal.write("abcde\x1b[2;1H").unwrap();
    assert!(!wrapped(&terminal, 1));
    set_pty(&terminal, pty("conpty", OptionValue::Number(21375.0)));
    terminal.write("\x1b[3;1H").unwrap();
    assert!(wrapped(&terminal, 1));
    assert!(!wrapped(&terminal, 2));
    let state = terminal.state();
    let buffer = state.active_buffer();
    assert_eq!(buffer.borrow().y, 2);
}
#[test]
fn option_disable_removes_lf_and_csi_h_heuristics_and_reenable_is_live() {
    let mut terminal = CoreTerminal::new(5, 3, 10).unwrap();
    set_pty(&terminal, pty("conpty", OptionValue::Number(21375.0)));
    set_pty(&terminal, OptionValue::Null);
    terminal.write("abcde\n").unwrap();
    assert!(!wrapped(&terminal, 1));
    terminal.write("\x1b[3;1H").unwrap();
    assert!(!wrapped(&terminal, 1));
    terminal.write("\x1b[2;1H").unwrap();
    set_pty(&terminal, pty("conpty", OptionValue::Number(21375.0)));
    terminal.write("\x1b[3;1H").unwrap();
    assert!(wrapped(&terminal, 1));
}
