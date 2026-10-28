// Copyright (c) 2024 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::{options, styles, viewport_error, Cleanup, Viewport};
use crate::{
    js_error::{callback_error, to_js_error},
    scroll::{element::SmoothScrollableElement, state::DimensionUpdate},
    terminal::options::set_data,
};
use js_sys::Object;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
use xterm_core::mouse_state::MouseEventTypes;

pub(super) fn initialize(viewport: &Rc<Viewport>, screen: HtmlElement) -> Result<(), JsValue> {
    let model = viewport.model.clone();
    let smooth = viewport
        .host
        .services
        .options
        .try_on_specific_option_change("smoothScrollDuration", move |value| {
            let duration = value.to_number()?;
            model.set_smooth_scroll_duration(duration);
            Ok(())
        })
        .map_err(|error| to_js_error(&error))?;
    viewport.listener(smooth);

    let element = SmoothScrollableElement::new(
        screen.clone(),
        options::initial(&viewport.host.services.options)?,
        viewport.model.clone(),
        Rc::clone(&viewport.host.services.environment),
    )?;
    *viewport.host.element.borrow_mut() = Some(Rc::clone(&element));
    viewport.track(Cleanup::Element(Rc::clone(&element)));
    let weak = Rc::downgrade(viewport);
    let change = viewport
        .host
        .services
        .options
        .on_any_option_change()
        .try_subscribe(move |key| {
            if matches!(
                key.as_str(),
                "scrollSensitivity" | "fastScrollSensitivity" | "scrollbar"
            ) {
                if let Some(viewport) = weak.upgrade() {
                    options::update(
                        &viewport.host.element().map_err(callback_error)?,
                        &viewport.host.services.options,
                    )
                    .map_err(callback_error)?;
                }
            }
            Ok(())
        });
    viewport.listener(change);
    let weak = Rc::downgrade(&element);
    let protocol = viewport
        .host
        .services
        .mouse
        .on_protocol_change()
        .try_subscribe(move |events| {
            if let Some(element) = weak.upgrade() {
                let options = Object::new();
                let wheel = events.0 & MouseEventTypes::WHEEL.0 != 0;
                set_data(&options, "handleMouseWheel", &JsValue::from_bool(!wheel))
                    .map_err(callback_error)?;
                element
                    .update_options(options.as_ref())
                    .map_err(callback_error)?;
            }
            Ok(())
        });
    viewport.listener(protocol);

    element.set_scroll_dimensions(DimensionUpdate {
        height: Some(0.0),
        scroll_height: Some(0.0),
        ..DimensionUpdate::default()
    })?;
    styles::apply_background(viewport)?;
    let weak = Rc::downgrade(viewport);
    viewport.listener(
        viewport
            .host
            .services
            .theme
            .on_change_colors()
            .try_subscribe(move |_| {
                if let Some(viewport) = weak.upgrade() {
                    styles::apply_background(&viewport).map_err(callback_error)?;
                }
                Ok(())
            }),
    );
    let node = element.get_dom_node()?;
    viewport.root.append_child(&node)?;
    viewport.track(Cleanup::Node(node));
    let style = viewport
        .host
        .services
        .main_document
        .create_element("style")?
        .dyn_into::<HtmlElement>()?;
    screen.append_child(&style)?;
    *viewport.style.borrow_mut() = Some(style.clone());
    viewport.track(Cleanup::Node(style));
    styles::apply_rules(viewport)?;
    let weak = Rc::downgrade(viewport);
    viewport.listener(
        viewport
            .host
            .services
            .theme
            .on_change_colors()
            .try_subscribe(move |_| {
                if let Some(viewport) = weak.upgrade() {
                    styles::apply_rules(&viewport).map_err(callback_error)?;
                }
                Ok(())
            }),
    );
    events(viewport, &element);
    Ok(())
}

fn events(viewport: &Rc<Viewport>, element: &Rc<SmoothScrollableElement>) {
    let weak = Rc::downgrade(viewport);
    viewport.listener(
        viewport
            .host
            .services
            .buffers
            .on_resize()
            .try_subscribe(move |_| {
                if let Some(viewport) = weak.upgrade() {
                    viewport.queue_sync(None).map_err(callback_error)?;
                }
                Ok(())
            }),
    );
    let weak = Rc::downgrade(viewport);
    viewport.listener(
        viewport
            .host
            .services
            .buffers
            .buffers
            .on_buffer_activate()
            .try_subscribe(move |_| {
                if let Some(viewport) = weak.upgrade() {
                    viewport
                        .controller
                        .buffer_activated()
                        .map_err(|error| callback_error(viewport_error(error)))?;
                }
                Ok(())
            }),
    );
    let weak = Rc::downgrade(viewport);
    viewport.listener(
        viewport
            .host
            .services
            .buffers
            .on_scroll()
            .try_subscribe(move |_| {
                if let Some(viewport) = weak.upgrade() {
                    viewport.sync(None).map_err(callback_error)?;
                }
                Ok(())
            }),
    );
    let weak = Rc::downgrade(viewport);
    viewport.listener(
        viewport
            .host
            .services
            .render
            .on_render()
            .try_subscribe(move |_| {
                if let Some(viewport) = weak.upgrade() {
                    viewport
                        .controller
                        .on_render()
                        .map_err(|error| callback_error(viewport_error(error)))?;
                }
                Ok(())
            }),
    );
    let weak = Rc::downgrade(viewport);
    viewport.listener(element.on_scroll().try_subscribe(move |event| {
        if let Some(viewport) = weak.upgrade() {
            viewport
                .controller
                .handle_scroll(event.current.top)
                .map_err(|error| callback_error(viewport_error(error)))?;
        }
        Ok(())
    }));
}
