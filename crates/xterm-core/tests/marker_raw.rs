use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{marker::Marker, marker_line::MarkerLine, Buffer, BufferService},
    circular_list::ListChange,
    options::{OptionValue, OptionsService},
    osc_link_service::{OscLinkData, OscLinkService},
};
fn buffer() -> Buffer {
    Buffer::new(
        true,
        4,
        3,
        Rc::new(OptionsService::new(
            false,
            std::iter::empty::<(&str, OptionValue)>(),
        )),
    )
    .unwrap()
}
fn string(value: &str) -> MarkerLine {
    MarkerLine::from(value)
}
#[test]
fn clones_share_the_single_raw_value_and_snapshots_do_not_change() {
    let marker = Marker::new(string("01"));
    let copy = marker.clone();
    let old = marker.line_value();
    copy.set_line(1.5);
    assert_eq!(marker.line_number(), 1.5);
    assert_eq!(
        old.as_string(),
        Some("01".encode_utf16().collect::<Vec<_>>().as_slice())
    );
    marker.dispose().unwrap();
    assert_eq!(copy.line_value().as_number(), Some(-1.0));
    copy.set_line(string("02"));
    copy.dispose().unwrap();
    assert_eq!(marker.line_value(), string("02"));
}
#[test]
fn insert_concatenates_then_trim_converts_to_number() {
    let mut b = buffer();
    let marker = b.add_marker_value(string("01"));
    b.lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 2,
        })
        .unwrap();
    assert_eq!(marker.line_value(), string("012"));
    b.lines.on_trim_emitter.fire(&1).unwrap();
    assert_eq!(marker.line_value().as_number(), Some(11.0));
}
#[test]
fn clearing_uses_strict_type_equality() {
    let mut b = buffer();
    let numeric = b.add_marker(0);
    let text = b.add_marker_value(string("0"));
    b.clear_markers(0).unwrap();
    assert!(numeric.is_disposed());
    assert!(!text.is_disposed());
    b.clear_markers_value(&string("0")).unwrap();
    assert!(text.is_disposed());
}
#[test]
fn delete_rechecks_the_line_changed_by_a_disposal_callback() {
    let mut b = buffer();
    let marker = b.add_marker_value(string("1"));
    let weak = marker.downgrade();
    marker
        .on_dispose()
        .subscribe(move |_| weak.upgrade().unwrap().set_line(string("99")));
    b.lines
        .on_delete_emitter
        .fire(&ListChange {
            index: 1,
            amount: 1,
        })
        .unwrap();
    assert!(marker.is_disposed());
    assert_eq!(marker.line_value().as_number(), Some(98.0));
}
#[test]
fn invalid_numeric_strings_become_nan_without_disposal() {
    let mut b = buffer();
    let marker = b.add_marker_value(string("0foo"));
    b.lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 2,
        })
        .unwrap();
    assert_eq!(marker.line_value(), string("0foo"));
    b.lines.on_trim_emitter.fire(&1).unwrap();
    assert!(marker.line_number().is_nan());
    assert!(!marker.is_disposed());
}
#[test]
fn numeric_precision_and_nonfinite_lines_follow_js() {
    let mut b = buffer();
    let large = b.add_marker_value(MarkerLine::number(2_f64.powi(53)));
    let nan = b.add_marker_value(MarkerLine::number(f64::NAN));
    let positive = b.add_marker_value(MarkerLine::number(f64::INFINITY));
    let negative = b.add_marker_value(MarkerLine::number(f64::NEG_INFINITY));
    b.lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert_eq!(large.line_number(), 2_f64.powi(53));
    b.lines.on_trim_emitter.fire(&0).unwrap();
    assert!(!nan.is_disposed());
    assert!(!positive.is_disposed());
    assert!(negative.is_disposed());
}
#[test]
fn osc_line_dedup_does_not_coerce_a_string_marker() {
    let options = Rc::new(OptionsService::new(
        false,
        [("cols", 4.0.into()), ("rows", 3.0.into())],
    ));
    let buffers = Rc::new(BufferService::new(options).unwrap());
    let links = OscLinkService::new(Rc::clone(&buffers));
    let id = links
        .register_link(Rc::new(RefCell::new(OscLinkData {
            id: None,
            uri: vec![],
        })))
        .unwrap();
    buffers.buffer().borrow().markers.borrow()[0].set_line(string("0"));
    links.add_line_to_link(id, 0).unwrap();
    assert_eq!(buffers.buffer().borrow().markers.borrow().len(), 2);
}
