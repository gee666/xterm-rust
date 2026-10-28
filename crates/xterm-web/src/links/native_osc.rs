// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{
    browser::{default_activate, is_http_url},
    browser_linkifier::{BrowserLink, BrowserLinkProvider, LinkCallback, LinkReply},
    osc::{try_osc_links_with_policy, OscBuffer},
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

pub type OscProtocolPolicy = Rc<dyn Fn() -> Result<bool, JsValue>>;
#[derive(Clone)]
pub struct OscLinkHandler {
    pub allow_non_http_protocols: OscProtocolPolicy,
    pub activate: LinkCallback,
    pub hover: Option<LinkCallback>,
    pub leave: Option<LinkCallback>,
}

struct LiveBuffer {
    buffer: Rc<RefCell<Buffer>>,
    scan_index: usize,
    scan_line: Rc<RefCell<BufferLine>>,
}
impl OscBuffer for LiveBuffer {
    type Error = JsValue;
    fn with_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, JsValue> {
        let line = self
            .buffer
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .get(index);
        let Some(line) = line else { return Ok(None) };
        let line = line.try_borrow().map_err(|error| to_js_error(&error))?;
        Ok(Some(read(&line)))
    }
    fn with_scan_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, JsValue> {
        if index != self.scan_index {
            return self.with_line(index, read);
        }
        let line = self
            .scan_line
            .try_borrow()
            .map_err(|error| to_js_error(&error))?;
        Ok(Some(read(&line)))
    }
}

/// Reads URL ids and URI data from the SAME Rust terminal state used by OSC8
/// parsing and printing. No terminal engine or link discovery runs in JavaScript.
/// Handler identity is sampled once for an existing row. The protocol getter is
/// read for each finished run, after URI lookup and wrapped-range expansion.
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
        let Some(row) = y.checked_sub(1) else {
            return callback(None);
        };
        let scan_line = buffer
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .get(row);
        let Some(scan_line) = scan_line else {
            return callback(None);
        };
        let source = LiveBuffer {
            buffer,
            scan_index: row,
            scan_line,
        };
        let handler = (self.handler)()?;
        let found = try_osc_links_with_policy(
            &source,
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
            |text| {
                let allow = handler
                    .as_ref()
                    .map(|handler| (handler.allow_non_http_protocols)())
                    .transpose()?
                    .unwrap_or(false);
                Ok(allow || is_http_url(text))
            },
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
