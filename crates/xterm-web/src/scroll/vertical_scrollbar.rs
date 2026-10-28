// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under MIT. Port of scrollable/verticalScrollbar.ts.
use super::{
    abstract_scrollbar::{Axis, Inner, Scrollbar, ScrollbarHost, SharedOptions},
    arrow::{ArrowOptions, ScrollbarArrow},
    browser_scheduler::{scroll_error, BrowserScrollable},
    fast_dom_node::{assign_style, begin_style},
    options::number,
    state::PositionUpdate,
};
use std::{ops::Deref, rc::Rc};
use wasm_bindgen::JsValue;
use xterm_core::options::OptionValue as V;

#[derive(Clone)]
pub struct VerticalScrollbar(Scrollbar);
impl VerticalScrollbar {
    pub fn new(
        model: BrowserScrollable,
        options: SharedOptions,
        host: ScrollbarHost,
        is_windows: bool,
    ) -> Result<Self, JsValue> {
        Scrollbar::new(Axis::Vertical, model, options, host, is_windows).map(Self)
    }
}
impl Deref for VerticalScrollbar {
    type Target = Scrollbar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(super) fn initialize(
    inner: &Rc<Inner>,
    options: &SharedOptions,
    has_arrows: V,
) -> Result<(), JsValue> {
    let size = options.borrow().vertical_scrollbar_size.clone();
    set_arrows(inner, has_arrows, &size)?;
    let (size, slider) = {
        let options = options.borrow();
        (
            options.vertical_scrollbar_size.clone(),
            options.vertical_slider_size.clone(),
        )
    };
    let left = ((number(&size)? - number(&slider)?) / 2.0).floor();
    let width = options.borrow().vertical_slider_size.clone();
    inner.create_slider(0.0, left, Some(width), None)
}
pub(super) fn update_options(inner: &Rc<Inner>, options: &SharedOptions) -> Result<(), JsValue> {
    let arrow_size = {
        let options = options.borrow();
        if options.vertical_has_arrows.is_truthy() {
            options.vertical_scrollbar_size.clone()
        } else {
            0.0.into()
        }
    };
    let arrow_size = number(&arrow_size)?;
    inner.state.borrow_mut().set_arrow_size(arrow_size);
    let (show, size) = {
        let options = options.borrow();
        (
            options.vertical_has_arrows.clone(),
            options.vertical_scrollbar_size.clone(),
        )
    };
    set_arrows(inner, show, &size)?;
    let size = {
        let options = options.borrow();
        if options.vertical.is_hidden() {
            0.0.into()
        } else {
            options.vertical_scrollbar_size.clone()
        }
    };
    inner.update_scrollbar_size(&size)?;
    inner.state.borrow_mut().set_opposite_scrollbar_size(0.0);
    let visibility = options.borrow().vertical.clone();
    inner.visibility.set_visibility(visibility)?;
    inner
        .scroll_by_page
        .set(options.borrow().scroll_by_page.is_truthy());
    Ok(())
}
fn set_arrows(inner: &Rc<Inner>, show: V, size: &V) -> Result<(), JsValue> {
    *inner.arrow_delta.borrow_mut() = size.clone();
    if inner.arrows.borrow().len() < 2 {
        create_arrow(inner, size, true)?;
        create_arrow(inner, size, false)?;
    }
    let arrows = inner.arrows.borrow().clone();
    for arrow in &arrows {
        for node in [&arrow.bg_dom_node, &arrow.dom_node] {
            for key in ["width", "height"] {
                // The style getter precedes template-string conversion. The
                // source repeats that conversion for every assignment.
                let style = begin_style(node)?;
                let mut text = size
                    .to_utf16()
                    .map_err(|error| crate::terminal::options::error_to_js(&error))?;
                text.extend([u16::from(b'p'), u16::from(b'x')]);
                assign_style(&style, key, &crate::utf16::to_js_string(&text).into())?;
            }
        }
    }
    let display = if show.is_truthy() { "" } else { "none" };
    for arrow in arrows {
        for node in [&arrow.bg_dom_node, &arrow.dom_node] {
            let style = begin_style(node)?;
            assign_style(&style, "display", &display.into())?;
        }
    }
    Ok(())
}
fn create_arrow(inner: &Rc<Inner>, size: &V, up: bool) -> Result<(), JsValue> {
    let weak = Rc::downgrade(inner);
    let options = ArrowOptions {
        class_name: if up {
            "xterm-scra xterm-arrow-up".into()
        } else {
            "xterm-scra xterm-arrow-down".into()
        },
        top: if up { Some(0.0.into()) } else { None },
        bottom: if up { None } else { Some(0.0.into()) },
        left: Some(0.0.into()),
        right: None,
        bg_width: size.clone(),
        bg_height: size.clone(),
        handle_activate: Rc::new(move || {
            let Some(inner) = weak.upgrade() else {
                return Ok(());
            };
            arrow_scroll(&inner, up)
        }),
    };
    let window =
        web_sys::window().ok_or_else(|| js_sys::Error::new("scrollbar arrow requires a window"))?;
    let arrow = Rc::new(ScrollbarArrow::new(window, options)?);
    if inner.disposed.get() {
        arrow.dispose()?;
    }
    inner.arrows.borrow_mut().push(arrow.clone());
    inner.node.dom_node.append_child(&arrow.bg_dom_node)?;
    inner.node.dom_node.append_child(&arrow.dom_node)?;
    Ok(())
}
fn arrow_scroll(inner: &Inner, up: bool) -> Result<(), JsValue> {
    let raw = inner.arrow_delta.borrow().clone();
    let delta = if up { V::Number(-number(&raw)?) } else { raw };
    // Up's unary minus happens before the current-position lookup. Down keeps
    // JS addition's default-hint concatenation until the numeric model boundary.
    let current = inner.model.current_scroll_position().top;
    let desired = V::Number(current)
        .js_add(&delta)
        .map_err(|error| crate::terminal::options::error_to_js(&error))?;
    let top = number(&desired)?;
    inner
        .model
        .set_scroll_position_now(PositionUpdate {
            top: Some(top),
            left: None,
        })
        .map_err(scroll_error)
}
