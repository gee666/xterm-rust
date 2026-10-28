// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Faithful assertion migration from common/services/DecorationService.test.ts.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, Buffer},
    decoration::{Decoration, DecorationLineCache, DecorationOptions, DecorationService},
    options::OptionsService,
};
fn setup() -> (Buffer, DecorationService) {
    let buffer = Buffer::new(
        true,
        80,
        24,
        Rc::new(OptionsService::new(
            false,
            std::iter::empty::<(&str, xterm_core::options::OptionValue)>(),
        )),
    )
    .unwrap();
    let service = DecorationService::new();
    service.attach_to_buffer_lines(&buffer.lines);
    (buffer, service)
}
fn register(
    buffer: &mut Buffer,
    service: &DecorationService,
    line: i64,
    x: Option<f64>,
    width: Option<f64>,
    height: Option<u32>,
) -> Decoration {
    let mut options = DecorationOptions::new(buffer.add_marker(line));
    options.x = x;
    options.width = width;
    options.height = height;
    service
        .register_decoration(Rc::new(RefCell::new(options)))
        .unwrap()
        .unwrap()
}
fn hits(service: &DecorationService, x: f64, line: i64) -> Vec<Decoration> {
    let mut found = Vec::new();
    service.for_each_decoration_at_cell(x, line, None, |d| found.push(d));
    found
}
#[test]
fn should_set_is_disposed_to_true_after_dispose() {
    let (mut buffer, service) = setup();
    let decoration = register(&mut buffer, &service, 1, None, None, None);
    assert!(!decoration.is_disposed());
    decoration.dispose().unwrap();
    assert!(decoration.is_disposed());
}
#[test]
fn should_find_decoration_at_its_marker_line() {
    let (mut buffer, service) = setup();
    register(&mut buffer, &service, 5, None, Some(10.0), None);
    assert_eq!(hits(&service, 0.0, 5).len(), 1);
}
#[test]
fn should_find_height_on_subsequent_lines_callback() {
    let (mut buffer, service) = setup();
    register(&mut buffer, &service, 5, None, Some(10.0), Some(3));
    for line in [5, 6, 7] {
        assert_eq!(hits(&service, 0.0, line).len(), 1);
    }
    assert!(hits(&service, 0.0, 8).is_empty());
}
#[test]
fn should_not_find_decoration_outside_x_range() {
    let (mut buffer, service) = setup();
    register(&mut buffer, &service, 5, Some(5.0), Some(3.0), Some(2));
    assert!(hits(&service, 4.0, 5).is_empty());
    assert_eq!(hits(&service, 5.0, 5).len(), 1);
    assert_eq!(hits(&service, 7.0, 6).len(), 1);
    assert!(hits(&service, 8.0, 5).is_empty());
}
#[test]
fn should_find_multiline_with_single_line_decorations_elsewhere() {
    let (mut buffer, service) = setup();
    buffer.fill_viewport_rows(None).unwrap();
    for i in 0..buffer.lines.len() {
        register(&mut buffer, &service, i as i64, None, Some(5.0), None);
    }
    let multi = register(&mut buffer, &service, 10, None, Some(10.0), Some(3));
    assert!(hits(&service, 0.0, 11).iter().any(|d| d.ptr_eq(&multi)));
}
#[test]
fn should_find_height_on_subsequent_lines_iterator() {
    let (mut buffer, service) = setup();
    register(&mut buffer, &service, 5, None, Some(10.0), Some(3));
    for line in [5, 6, 7] {
        assert_eq!(service.get_decorations_at_cell(0.0, line, None).count(), 1);
    }
    assert_eq!(service.get_decorations_at_cell(0.0, 8, None).count(), 0);
}
#[test]
fn should_return_absence_for_lines_without_indexed_decorations() {
    assert!(DecorationLineCache::new().decorations_on_line(0).is_none());
}
#[test]
fn should_keep_lookups_correct_after_buffer_trim() {
    let (mut buffer, service) = setup();
    buffer.fill_viewport_rows(None).unwrap();
    let line = buffer.lines.len() as i64 - 1;
    let decoration = register(&mut buffer, &service, line, None, Some(10.0), None);
    buffer.lines.on_trim_emitter.fire(&1).unwrap();
    let line = decoration.marker().line_number();
    assert_eq!(line.fract(), 0.0);
    assert_eq!(hits(&service, 0.0, line as i64).len(), 1);
}
#[test]
fn should_remove_decoration_when_marker_trimmed_off_buffer() {
    let (mut buffer, service) = setup();
    buffer.fill_viewport_rows(None).unwrap();
    let decoration = register(&mut buffer, &service, 0, None, Some(10.0), None);
    buffer.lines.on_trim_emitter.fire(&1).unwrap();
    assert!(decoration.marker().is_disposed());
    assert!(decoration.is_disposed());
    assert!(hits(&service, 0.0, 0).is_empty());
}
#[test]
fn should_keep_multiline_index_after_actual_line_insert() {
    let (mut buffer, service) = setup();
    buffer.fill_viewport_rows(None).unwrap();
    let decoration = register(&mut buffer, &service, 3, None, Some(10.0), Some(3));
    let blank = buffer.get_blank_line(&AttributeData::default(), false);
    buffer.lines.splice(5, 0, &[blank]).unwrap();
    service.line_cache().flush_pending().unwrap();
    let mut found = Vec::new();
    let start = decoration.marker().line_number();
    assert_eq!(start.fract(), 0.0);
    for line in start as i64..start as i64 + 3 {
        found.extend(hits(&service, 0.0, line));
    }
    assert!(found.iter().any(|d| d.ptr_eq(&decoration)));
    assert!(hits(&service, 0.0, start as i64 + 3).is_empty());
}
