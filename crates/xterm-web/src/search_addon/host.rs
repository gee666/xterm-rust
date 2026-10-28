// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::timer;
use crate::{search_host::SearchTerminal, terminal::Terminal};
use std::{
    any::Any,
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, Performance, Window};
use xterm_addons::search::{SearchAddonError, SearchHost, TimeoutCallback};
use xterm_core::{
    decoration::DecorationService,
    event::EmitError,
    headless,
    lifecycle::DisposableHandle,
    search::{SearchResult, SearchSelection},
};

pub struct BrowserSearchHost {
    terminal: SearchTerminal,
    window: Window,
    performance: Performance,
    shared: Weak<crate::terminal::Shared>,
}
impl BrowserSearchHost {
    pub fn new(terminal: &Terminal) -> Result<Self, JsValue> {
        let window = web_sys::window()
            .ok_or_else(|| js_sys::Error::new("Search requires a browser window"))?;
        let performance = window
            .performance()
            .ok_or_else(|| js_sys::Error::new("Search requires performance.now"))?;
        Ok(Self {
            terminal: SearchTerminal {
                native: terminal.shared_native(),
                selection: terminal.selection_service(),
                decorations: terminal.decoration_service(),
            },
            window,
            performance,
            shared: Rc::downgrade(&terminal.shared),
        })
    }
}
impl BrowserSearchHost {
    fn is_open(&self) -> bool {
        self.shared
            .upgrade()
            .is_some_and(|shared| shared.dom.borrow().is_some())
    }
}
impl SearchHost for BrowserSearchHost {
    fn terminal(&self) -> Rc<headless::Terminal> {
        Rc::clone(&self.terminal.native)
    }
    fn decorations(&self) -> Rc<DecorationService> {
        Rc::clone(&self.terminal.decorations)
    }
    fn selection(&self) -> Result<Option<SearchSelection>, SearchAddonError> {
        if !self.is_open() {
            return Ok(None);
        }
        self.terminal.selection_position()
    }
    fn clear_selection(&self) -> Result<(), SearchAddonError> {
        if !self.is_open() {
            return Ok(());
        }
        self.terminal.clear_selection()
    }
    fn select(&self, result: &SearchResult) -> Result<(), SearchAddonError> {
        if !self.is_open() {
            let error = js_sys::TypeError::new(
                "Cannot read properties of undefined (reading 'setSelection')",
            );
            return Err(SearchAddonError::Host(Rc::new(
                crate::js_error::JsCallbackError(error.into()),
            )));
        }
        self.terminal.select(result)
    }
    fn style_decoration(
        &self,
        element: &Rc<dyn Any>,
        border: Option<&str>,
    ) -> Result<(), EmitError> {
        let element = element.downcast_ref::<HtmlElement>().ok_or_else(|| {
            crate::js_error::callback_error(
                js_sys::TypeError::new("Search decoration renderer supplied a non-HTML element")
                    .into(),
            )
        })?;
        if !element
            .class_list()
            .contains("xterm-find-result-decoration")
        {
            element
                .class_list()
                .add_1("xterm-find-result-decoration")
                .map_err(crate::js_error::callback_error)?;
            if let Some(border) = border.filter(|border| !border.is_empty()) {
                element
                    .style()
                    .set_property("outline", &format!("1px solid {border}"))
                    .map_err(crate::js_error::callback_error)?;
            }
        }
        Ok(())
    }
    fn now_ms(&self) -> u64 {
        self.performance.now().floor() as u64
    }
    fn timeout(
        &self,
        delay: u64,
        callback: TimeoutCallback,
    ) -> Result<DisposableHandle, SearchAddonError> {
        timer::timeout(&self.window, delay, callback)
    }
}
