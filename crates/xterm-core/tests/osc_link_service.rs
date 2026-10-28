// Copyright (c) 2020-2022 The xterm.js authors. All rights reserved. MIT license.
// First three tests port every OscLinkService.test.ts assertion.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, BufferService},
    options::{OptionValue, OptionsService},
    osc_link_service::{LinkDataRef, OscLinkData, OscLinkError, OscLinkService},
};

#[path = "support/osc_link_trace.rs"]
mod trace;

fn service() -> (Rc<BufferService>, OscLinkService) {
    let options = OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(10.0)),
            ("rows", OptionValue::Number(3.0)),
            ("scrollback", OptionValue::Number(2.0)),
        ],
    );
    let buffers = Rc::new(BufferService::new(Rc::new(options)).unwrap());
    let links = OscLinkService::new(Rc::clone(&buffers));
    (buffers, links)
}
fn data(id: Option<&str>, uri: &str) -> LinkDataRef {
    Rc::new(RefCell::new(OscLinkData {
        id: id.map(|id| id.encode_utf16().collect()),
        uri: uri.encode_utf16().collect(),
    }))
}

#[test]
fn upstream_link_ids_are_created_and_fetched_consistently() {
    let (_, links) = service();
    let id = links.register_link(data(Some("foo"), "bar")).unwrap();
    assert!(id > 0);
    assert_eq!(links.register_link(data(Some("foo"), "bar")), Ok(id));
}
#[test]
fn upstream_last_trimmed_marker_expires_link_id() {
    let (buffers, links) = service();
    buffers.buffers.activate_alt_buffer(None).unwrap();
    let id = links.register_link(data(Some("foo"), "bar")).unwrap();
    assert!(id > 0);
    buffers.scroll(&AttributeData::default(), false).unwrap();
    assert_ne!(links.register_link(data(Some("foo"), "bar")).unwrap(), id);
}
#[test]
fn upstream_fetches_exact_link_data() {
    let (_, links) = service();
    let id = links.register_link(data(Some("foo"), "bar")).unwrap();
    assert_eq!(
        *links.get_link_data(id).unwrap().borrow(),
        *data(Some("foo"), "bar").borrow()
    );
}

#[test]
fn unnamed_links_are_unique_empty_named_ids_reuse_and_delimiters_collide() {
    let (_, links) = service();
    assert!(links.link_ids().is_empty());
    let a = links.register_link(data(None, "same")).unwrap();
    let b = links.register_link(data(None, "same")).unwrap();
    assert_ne!(a, b);
    assert_eq!(links.link_ids(), vec![a, b]);
    let empty = links.register_link(data(Some(""), "same")).unwrap();
    assert_eq!(links.register_link(data(Some(""), "same")), Ok(empty));
    let collision = links.register_link(data(Some("a;;b"), "c")).unwrap();
    assert_eq!(links.register_link(data(Some("a"), "b;;c")), Ok(collision));
    assert_eq!(
        links.get_link_data(collision).unwrap().borrow().uri,
        vec![u16::from(b'c')]
    );
}

#[test]
fn data_identity_and_utf16_surrogates_are_lossless() {
    let (_, links) = service();
    let shared = Rc::new(RefCell::new(OscLinkData {
        id: Some(vec![0xd800]),
        uri: vec![0xdc00, 0, 0xd83d, 0xde00],
    }));
    let id = links.register_link(Rc::clone(&shared)).unwrap();
    assert!(Rc::ptr_eq(&shared, &links.get_link_data(id).unwrap()));
    shared.borrow_mut().uri.push(0xffff);
    assert_eq!(
        links.get_link_data(id).unwrap().borrow().uri,
        shared.borrow().uri
    );
    assert!(links.get_link_data(0).is_none());
    assert!(links.get_link_data(u64::MAX).is_none());
}

#[test]
fn markers_deduplicate_current_lines_and_last_disposal_expires_entry() {
    let (buffers, links) = service();
    let id = links.register_link(data(Some("foo"), "bar")).unwrap();
    links.add_line_to_link(id, 0).unwrap();
    assert_eq!(buffers.buffer().borrow().markers.borrow().len(), 1);
    links.add_line_to_link(id, 1).unwrap();
    let markers = buffers.buffer().borrow().markers.borrow().clone();
    markers[0].set_line(9);
    links.add_line_to_link(id, 9).unwrap();
    assert_eq!(buffers.buffer().borrow().markers.borrow().len(), 2);
    markers[0].dispose().unwrap();
    assert!(links.get_link_data(id).is_some());
    markers[1].dispose().unwrap();
    assert!(links.get_link_data(id).is_none());
    assert_ne!(links.register_link(data(Some("foo"), "bar")).unwrap(), id);
}

#[test]
fn active_buffers_and_absolute_cursor_lines_are_live() {
    let (buffers, links) = service();
    buffers.buffer().borrow_mut().ybase = 3;
    buffers.buffer().borrow_mut().y = 2;
    let id = links.register_link(data(Some("foo"), "bar")).unwrap();
    assert_eq!(buffers.buffer().borrow().markers.borrow()[0].line(), 5);
    buffers.buffers.activate_alt_buffer(None).unwrap();
    links.add_line_to_link(id, 5).unwrap();
    assert!(buffers.buffer().borrow().markers.borrow().is_empty());
    links.add_line_to_link(id, -1).unwrap();
    assert_eq!(buffers.buffer().borrow().markers.borrow()[0].line(), -1);
}

#[test]
fn named_id_mutation_preserves_stale_key_source_behavior() {
    let (buffers, links) = service();
    let shared = data(Some("foo"), "bar");
    let id = links.register_link(Rc::clone(&shared)).unwrap();
    shared.borrow_mut().id = None;
    let marker = buffers.buffer().borrow().markers.borrow()[0].clone();
    marker.dispose().unwrap();
    assert!(links.get_link_data(id).is_none());
    assert_eq!(links.register_link(data(Some("foo"), "bar")), Ok(id));
    assert!(links.get_link_data(id).is_none());
    assert!(buffers.buffer().borrow().markers.borrow().is_empty());
}

#[test]
fn unnamed_to_named_mutation_cleans_only_its_numeric_entry() {
    let (buffers, links) = service();
    let shared = data(None, "bar");
    let id = links.register_link(Rc::clone(&shared)).unwrap();
    shared.borrow_mut().id = Some("foo".encode_utf16().collect());
    let marker = buffers.buffer().borrow().markers.borrow()[0].clone();
    marker.dispose().unwrap();
    assert!(links.get_link_data(id).is_none());
    assert_ne!(links.register_link(data(Some("foo"), "bar")).unwrap(), id);
}

#[test]
fn typed_borrow_errors_do_not_allocate_a_marker_or_id() {
    let (buffers, links) = service();
    let shared = data(Some("foo"), "bar");
    let active = buffers.buffer();
    let borrowed_buffer = active.borrow_mut();
    assert_eq!(
        links.register_link(Rc::clone(&shared)),
        Err(OscLinkError::BorrowedBuffer)
    );
    drop(borrowed_buffer);
    let borrowed_data = shared.borrow_mut();
    assert_eq!(
        links.register_link(Rc::clone(&shared)),
        Err(OscLinkError::BorrowedData)
    );
    drop(borrowed_data);
    assert!(active.borrow().markers.borrow().is_empty());
    assert_eq!(links.register_link(shared), Ok(1));
}

#[test]
fn marker_cleanup_borrow_failure_retains_original_error_and_partial_state() {
    let (buffers, links) = service();
    let shared = data(Some("foo"), "bar");
    let id = links.register_link(Rc::clone(&shared)).unwrap();
    let marker = buffers.buffer().borrow().markers.borrow()[0].clone();
    let borrowed = shared.borrow_mut();
    let error = marker.dispose().unwrap_err();
    assert_eq!(
        error.callback_ref::<OscLinkError>(),
        Some(&OscLinkError::BorrowedData)
    );
    assert!(marker.is_disposed());
    assert_eq!(marker.line(), -1);
    assert!(links.get_link_data(id).is_some());
    drop(borrowed);
    marker.dispose().unwrap();
    assert!(links.get_link_data(id).is_some());
}

#[test]
fn service_drop_releases_data_without_disposing_buffer_markers() {
    let (buffers, links) = service();
    let shared = data(Some("foo"), "bar");
    let weak = Rc::downgrade(&shared);
    links.register_link(Rc::clone(&shared)).unwrap();
    let marker = buffers.buffer().borrow().markers.borrow()[0].clone();
    drop(shared);
    assert!(weak.upgrade().is_some());
    drop(links);
    assert!(weak.upgrade().is_none());
    assert!(!marker.is_disposed());
    marker.dispose().unwrap();
}

#[test]
fn deterministic_source_service_trace_matches_rust() {
    let mut runner = trace::Runner::new().unwrap();
    for (index, line) in include_str!("fixtures/osc-link-traces.tsv")
        .lines()
        .enumerate()
    {
        let (operation, expected) = line.split_once('\t').unwrap();
        assert_eq!(
            runner.step(operation).unwrap(),
            expected,
            "trace operation{index}: {operation}"
        );
    }
}
