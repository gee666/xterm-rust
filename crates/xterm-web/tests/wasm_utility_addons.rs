#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::{Function, Promise};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, WebSocket};
use xterm_addons::{attach::*, fit::*, web_links::*};
use xterm_core::lifecycle::{Disposable, DisposableHandle, NoopDisposable};
use xterm_web::{
    api::{RenderDimensions, Size},
    utility_addons::{
        fit::{BrowserFitTerminal, ScrollbarOptions},
        matcher::BrowserLinkMatcher,
        socket::BrowserSocket,
    },
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn standard_regex_url_and_utf16_match_positions() {
    let matcher = BrowserLinkMatcher;
    let data = "🦀 HTTP://Ab:xY@abc.com:80/staysUpper http://[::1]/ bad http://example.com:999999/"
        .encode_utf16()
        .collect::<Vec<_>>();
    let found = matcher.matches(&data, &RegexSpec::default()).unwrap();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].index, 3);
    assert_eq!(
        String::from_utf16(&found[0].text).unwrap(),
        "HTTP://Ab:xY@abc.com:80/staysUpper"
    );
    assert_eq!(String::from_utf16(&found[1].text).unwrap(), "http://[::1]/");
    let custom = RegexSpec {
        source: "(?=x)".into(),
        flags: String::new(),
    };
    assert!(matcher.matches(&[120], &custom).is_err());
    assert!(matcher
        .matches(
            &[120],
            &RegexSpec {
                source: "[".into(),
                flags: String::new()
            }
        )
        .is_err());
}
#[wasm_bindgen_test]
fn live_dom_fit_css_padding_scrollbar_and_same_dimensions() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    let element: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    parent
        .style()
        .set_css_text("width:800px;height:450px;position:absolute");
    element.style().set_css_text("padding:10px 5px");
    document.body().unwrap().append_child(&parent).unwrap();
    parent.append_child(&element).unwrap();
    let resized = Rc::new(RefCell::new(Vec::new()));
    let captured = resized.clone();
    let terminal = Rc::new(BrowserFitTerminal {
        element: Rc::new(move || Some(element.clone())),
        dimensions: Rc::new(|| {
            let mut dimensions = RenderDimensions::default();
            dimensions.css.cell = Size {
                width: 9.0,
                height: 17.0,
            };
            Some(dimensions)
        }),
        scrollbar: Rc::new(|| {
            Ok(ScrollbarOptions {
                scrollback: 1000.0,
                show: None,
                width: None,
            })
        }),
        resize: Rc::new(move |dimensions| {
            captured.borrow_mut().push(dimensions);
            Ok(())
        }),
    });
    let mut addon = FitAddon::new();
    assert_eq!(addon.propose_dimensions().unwrap(), None);
    addon.activate(terminal.clone());
    assert_eq!(
        addon.propose_dimensions().unwrap(),
        Some(Dimensions {
            cols: 86.0,
            rows: 25.0
        })
    );
    addon.fit().unwrap();
    addon.fit().unwrap();
    assert_eq!(resized.borrow().len(), 2);
    parent
        .style()
        .set_css_text("width:1px;height:1px;position:absolute");
    addon.fit().unwrap();
    assert_eq!(
        resized.borrow()[2],
        Dimensions {
            cols: 2.0,
            rows: 1.0
        }
    );
    addon.dispose();
    assert!(addon.propose_dimensions().unwrap().is_some());
    parent.remove();
}

struct Sink {
    writes: RefCell<Vec<SocketMessage>>,
    done: RefCell<Option<Function>>,
}
impl AttachTerminal for Sink {
    fn write(&self, data: SocketMessage) -> Result<(), AttachError> {
        self.writes.borrow_mut().push(data);
        if self.writes.borrow().len() == 2 {
            self.done
                .borrow_mut()
                .take()
                .unwrap()
                .call0(&JsValue::UNDEFINED)
                .unwrap();
        }
        Ok(())
    }
    fn on_data(&self, _: DataCallback) -> DisposableHandle {
        Rc::new(NoopDisposable)
    }
    fn on_binary(&self, _: DataCallback) -> DisposableHandle {
        Rc::new(NoopDisposable)
    }
}
#[wasm_bindgen_test(async)]
async fn real_websocket_delivers_string_and_arraybuffer_to_rust_addon() {
    let Some(url) = option_env!("UTILITY_SOCKET_URL") else {
        panic!("run with scripts/utility-addon-browser-check.mjs");
    };
    let socket = WebSocket::new(url).unwrap();
    let terminal = Rc::new(Sink {
        writes: RefCell::default(),
        done: RefCell::default(),
    });
    let target = terminal.clone();
    let done = Promise::new(&mut move |resolve, _| {
        *target.done.borrow_mut() = Some(resolve);
    });
    let addon = AttachAddon::new(Rc::new(BrowserSocket(socket.clone())), None).unwrap();
    addon.activate(terminal.clone()).unwrap();
    JsFuture::from(done).await.unwrap();
    assert_eq!(
        *terminal.writes.borrow(),
        [
            SocketMessage::Text(vec![102, 111, 111]),
            SocketMessage::Binary(vec![102, 111, 111])
        ]
    );
    addon.dispose();
    addon.dispose();
    socket.close().unwrap();
}
