//! Public wrappers share Viewport routing only after open has constructed it.
use super::tests::Fixture;
use crate::terminal::Terminal;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

fn populate(terminal: &Terminal) {
    terminal
        .shared
        .native
        .write_utf16(
            &"a\r\nb\r\nc\r\nd\r\ne\r\nf"
                .encode_utf16()
                .collect::<Vec<_>>(),
        )
        .unwrap();
}

fn row(terminal: &Terminal) -> usize {
    terminal
        .shared
        .native
        .with_state(|state| state.active_buffer().borrow().ydisp)
        .unwrap()
}

#[wasm_bindgen_test]
fn unopened_public_scroll_wrappers_keep_native_clamping_and_validation() {
    let options = js_sys::JSON::parse(r#"{"cols":8,"rows":3,"scrollback":20}"#).unwrap();
    let terminal = Terminal::new(options).unwrap();
    populate(&terminal);
    assert!(terminal.shared.services.borrow().is_none());
    assert_eq!(row(&terminal), 3);
    terminal.scroll_lines(-1.0).unwrap();
    assert_eq!(row(&terminal), 2);
    terminal.scroll_pages(-1.0).unwrap();
    assert_eq!(row(&terminal), 0);
    terminal.scroll_to_bottom().unwrap();
    assert_eq!(row(&terminal), 3);
    terminal.scroll_to_top().unwrap();
    assert_eq!(row(&terminal), 0);
    terminal.scroll_to_line(2.0).unwrap();
    assert_eq!(row(&terminal), 2);
    terminal.scroll_to_line(f64::MAX).unwrap();
    assert_eq!(row(&terminal), 3);
    terminal.scroll_lines(-f64::MAX).unwrap();
    assert_eq!(row(&terminal), 0);
    for invalid in [0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(terminal.scroll_lines(invalid).is_err());
        assert!(terminal.scroll_pages(invalid).is_err());
        assert!(terminal.scroll_to_line(invalid).is_err());
        assert_eq!(row(&terminal), 0);
    }
    terminal.dispose().unwrap();
}

#[wasm_bindgen_test]
fn opened_scroll_lines_retains_pixels_and_zero_delta_leaves_position_unchanged() {
    let fixture = Fixture::new();
    populate(&fixture.terminal);
    let viewport = fixture.viewport();
    viewport.sync(None).unwrap();
    let height = fixture
        .services()
        .render_service
        .dimensions()
        .unwrap()
        .css
        .cell
        .height;
    fixture.terminal.scroll_to_top().unwrap();
    viewport.handle_touch_scroll(-height * 0.6).unwrap();
    assert_eq!(fixture.row(), 1);
    fixture.terminal.scroll_lines(1.0).unwrap();
    let expected = height * 0.6 + height;
    assert_eq!(fixture.row(), 2);
    let element = viewport.host.element().unwrap();
    assert_eq!(element.get_scroll_position().top, expected);
    fixture.terminal.scroll_to_line(2.0).unwrap();
    assert_eq!(element.get_scroll_position().top, expected);
    fixture.terminal.scroll_lines(0.0).unwrap();
    assert_eq!(element.get_scroll_position().top, expected);
    fixture.terminal.scroll_lines(-1.0).unwrap();
    assert_eq!(fixture.row(), 1);
    assert_eq!(element.get_scroll_position().top, expected - height);
    let invalid = fixture.terminal.scroll_lines(0.5).unwrap_err();
    assert!(invalid.is_instance_of::<js_sys::Error>());
    assert_eq!(element.get_scroll_position().top, expected - height);
}
