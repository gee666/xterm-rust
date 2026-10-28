use std::rc::Rc;
use xterm_core::{
    headless::Terminal,
    options::OptionValue,
    public_api::addon_manager::TerminalAddon,
    serialize::{AddonSerializeError, HtmlOptions, SerializeAddon, SerializeOptions},
};
#[path = "support/serialize_live_snapshot.rs"]
mod snapshot;
fn terminal(cols: usize, rows: usize) -> Terminal {
    let t = Terminal::new(cols, rows, 20).unwrap();
    t.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    t
}
fn addon(t: &Terminal) -> Rc<SerializeAddon> {
    let a = Rc::new(SerializeAddon::new());
    t.load_addon(a.clone()).unwrap();
    a
}
fn text(units: Vec<u16>) -> String {
    String::from_utf16(&units).unwrap()
}

#[test]
fn activation_is_required_and_live_writes_are_never_cached() {
    let a = SerializeAddon::new();
    assert!(matches!(
        a.serialize(&SerializeOptions::default()),
        Err(AddonSerializeError::NotLoaded)
    ));
    assert!(matches!(
        a.serialize_as_html(&HtmlOptions::default()),
        Err(AddonSerializeError::NotLoaded)
    ));
    let t = terminal(10, 2);
    a.activate(&t).unwrap();
    assert_eq!(
        a.serialize(&SerializeOptions::default()).unwrap(),
        Vec::<u16>::new()
    );
    t.write("first").unwrap();
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "first"
    );
    t.write("\rnext").unwrap();
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "nextt\x1b[1D"
    );
}
#[test]
fn reactivation_dispose_and_manager_handle_preserve_source_behavior() {
    let one = terminal(10, 2);
    let two = terminal(10, 2);
    one.write("one").unwrap();
    two.write("two").unwrap();
    let a = Rc::new(SerializeAddon::new());
    let handle = one.load_addon(a.clone()).unwrap();
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "one"
    );
    a.activate(&two).unwrap();
    handle.dispose().unwrap();
    assert!(handle.is_disposed());
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "two"
    );
    a.dispose().unwrap();
    two.write("!").unwrap();
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "two!"
    );
}
#[test]
fn proposed_api_gate_is_live_and_borrow_failures_are_recoverable() {
    let t = Terminal::new(10, 2, 20).unwrap();
    let a = addon(&t);
    assert!(matches!(
        a.serialize(&SerializeOptions::default()),
        Err(AddonSerializeError::Api(_))
    ));
    t.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    assert!(a.serialize(&SerializeOptions::default()).is_ok());
    let state = t.state_handle_for_addon();
    let guard = state.borrow_mut();
    assert!(matches!(
        a.serialize(&SerializeOptions::default()),
        Err(AddonSerializeError::Borrowed)
    ));
    drop(guard);
    t.write("x").unwrap();
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "x"
    );
}
#[test]
fn original_cursor_range_and_underline_assertions_use_real_rust_writes() {
    for (input, expected) in [
        ("\x1b[32m> \x1b[0m", "\x1b[32m> \x1b[0m"),
        ("\x1b[4:1mtest\x1b[24m", "\x1b[4mtest\x1b[0m"),
        ("\x1b[4:2mtest\x1b[24m", "\x1b[4:2mtest\x1b[0m"),
        ("\x1b[4:3mtest\x1b[24m", "\x1b[4:3mtest\x1b[0m"),
        ("\x1b[4:4mtest\x1b[24m", "\x1b[4:4mtest\x1b[0m"),
        ("\x1b[4:5mtest\x1b[24m", "\x1b[4:5mtest\x1b[0m"),
    ] {
        let t = terminal(10, 2);
        let a = addon(&t);
        t.write(input).unwrap();
        assert_eq!(
            text(a.serialize(&SerializeOptions::default()).unwrap()),
            expected,
            "{input:?}"
        );
    }
    let t = terminal(10, 2);
    let a = addon(&t);
    t.write("hello\r\nworld").unwrap();
    for (start, end, expected) in [(0, 0, "hello"), (0, 1, "hello\r\nworld"), (1, 1, "world")] {
        assert_eq!(
            text(
                a.serialize(&SerializeOptions {
                    range: Some(xterm_core::serialize::LineRange { start, end }),
                    ..SerializeOptions::default()
                })
                .unwrap()
            ),
            expected
        );
    }
}
#[test]
fn real_terminal_round_trips_match_source_buffers_cells_attributes_cursor_and_modes() {
    let cases: Vec<_> = include_str!("fixtures/serialize-live.tsv")
        .lines()
        .collect();
    assert_eq!(cases.len(), 10);
    let mut losses = 0;
    for row in cases {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 8);
        let units = |value: &str| {
            value
                .split(',')
                .map(|unit| u16::from_str_radix(unit, 16).unwrap())
                .collect::<Vec<_>>()
        };
        let number = |index: usize| fields[index].parse::<usize>().unwrap();
        let input = units(fields[1]);
        let expected_output = units(fields[2]);
        let t = terminal(10, 5);
        let a = addon(&t);
        t.write_utf16(&input).unwrap();
        let before = snapshot::terminal(&t);
        assert_eq!(
            before.normal_cursor(),
            [number(3), number(4)],
            "source pre-cursor case{}",
            fields[0]
        );
        let output = a.serialize(&SerializeOptions::default()).unwrap();
        assert_eq!(
            output, expected_output,
            "source exact ANSI case{}",
            fields[0]
        );
        let restored = terminal(10, 5);
        restored.write_utf16(&output).unwrap();
        // Source's final DECSTBM homes the cursor. The oracle asserts this is
        // the only changed observed field before recording the expected cursor.
        let expected = before.with_normal_cursor(number(5), number(6));
        assert_eq!(
            snapshot::terminal(&restored),
            expected,
            "source round-trip case{}",
            fields[0]
        );
        if fields[7] == "0" {
            losses += 1;
            assert_eq!(fields[0], "7");
            assert_eq!([number(3), number(4), number(5), number(6)], [8, 2, 0, 0]);
        }
    }
    assert_eq!(
        losses, 1,
        "retain the source's margin/cursor loss instead of claiming every round-trip is lossless"
    );
}
#[test]
fn live_resize_reset_and_scrollback_options_use_current_terminal_state() {
    let t = terminal(10, 2);
    let a = addon(&t);
    t.write("0\r\n1\r\n2\r\n3\r\n4").unwrap();
    let viewport = a
        .serialize(&SerializeOptions {
            scrollback: Some(0),
            ..SerializeOptions::default()
        })
        .unwrap();
    assert_eq!(text(viewport), "3\r\n4");
    t.resize(12, 3).unwrap();
    let state = t.state_handle_for_addon();
    assert_eq!(state.borrow().cols(), 12);
    assert_eq!(state.borrow().rows(), 3);
    t.reset().unwrap();
    assert_eq!(
        a.serialize(&SerializeOptions::default()).unwrap(),
        Vec::<u16>::new()
    );
    t.write("fresh").unwrap();
    assert_eq!(
        text(a.serialize(&SerializeOptions::default()).unwrap()),
        "fresh"
    );
}

#[test]
fn original_underline_colors_margins_and_cursor_visibility_assertions_use_live_engine() {
    for (sgr, color) in [
        ("4;58;2;255;128;64", "58:2::255:128:64"),
        ("4;58;5;46", "58:5:46"),
    ] {
        let t = terminal(10, 2);
        let a = addon(&t);
        t.write(&format!("\x1b[{sgr}mtest\x1b[24m")).unwrap();
        let output = text(a.serialize(&SerializeOptions::default()).unwrap());
        assert!(output.contains("4:1"), "{output}");
        assert!(output.contains(color), "{output}");
    }
    let t = terminal(10, 5);
    let a = addon(&t);
    t.write("\x1b[2;4r").unwrap();
    t.with_state(|state| {
        let active = state.buffers.active();
        let b = active.borrow();
        assert_eq!(b.scroll_top, 1);
        assert_eq!(b.scroll_bottom, 3);
    })
    .unwrap();
    let output = a.serialize(&SerializeOptions::default()).unwrap();
    assert!(text(output.clone()).contains("\x1b[2;4r"));
    assert!(!text(
        a.serialize(&SerializeOptions {
            exclude_modes: true,
            ..SerializeOptions::default()
        })
        .unwrap()
    )
    .contains("\x1b[2;4r"));
    let restored = terminal(10, 5);
    restored.write_utf16(&output).unwrap();
    restored
        .with_state(|state| {
            let active = state.buffers.active();
            let b = active.borrow();
            assert_eq!(b.scroll_top, 1);
            assert_eq!(b.scroll_bottom, 3);
        })
        .unwrap();
    let t = terminal(10, 2);
    let a = addon(&t);
    t.write("hello").unwrap();
    assert!(t.modes().unwrap().show_cursor);
    let visible = text(a.serialize(&SerializeOptions::default()).unwrap());
    assert!(!visible.contains("\x1b[?25l"));
    assert!(!visible.contains("\x1b[?25h"));
    t.write("\x1b[?25l").unwrap();
    assert!(!t.modes().unwrap().show_cursor);
    let output = a.serialize(&SerializeOptions::default()).unwrap();
    assert!(text(output.clone()).contains("\x1b[?25l"));
    assert!(!text(
        a.serialize(&SerializeOptions {
            exclude_modes: true,
            ..SerializeOptions::default()
        })
        .unwrap()
    )
    .contains("\x1b[?25l"));
    let restored = terminal(10, 2);
    restored.write_utf16(&output).unwrap();
    assert!(!restored.modes().unwrap().show_cursor);
}

#[test]
fn native_html_reads_current_font_theme_and_active_buffer() {
    let t = terminal(10, 2);
    let a = addon(&t);
    t.write("\x1b[1mhello\x1b[0m").unwrap();
    let output = text(a.serialize_as_html(&HtmlOptions::default()).unwrap());
    assert!(output.contains("font-weight: bold;"));
    assert!(output.contains("hello"));
    t.options()
        .set("fontFamily", OptionValue::from("verdana"))
        .unwrap();
    t.options()
        .set("fontSize", OptionValue::Number(20.0))
        .unwrap();
    t.options()
        .set(
            "theme",
            OptionValue::property_map([
                ("foreground", OptionValue::from("#ff00ff")),
                ("background", OptionValue::from("#00ff00")),
            ]),
        )
        .unwrap();
    let output = text(
        a.serialize_as_html(&HtmlOptions {
            include_global_background: true,
            ..HtmlOptions::default()
        })
        .unwrap(),
    );
    assert!(output.contains(
        "color: #ff00ff; background-color: #00ff00; font-family: verdana; font-size: 20px;"
    ));
    assert!(matches!(
        a.serialize_as_html(&HtmlOptions {
            only_selection: true,
            ..HtmlOptions::default()
        }),
        Err(AddonSerializeError::SelectionUnavailable)
    ));
}
