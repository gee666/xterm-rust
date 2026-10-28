#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_addons::{
    link_snapshot::LinkSnapshot,
    web_link_provider::{LinkOptions, WebLinksAddon},
    web_links::{compute_links, LinkRange, Position, RegexSpec},
};
use xterm_core::{lifecycle::Disposable, terminal::CoreTerminal};
use xterm_web::{
    links::{browser_linkifier::BrowserLinkifier, Position as BrowserPosition},
    utility_addons::{matcher::BrowserLinkMatcher, provider::BrowserWebLinkTerminal},
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn source_wide_combining_and_wrapped_offsets_use_actual_rust_buffer() {
    for (input, y, start, end) in [
        ("aaa http://example.com aaa http://example.com aaa",1,(5,1),(22,1)),
        ("￥￥￥ http://example.com ￥￥￥ http://example.com aaa",1,(8,1),(25,1)),
        ("￥￥￥ https://ko.wikipedia.org/wiki/위키백과:대문 aaa https://ko.wikipedia.org/wiki/위키백과:대문 ￥￥￥",1,(8,1),(11,2)),
        ("￥￥￥cafe\u{0301} http://test:password@example.com/some_path",1,(12,1),(13,2)),
        ("￥￥￥cafe\u{0301} http://test:password@example.com/some_path?param=1%202%3",1,(12,1),(27,2)),
    ] {
        let mut engine=CoreTerminal::new(40,24,1000).unwrap(); engine.write(input).unwrap();
        let snapshot=LinkSnapshot::from_engine(&engine).unwrap();
        let found=compute_links(y,&RegexSpec::default(),&snapshot,&BrowserLinkMatcher).unwrap();
        assert!(!found.is_empty(), "{input}");
        assert_eq!(found[0].range, LinkRange { start:Position{x:start.0,y:start.1}, end:Position{x:end.0,y:end.1} }, "{input}");
    }
}

#[wasm_bindgen_test]
fn addon_provider_real_dom_hover_callbacks_underline_and_dispose() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let mut engine = CoreTerminal::new(40, 24, 1000).unwrap();
    engine.write("aaa http://example.com aaa").unwrap();
    let engine = Rc::new(engine);
    let underlines = Rc::new(RefCell::new(Vec::new()));
    let capture = underlines.clone();
    let linkifier = Rc::new(
        BrowserLinkifier::new(
            element.clone(),
            40,
            0,
            Rc::new(|_| Some(BrowserPosition { x: 5, y: 1 })),
            Rc::new(move |enabled, range| {
                capture.borrow_mut().push((enabled, range));
                Ok(())
            }),
        )
        .unwrap(),
    );
    let snapshot = engine.clone();
    let terminal = Rc::new(BrowserWebLinkTerminal {
        linkifier: linkifier.clone(),
        snapshot: Rc::new(move || Ok(Box::new(LinkSnapshot::from_engine(&snapshot)?))),
    });
    let hover = Rc::new(RefCell::new(Vec::new()));
    let hovered = hover.clone();
    let addon = WebLinksAddon::new(
        Rc::new(BrowserLinkMatcher),
        None,
        LinkOptions {
            hover: Some(Rc::new(move |_: &MouseEvent, text, range| {
                hovered.borrow_mut().push((text.to_vec(), range));
                Ok(())
            })),
            ..LinkOptions::default()
        },
    );
    addon.activate(terminal.clone()).unwrap();
    element
        .dispatch_event(&MouseEvent::new("mousemove").unwrap())
        .unwrap();
    let active = linkifier.current_link().unwrap();
    assert_eq!(
        String::from_utf16(&active.text).unwrap(),
        "http://example.com"
    );
    assert_eq!(hover.borrow().len(), 1);
    assert_eq!(hover.borrow()[0].1.start.x, 5);
    assert!(underlines
        .borrow()
        .iter()
        .any(|(enabled, range)| *enabled && range.x1 == 4 && range.x2 == 22));
    addon.dispose();
    linkifier.on_render(0, 0).unwrap();
    assert!(linkifier.current_link().is_none());
    drop(terminal);
    drop(linkifier);
    element.remove();
}

#[wasm_bindgen_test]
fn every_original_hostname_and_path_variant_uses_rust_buffer_and_js_url() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/utility-hostnames.json")).unwrap();
    let hosts = corpus["hosts"].as_array().unwrap();
    assert_eq!(hosts.len(), 495);
    let mut assertions = 0;
    for host in hosts {
        let host = host.as_str().unwrap();
        for (input, expected) in [
            (format!("  http://{host}  "), format!("http://{host}")),
            (
                format!("  http://{host}/a~b#c~d?e~f  "),
                format!("http://{host}/a~b#c~d?e~f"),
            ),
            (
                format!("  http://{host}/colon:test  "),
                format!("http://{host}/colon:test"),
            ),
            (
                format!("  http://{host}/colon:test:  "),
                format!("http://{host}/colon:test"),
            ),
            (format!("\"http://{host}/\""), format!("http://{host}/")),
            (format!("'http://{host}/'"), format!("http://{host}/")),
            (
                format!("http://{host}/subpath/+/id"),
                format!("http://{host}/subpath/+/id"),
            ),
        ] {
            let mut engine = CoreTerminal::new(40, 24, 1000).unwrap();
            engine.write(&input).unwrap();
            let snapshot = LinkSnapshot::from_engine(&engine).unwrap();
            let links =
                compute_links(1, &RegexSpec::default(), &snapshot, &BrowserLinkMatcher).unwrap();
            assert_eq!(links.len(), 1, "{input}");
            assert_eq!(
                String::from_utf16(&links[0].text).unwrap(),
                expected,
                "{input}"
            );
            assertions += 1;
        }
    }
    assert_eq!(assertions, 3465);
    console_log!("Executed495 original hostname cases with3465 path assertions through actual Rust buffers and browser RegExp/URL.");
}
