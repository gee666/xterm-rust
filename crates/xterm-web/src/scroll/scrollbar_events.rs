// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Slider/track DOM events and pointer-drag policy shared by both orientations.
use super::{Axis, FastDomNode, Inner, V};
use crate::{
    dom_listener::DomListener,
    scroll::{
        browser_scheduler::scroll_error, interop::method0, options::number,
        pointer_monitor::store_listener, state::PositionUpdate, widget,
    },
    terminal::options::from_js,
};
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, HtmlElement, Window};

const POINTER_DRAG_RESET_DISTANCE: f64 = 140.0;

pub(super) fn owner_window(element: &HtmlElement) -> Result<Window, JsValue> {
    element
        .owner_document()
        .and_then(|doc| doc.default_view())
        .or_else(web_sys::window)
        .ok_or_else(|| js_sys::Error::new("scrollbar event requires a window").into())
}
fn property(event: &Event, key: &str) -> Result<V, JsValue> {
    from_js(&Reflect::get(event.as_ref(), &key.into())?)
}
fn primary(event: &Event) -> Result<bool, JsValue> {
    Ok(Reflect::get(event.as_ref(), &"button".into())?.as_f64() == Some(0.0))
}
fn position(axis: Axis, event: &Event) -> Result<V, JsValue> {
    property(
        event,
        match axis {
            Axis::Horizontal => "pageX",
            Axis::Vertical => "pageY",
        },
    )
}
fn orthogonal(axis: Axis, event: &Event) -> Result<V, JsValue> {
    property(
        event,
        match axis {
            Axis::Horizontal => "pageY",
            Axis::Vertical => "pageX",
        },
    )
}

impl Inner {
    pub(in crate::scroll) fn create_slider(
        self: &Rc<Self>,
        top: f64,
        left: f64,
        width: Option<V>,
        height: Option<V>,
    ) -> Result<(), JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| js_sys::Error::new("slider requires a document"))?;
        let slider = FastDomNode::new(document.create_element("div")?.dyn_into::<HtmlElement>()?);
        self.slider
            .set(slider)
            .map_err(|_| js_sys::Error::new("scrollbar slider was initialized twice"))?;
        let slider = self.slider()?;
        slider.set_class_name("xterm-slider")?;
        slider.set_position("absolute")?;
        slider.set_top(&top)?;
        slider.set_left(&left)?;
        // Source explicitly ignores non-number constructor sizes, unlike later
        // _updateScrollbarSize calls which pass raw values to FastDomNode.
        if let Some(V::Number(width)) = width {
            slider.set_width(&width)?;
        }
        if let Some(V::Number(height)) = height {
            slider.set_height(&height)?;
        }
        slider.set_layer_hinting(true)?;
        slider.set_contain("strict")?;
        self.node.dom_node.append_child(&slider.dom_node)?;
        let weak = Rc::downgrade(self);
        let listener = DomListener::new_reentrant_fallible(
            slider.dom_node.as_ref(),
            "pointerdown",
            false,
            None,
            move |event| {
                if let Some(inner) = weak.upgrade() {
                    if primary(&event)? {
                        method0(event.as_ref(), "preventDefault")?;
                        inner.slider_pointer_down(&event)?;
                    }
                }
                Ok(())
            },
        )?;
        store_listener(&self.slider_listeners, listener);
        let fallback = web_sys::window()
            .ok_or_else(|| js_sys::Error::new("slider click requires a window"))?;
        widget::on_click(
            &self.slider_listeners,
            &slider.dom_node,
            &fallback,
            self.host.iframe_cache.clone(),
            Rc::new(|event| {
                if event.left_button {
                    event.stop_propagation()?;
                }
                Ok(())
            }),
        )
    }
    pub(super) fn delegate_pointer_down(self: &Rc<Self>, event: &Event) -> Result<(), JsValue> {
        let rect = self
            .node
            .dom_node
            .get_client_rects()
            .item(0)
            .ok_or_else(|| js_sys::TypeError::new("scrollbar has no client rectangle"))?;
        let (start, stop) = {
            let state = self.state.borrow();
            let start = rect.top() + state.get_slider_position();
            (start, start + state.get_slider_size())
        };
        let pointer = position(self.axis, event)?;
        // Source uses rectangle.top even horizontally and omits arrow size.
        if start <= number(&pointer)? && number(&pointer)? <= stop {
            if primary(event)? {
                method0(event.as_ref(), "preventDefault")?;
                self.slider_pointer_down(event)?;
            }
        } else {
            self.handle_pointer_down(event)?;
        }
        Ok(())
    }
    pub(super) fn handle_pointer_down(self: &Rc<Self>, event: &Event) -> Result<(), JsValue> {
        let target = Reflect::get(event.as_ref(), &"target".into())?;
        let node: &JsValue = self.node.dom_node.as_ref();
        let direct = target == *node;
        let (x, y) = if direct
            && Reflect::get(event.as_ref(), &"offsetX".into())?
                .as_f64()
                .is_some()
            && Reflect::get(event.as_ref(), &"offsetY".into())?
                .as_f64()
                .is_some()
        {
            (property(event, "offsetX")?, property(event, "offsetY")?)
        } else {
            let rect = self.node.dom_node.get_bounding_client_rect();
            let window = owner_window(&self.node.dom_node)?;
            let left = rect.left() + window.scroll_x()?;
            let top = rect.top() + window.scroll_y()?;
            let x = number(&property(event, "pageX")?)? - left;
            let y = number(&property(event, "pageY")?)? - top;
            (x.into(), y.into())
        };
        let raw = match self.axis {
            Axis::Horizontal => x,
            Axis::Vertical => y,
        };
        let needed = self.state.borrow().is_needed();
        let offset = if needed { number(&raw)? } else { 0.0 };
        let desired = {
            let state = self.state.borrow();
            if self.scroll_by_page.get() {
                state.get_desired_scroll_position_from_offset_paged(offset)
            } else {
                state.get_desired_scroll_position_from_offset(offset)
            }
        };
        self.set_desired_position(desired)?;
        if primary(event)? {
            method0(event.as_ref(), "preventDefault")?;
            self.slider_pointer_down(event)?;
        }
        Ok(())
    }
    fn slider_pointer_down(self: &Rc<Self>, event: &Event) -> Result<(), JsValue> {
        if Reflect::get(event.as_ref(), &"target".into())?.is_falsy() {
            return Ok(());
        }
        if Reflect::get(event.as_ref(), &"target".into())?
            .dyn_ref::<Element>()
            .is_none()
        {
            return Ok(());
        }
        let initial = position(self.axis, event)?;
        let initial_other = orthogonal(self.axis, event)?;
        let state = self.state.borrow().clone();
        self.slider()?
            .toggle_class_name("xterm-active", Some(true))?;
        let target = Reflect::get(event.as_ref(), &"target".into())?;
        let pointer_id = Reflect::get(event.as_ref(), &"pointerId".into())?;
        let buttons = Reflect::get(event.as_ref(), &"buttons".into())?;
        let weak = Rc::downgrade(self);
        let stop = Rc::downgrade(self);
        self.monitor.start_monitoring_raw(
            target,
            pointer_id,
            buttons,
            Rc::new(move |event| {
                let Some(inner) = weak.upgrade() else {
                    return Ok(());
                };
                let other = orthogonal(inner.axis, &event)?;
                let delta = (number(&other)? - number(&initial_other)?).abs();
                if inner.is_windows && delta > POINTER_DRAG_RESET_DISTANCE {
                    return inner.set_desired_position(state.get_scroll_position());
                }
                let current = position(inner.axis, &event)?;
                let delta = number(&current)? - number(&initial)?;
                inner.set_desired_position(state.get_desired_scroll_position_from_delta(delta))
            }),
            Rc::new(move || {
                if let Some(inner) = stop.upgrade() {
                    inner
                        .slider()?
                        .toggle_class_name("xterm-active", Some(false))?;
                    (inner.host.drag_end)()?;
                }
                Ok(())
            }),
        )?;
        (self.host.drag_start)()
    }
    fn set_desired_position(&self, position: f64) -> Result<(), JsValue> {
        let mut update = PositionUpdate::default();
        self.axis.write_position(&mut update, position);
        self.model
            .set_scroll_position_now(update)
            .map_err(scroll_error)
    }
}
