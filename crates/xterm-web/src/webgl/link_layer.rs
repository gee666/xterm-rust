// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! LinkRenderLayer over a real canvas and the terminal's existing underline route.
use super::{canvas_layer::CanvasLayer, config::is_256_color, layer_services::LayerServices};
use crate::{
    api::RenderDimensions, links::UnderlineEvent, renderer::service_api::LinkUnderlineRenderer,
};
use js_sys::JsString;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

const INVERTED_DEFAULT_COLOR: u32 = 257;

/// Typed counterpart of ILinkifierEvent. Rc<Cell<_>> preserves retained event
/// mutation between show and hide. Arbitrary JS event getters are not this API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkLayerEvent {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub cols: f64,
    pub foreground: Option<u32>,
}
impl From<UnderlineEvent> for LinkLayerEvent {
    fn from(event: UnderlineEvent) -> Self {
        Self {
            x1: event.x1 as f64,
            y1: event.y1 as f64,
            x2: event.x2 as f64,
            y2: event.y2 as f64,
            cols: event.cols as f64,
            foreground: None,
        }
    }
}
pub struct LinkRenderLayer {
    pub base: Rc<CanvasLayer>,
    state: RefCell<Option<Rc<Cell<LinkLayerEvent>>>>,
}
impl LinkRenderLayer {
    pub fn new(
        services: Rc<LayerServices>,
        container: HtmlElement,
        z_index: i32,
    ) -> Result<Rc<Self>, JsValue> {
        let base = CanvasLayer::new(services, container, "link", z_index, true)?;
        let this = Rc::new(Self {
            base,
            state: RefCell::new(None),
        });
        let weak = Rc::downgrade(&this);
        this.base.subscribe_theme(move || {
            if let Some(this) = weak.upgrade() {
                this.reset()?;
            }
            Ok(())
        });
        Ok(this)
    }
    pub fn resize(&self, dimensions: RenderDimensions) -> Result<(), JsValue> {
        self.base.resize(dimensions)?;
        self.state.borrow_mut().take();
        Ok(())
    }
    pub fn show(&self, event: Rc<Cell<LinkLayerEvent>>) -> Result<(), JsValue> {
        let color = {
            let colors = self.base.services.theme.colors();
            let colors = colors.borrow();
            match event.get().foreground {
                Some(INVERTED_DEFAULT_COLOR) => colors.background.clone(),
                Some(value) if is_256_color(value) => {
                    colors.ansi.get(value as usize).cloned().ok_or_else(|| {
                        js_sys::TypeError::new(
                            "Cannot read properties of undefined (reading 'css')",
                        )
                    })?
                }
                _ => colors.foreground.clone(),
            }
        };
        self.base
            .fill_style(&JsString::from_char_code(&color.css_units()).into())?;
        let e = event.get();
        if e.y1 == e.y2 {
            self.base.fill_bottom_line(e.x1, e.y1, e.x2 - e.x1)?;
        } else {
            self.base.fill_bottom_line(e.x1, e.y1, e.cols - e.x1)?;
            let mut y = event.get().y1 + 1.0;
            while y < event.get().y2 {
                self.base.fill_bottom_line(0.0, y, event.get().cols)?;
                y += 1.0;
            }
            let e = event.get();
            self.base.fill_bottom_line(0.0, e.y2, e.x2)?;
        }
        *self.state.borrow_mut() = Some(event);
        Ok(())
    }
    fn current(&self) -> Option<LinkLayerEvent> {
        self.state.borrow().as_ref().map(|e| e.get())
    }
    /// Hide ignores the new event and clears the stored show event. Even a
    /// single-row link clears first-to-end plus row-start-to-x2, as upstream.
    pub fn reset(&self) -> Result<(), JsValue> {
        if let Some(e) = self.current() {
            self.base.clear_cells(e.x1, e.y1, e.cols - e.x1, 1.0)?;
            let e = self
                .current()
                .ok_or_else(|| js_sys::TypeError::new("Missing current link"))?;
            let middle = e.y2 - e.y1 - 1.0;
            if middle > 0.0 {
                self.base.clear_cells(0.0, e.y1 + 1.0, e.cols, middle)?;
            }
            let e = self
                .current()
                .ok_or_else(|| js_sys::TypeError::new("Missing current link"))?;
            self.base.clear_cells(0.0, e.y2, e.x2, 1.0)?;
            self.state.borrow_mut().take();
        }
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.base.dispose()
    }
}
impl LinkUnderlineRenderer for LinkRenderLayer {
    fn handle_link_underline(&self, enabled: bool, event: UnderlineEvent) -> Result<(), JsValue> {
        if enabled {
            self.show(Rc::new(Cell::new(event.into())))
        } else {
            self.reset()
        }
    }
}
