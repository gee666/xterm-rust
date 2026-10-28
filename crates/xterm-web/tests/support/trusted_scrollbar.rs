//! Test-only DOM fixture for Playwright's trusted pointer input. It owns the
//! same Rust scrollbars/model as the assertion suite, not a JavaScript replica.
use super::{observe, parts, vertical, Fixture};
use std::{ops::Deref, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use xterm_web::scroll::{
    abstract_scrollbar::Scrollbar, horizontal_scrollbar::HorizontalScrollbar,
    vertical_scrollbar::VerticalScrollbar,
};

enum Bar {
    Horizontal(Fixture<HorizontalScrollbar>),
    Vertical(Fixture<VerticalScrollbar>),
}
impl Bar {
    fn bar(&self) -> &Scrollbar {
        match self {
            Self::Horizontal(f) => f.bar.deref().deref(),
            Self::Vertical(f) => f.bar.deref().deref(),
        }
    }
}
#[wasm_bindgen]
pub struct TrustedScrollbarFixture {
    bar: Bar,
}
#[wasm_bindgen]
impl TrustedScrollbarFixture {
    #[wasm_bindgen(constructor)]
    pub fn new(horizontal: bool) -> Self {
        let bar = if horizontal {
            let (model, options, host, starts, ends, parent) = parts();
            let bar = Rc::new(
                HorizontalScrollbar::new(model.clone(), options.clone(), host, true).unwrap(),
            );
            parent.append_child(&bar.dom_node()).unwrap();
            bar.render().unwrap();
            let subscription = observe(&model, &bar);
            Bar::Horizontal(Fixture {
                bar,
                model,
                options,
                parent,
                starts,
                ends,
                subscription,
            })
        } else {
            Bar::Vertical(vertical(true, false, false))
        };
        bar.bar().begin_reveal().unwrap();
        Self { bar }
    }
    #[wasm_bindgen(getter)]
    pub fn slider(&self) -> Result<HtmlElement, JsValue> {
        self.bar.bar().slider_node()
    }
    #[wasm_bindgen(getter)]
    pub fn element(&self) -> HtmlElement {
        self.bar.bar().dom_node()
    }
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> f64 {
        match &self.bar {
            Bar::Horizontal(f) => f.model.current_scroll_position().left,
            Bar::Vertical(f) => f.model.current_scroll_position().top,
        }
    }
    #[wasm_bindgen(getter)]
    pub fn starts(&self) -> usize {
        match &self.bar {
            Bar::Horizontal(f) => f.starts.get(),
            Bar::Vertical(f) => f.starts.get(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn ends(&self) -> usize {
        match &self.bar {
            Bar::Horizontal(f) => f.ends.get(),
            Bar::Vertical(f) => f.ends.get(),
        }
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.bar.bar().dispose()
    }
}
