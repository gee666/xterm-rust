//! Actual render/buffer events must drive the same Viewport as user scrolling.
use super::tests::Fixture;
use wasm_bindgen_test::*;

fn write(fixture: &Fixture, text: &str) {
    fixture
        .terminal
        .shared
        .native
        .write_utf16(&text.encode_utf16().collect::<Vec<_>>())
        .unwrap();
}
fn fill(fixture: &Fixture) {
    write(fixture, "a\r\nb\r\nc\r\nd\r\ne\r\nf");
    fixture.viewport().sync(None).unwrap();
    assert_eq!(fixture.row(), 3);
}

#[wasm_bindgen_test]
fn synchronized_output_defers_external_scroll_until_real_render_completion() {
    let fixture = Fixture::new();
    fill(&fixture);
    let viewport = fixture.viewport();
    let element = viewport.host.element().unwrap();
    let height = fixture
        .services()
        .render_service
        .dimensions()
        .unwrap()
        .css
        .cell
        .height;
    assert!(height > 0.0);
    assert_eq!(element.get_scroll_position().top, height * 3.0);
    write(&fixture, "\x1b[?2026h");
    fixture.terminal.shared.native.scroll_lines(-2).unwrap();
    assert_eq!(fixture.row(), 1);
    assert_eq!(element.get_scroll_position().top, height * 3.0);
    fixture.services().render_rows(0.0, 2.0).unwrap();
    assert_eq!(element.get_scroll_position().top, height * 3.0);
    write(&fixture, "\x1b[?2026l");
    fixture.services().render_rows(0.0, 2.0).unwrap();
    assert_eq!(element.get_scroll_position().top, height);
}

#[wasm_bindgen_test]
fn alternate_activation_and_resize_use_live_buffer_and_renderer_dimensions() {
    let fixture = Fixture::new();
    fill(&fixture);
    let viewport = fixture.viewport();
    let element = viewport.host.element().unwrap();
    let height = fixture
        .services()
        .render_service
        .dimensions()
        .unwrap()
        .css
        .cell
        .height;
    assert!(height > 0.0);
    write(&fixture, "\x1b[?1049h");
    viewport.sync(None).unwrap();
    assert_eq!(fixture.row(), 0);
    assert_eq!(element.get_scroll_position().top, 0.0);
    write(&fixture, "\x1b[?1049l");
    viewport.sync(None).unwrap();
    assert_eq!(fixture.row(), 3);
    assert_eq!(element.get_scroll_position().top, height * 3.0);
    fixture.terminal.resize(8.0, 4.0).unwrap();
    viewport.sync(None).unwrap();
    let dimensions = element.get_scroll_dimensions();
    assert_eq!(dimensions.height, height * 4.0);
    assert_eq!(dimensions.scroll_height, height * 6.0);
    assert_eq!(fixture.row(), 2);
    assert_eq!(element.get_scroll_position().top, height * 2.0);
}
