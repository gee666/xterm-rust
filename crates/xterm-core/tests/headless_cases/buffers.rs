use super::support::*;
use std::rc::Rc;
use xterm_core::public_api::buffer::BufferType;
#[test]
fn cursor_x_cursor_y() {
    let term = terminal(5, 5);
    let buffer = term.buffer().unwrap().active().unwrap();
    assert_eq!(buffer.cursor_x().unwrap(), 0);
    assert_eq!(buffer.cursor_y().unwrap(), 0);
    for (text, x, y) in [
        ("foo", 3, 0),
        ("\n", 3, 1),
        ("\r", 0, 1),
        ("abcde", 5, 1),
        ("\n\r\n\n\n\n\n", 0, 4),
    ] {
        term.write(text).unwrap();
        assert_eq!(buffer.cursor_x().unwrap(), x);
        assert_eq!(buffer.cursor_y().unwrap(), y);
    }
}
#[test]
fn viewport_y() {
    let term = terminal(80, 5);
    let buffer = term.buffer().unwrap().active().unwrap();
    assert_eq!(buffer.viewport_y().unwrap(), 0);
    for (text, y) in [("\n\n\n\n", 0), ("\n", 1), ("\n\n\n\n", 5)] {
        term.write(text).unwrap();
        assert_eq!(buffer.viewport_y().unwrap(), y);
    }
    term.scroll_lines(-1).unwrap();
    assert_eq!(buffer.viewport_y().unwrap(), 4);
    term.scroll_to_top().unwrap();
    assert_eq!(buffer.viewport_y().unwrap(), 0);
}
#[test]
fn base_y() {
    let term = terminal(80, 5);
    let buffer = term.buffer().unwrap().active().unwrap();
    assert_eq!(buffer.base_y().unwrap(), 0);
    for (text, y) in [("\n\n\n\n", 0), ("\n", 1), ("\n\n\n\n", 5)] {
        term.write(text).unwrap();
        assert_eq!(buffer.base_y().unwrap(), y);
    }
    term.scroll_lines(-1).unwrap();
    assert_eq!(buffer.base_y().unwrap(), 5);
    term.scroll_to_top().unwrap();
    assert_eq!(buffer.base_y().unwrap(), 5);
}
#[test]
fn length() {
    let term = terminal(80, 5);
    let buffer = term.buffer().unwrap().active().unwrap();
    assert_eq!(buffer.len().unwrap(), 5);
    for (text, length) in [("\n\n\n\n", 5), ("\n", 6), ("\n\n\n\n", 10)] {
        term.write(text).unwrap();
        assert_eq!(buffer.len().unwrap(), length);
    }
}
#[test]
fn get_line_invalid_index() {
    let term = terminal(80, 5);
    let buffer = term.buffer().unwrap().active().unwrap();
    // Negative native indexing is represented explicitly by the signed adapter.
    assert!(buffer.get_line_signed(-1).unwrap().is_none());
    assert!(buffer.get_line(5).unwrap().is_none());
}
#[test]
fn get_line_is_wrapped() {
    let term = terminal(5, 24);
    let buffer = term.buffer().unwrap().active().unwrap();
    let wrapped = |row| buffer.get_line(row).unwrap().unwrap().borrow().is_wrapped;
    assert!(!wrapped(0));
    assert!(!wrapped(1));
    term.write("abcde").unwrap();
    assert!(!wrapped(0));
    assert!(!wrapped(1));
    term.write("f").unwrap();
    assert!(!wrapped(0));
    assert!(wrapped(1));
}
#[test]
fn get_line_translate_to_string() {
    let term = terminal(5, 24);
    assert_eq!(line(&term, 0, false), "     ");
    line_equals(&term, 0, "");
    term.write("foo").unwrap();
    assert_eq!(line(&term, 0, false), "foo  ");
    line_equals(&term, 0, "foo");
    term.write("bar").unwrap();
    assert_eq!(line(&term, 0, false), "fooba");
    line_equals(&term, 0, "fooba");
    line_equals(&term, 1, "r");
    let line = term
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf16(
            &line
                .borrow()
                .translate_to_string(false, Some(1), None, None)
        )
        .unwrap(),
        "ooba"
    );
    assert_eq!(
        String::from_utf16(
            &line
                .borrow()
                .translate_to_string(false, Some(1), Some(3), None)
        )
        .unwrap(),
        "oo"
    );
}
#[test]
fn get_line_get_cell() {
    let term = terminal(5, 24);
    let line = term
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    assert!(xterm_core::public_api::buffer::get_cell_signed(&line.borrow(), -1).is_none());
    assert!(line.borrow().get_cell(5).is_none());
    let cell = line.borrow().get_cell(0).unwrap();
    assert_eq!(cell.get_chars(), Vec::<u16>::new());
    assert_eq!(cell.get_width(), 1);
    term.write("a文").unwrap();
    for (index, text, width) in [(0, "a", 1), (1, "文", 2), (2, "", 0)] {
        let cell = line.borrow().get_cell(index).unwrap();
        assert_eq!(cell.get_chars(), text.encode_utf16().collect::<Vec<_>>());
        assert_eq!(cell.get_width(), width);
    }
}
#[test]
fn active_normal_alternate() {
    let term = terminal(5, 24);
    let buffers = term.buffer().unwrap();
    assert_eq!(buffers.active().unwrap().buffer_type(), BufferType::Normal);
    assert_eq!(buffers.normal().buffer_type(), BufferType::Normal);
    assert_eq!(buffers.alternate().buffer_type(), BufferType::Alternate);
    term.write("norm ").unwrap();
    assert_eq!(line(&term, 0, false), "norm ");
    assert_eq!(view_line(&buffers.normal(), 0, false), "norm ");
    assert!(buffers.alternate().get_line(0).unwrap().is_none());
    let normal = buffers.normal();
    term.write("\x1b[?47h\r").unwrap();
    assert_eq!(
        buffers.active().unwrap().buffer_type(),
        BufferType::Alternate
    );
    assert_eq!(buffers.normal().buffer_type(), BufferType::Normal);
    assert_eq!(buffers.alternate().buffer_type(), BufferType::Alternate);
    assert_eq!(line(&term, 0, false), "     ");
    term.write("alt  ").unwrap();
    assert_eq!(line(&term, 0, false), "alt  ");
    assert_eq!(view_line(&buffers.alternate(), 0, false), "alt  ");
    let line = normal.get_line(0).unwrap().unwrap();
    assert_eq!(
        String::from_utf16(&line.borrow().translate_to_string(false, None, None, None)).unwrap(),
        "norm "
    );
    term.write("\x1b[?47l\r").unwrap();
    assert_eq!(buffers.active().unwrap().buffer_type(), BufferType::Normal);
    assert_eq!(buffers.normal().buffer_type(), BufferType::Normal);
    assert_eq!(buffers.alternate().buffer_type(), BufferType::Alternate);
    assert_eq!(super::support::line(&term, 0, false), "norm ");
    assert_eq!(view_line(&buffers.normal(), 0, false), "norm ");
    assert!(buffers.alternate().get_line(0).unwrap().is_none());
    assert!(Rc::ptr_eq(&normal, &buffers.normal()));
}
#[test]
fn register_marker_on_alternate_buffer() {
    let term = terminal(5, 24);
    term.write("\x1b[?47h").unwrap();
    let marker = term.register_marker(0).unwrap();
    assert_eq!(
        term.buffer().unwrap().active().unwrap().buffer_type(),
        BufferType::Alternate
    );
    let markers = term.markers().unwrap();
    assert_eq!(markers.len(), 1);
    assert!(markers[0].ptr_eq(&marker));
}
