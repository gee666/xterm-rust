// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under MIT. Port of scrollable/horizontalScrollbar.ts.
use super::{
    abstract_scrollbar::{Axis, Inner, Scrollbar, ScrollbarHost, SharedOptions},
    browser_scheduler::BrowserScrollable,
    options::number,
};
use std::{ops::Deref, rc::Rc};
use wasm_bindgen::JsValue;

/// Thin source-named orientation constructor; state and drag code are shared.
#[derive(Clone)]
pub struct HorizontalScrollbar(Scrollbar);
impl HorizontalScrollbar {
    pub fn new(
        model: BrowserScrollable,
        options: SharedOptions,
        host: ScrollbarHost,
        is_windows: bool,
    ) -> Result<Self, JsValue> {
        Scrollbar::new(Axis::Horizontal, model, options, host, is_windows).map(Self)
    }
}
impl Deref for HorizontalScrollbar {
    type Target = Scrollbar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(super) fn initialize(inner: &Rc<Inner>, options: &SharedOptions) -> Result<(), JsValue> {
    // This source check follows super(), including geometry coercion effects.
    if options.borrow().horizontal_has_arrows.is_truthy() {
        return Err(js_sys::Error::new("horizontalHasArrows is not supported in xterm.js").into());
    }
    let (size, slider) = {
        let options = options.borrow();
        (
            options.horizontal_scrollbar_size.clone(),
            options.horizontal_slider_size.clone(),
        )
    };
    let top = ((number(&size)? - number(&slider)?) / 2.0).floor();
    let height = options.borrow().horizontal_slider_size.clone();
    inner.create_slider(top, 0.0, None, Some(height))
}
pub(super) fn update_options(inner: &Inner, options: &SharedOptions) -> Result<(), JsValue> {
    let size = {
        let options = options.borrow();
        if options.horizontal.is_hidden() {
            0.0.into()
        } else {
            options.horizontal_scrollbar_size.clone()
        }
    };
    inner.update_scrollbar_size(&size)?;
    let opposite = {
        let options = options.borrow();
        if options.vertical.is_hidden() {
            0.0.into()
        } else {
            options.vertical_scrollbar_size.clone()
        }
    };
    let opposite = number(&opposite)?;
    inner
        .state
        .borrow_mut()
        .set_opposite_scrollbar_size(opposite);
    let visibility = options.borrow().horizontal.clone();
    inner.visibility.set_visibility(visibility)?;
    inner
        .scroll_by_page
        .set(options.borrow().scroll_by_page.is_truthy());
    Ok(())
}
