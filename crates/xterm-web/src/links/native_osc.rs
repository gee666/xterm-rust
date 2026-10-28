// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{
    browser::{default_activate, is_http_url},
    browser_linkifier::{BrowserLink, BrowserLinkProvider, LinkCallback, LinkReply},
    osc::{try_osc_links, OscBuffer},
};
use crate::js_error::to_js_error;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use web_sys::Window;
use xterm_core::{
    buffer::{Buffer, BufferLine},
    headless,
};

#[derive(Clone)]
pub struct OscLinkHandler {
    pub allow_non_http_protocols: bool,
    pub activate: LinkCallback,
    pub hover: Option<LinkCallback>,
    pub leave: Option<LinkCallback>,
}

struct LiveBuffer(Rc<RefCell<Buffer>>);
impl OscBuffer for LiveBuffer {
    type Error = JsValue;
    fn with_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, JsValue> {
        let line = self
            .0
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .get(index);
        let Some(line) = line else { return Ok(None) };
        let line = line.try_borrow().map_err(|error| to_js_error(&error))?;
        Ok(Some(read(&line)))
    }
}

/// Reads URL ids and URI data from the SAME Rust terminal state used by OSC8
/// parsing and printing. No terminal engine or link discovery runs in JavaScript.
/// Handler settings are sampled once per provideLinks call, as upstream does.
pub struct NativeOscLinkProvider {
    terminal: Weak<headless::Terminal>,
    window: Window,
    handler: Rc<dyn Fn() -> Result<Option<OscLinkHandler>, JsValue>>,
}
impl NativeOscLinkProvider {
    pub fn new(
        terminal: Weak<headless::Terminal>,
        window: Window,
        handler: impl Fn() -> Result<Option<OscLinkHandler>, JsValue> + 'static,
    ) -> Self {
        Self {
            terminal,
            window,
            handler: Rc::new(handler),
        }
    }
}
impl BrowserLinkProvider for NativeOscLinkProvider {
    fn provide_links(&self, y: usize, callback: LinkReply) -> Result<(), JsValue> {
        let Some(terminal) = self.terminal.upgrade() else {
            return callback(None);
        };
        let (buffer, links) = terminal
            .with_state(|state| (state.active_buffer(), state.links.clone()))
            .map_err(|error| to_js_error(&error))?;
        let handler = (self.handler)()?;
        let allow = handler
            .as_ref()
            .is_some_and(|handler| handler.allow_non_http_protocols);
        let found = try_osc_links(
            &LiveBuffer(buffer),
            y,
            |id| {
                links
                    .get_link_data(id)
                    .map(|data| {
                        data.try_borrow()
                            .map(|data| data.uri.clone())
                            .map_err(|error| to_js_error(&error))
                    })
                    .transpose()
            },
            allow,
            is_http_url,
        )?;
        let found = found.map(|links| {
            links
                .into_iter()
                .map(|data| {
                    let window = self.window.clone();
                    let activate = handler
                        .as_ref()
                        .map(|handler| handler.activate.clone())
                        .unwrap_or_else(|| {
                            Rc::new(move |_, link| default_activate(&window, &link.text))
                        });
                    BrowserLink {
                        data: Rc::new(data),
                        decorations: None,
                        activate,
                        hover: handler.as_ref().and_then(|handler| handler.hover.clone()),
                        leave: handler.as_ref().and_then(|handler| handler.leave.clone()),
                        dispose: None,
                    }
                })
                .collect()
        });
        callback(found)
    }
}
