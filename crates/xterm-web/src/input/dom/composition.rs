// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from browser/input/CompositionHelper.ts.

use super::{listen, read_value, InputBindings, State};
use crate::input::composition::Deferred;
use crate::input::composition_geometry::composition_geometry;
use crate::input::keyboard_dispatch::KeyEventSource;
use crate::utf16::to_js_string;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};

pub(super) fn bind(bindings: &mut InputBindings) -> Result<(), JsValue> {
    let area = bindings.state.textarea.clone();
    listen(
        bindings,
        area.as_ref(),
        "compositionstart",
        false,
        |state, _| {
            if !state.composition.borrow().is_composing {
                state.host.sync_textarea()?;
            }
            state.composition.borrow_mut().begin_composition();
            let start = match state.textarea.selection_start()? {
                Some(start) => start as usize,
                None => read_value(&state.textarea)?.len(),
            };
            let end = state
                .textarea
                .selection_end()?
                .map_or(start, |end| end as usize);
            state.composition.borrow_mut().set_start_range(start, end);
            let value = read_value(&state.textarea)?;
            state
                .composition
                .borrow_mut()
                .finish_composition_start(&value);
            state.overlay.set_text_content(Some(""));
            state.overlay.class_list().add_1("active")?;
            update_elements(state, true)
        },
    )?;
    listen(
        bindings,
        area.as_ref(),
        "compositionupdate",
        false,
        |state, event| {
            let raw = js_sys::Reflect::get(event.as_ref(), &JsValue::from_str("data"))?;
            let data = crate::terminal::options::from_js(&raw)?
                .to_utf16()
                .map_err(|error| crate::terminal::options::error_to_js(&error))?;
            let action = state.composition.borrow_mut().update(&data);
            let overlay = state.composition.borrow().overlay_text.clone();
            if !js_sys::Reflect::set(
                state.overlay.as_ref(),
                &JsValue::from_str("textContent"),
                &to_js_string(&overlay),
            )? {
                return Err(js_sys::TypeError::new("Cannot set composition text").into());
            }
            update_elements(state, true)?;
            schedule(state, action)
        },
    )?;
    listen(
        bindings,
        area.as_ref(),
        "compositionend",
        false,
        |state, _| {
            let action = state.composition.borrow_mut().end();
            state.overlay.class_list().remove_1("active")?;
            schedule(state, action)
        },
    )?;
    Ok(())
}

pub(super) fn keydown_source(
    state: &Rc<State>,
    event: &dyn KeyEventSource<Error = JsValue>,
) -> Result<bool, JsValue> {
    let pending = state.composition.borrow().has_pending_composition();
    if pending {
        if event.key_code()? == 20.0 || event.key_code()? == 229.0 {
            return Ok(false);
        }
        if event.key_code()? == 16.0 || event.key_code()? == 17.0 || event.key_code()? == 18.0 {
            return Ok(false);
        }
        state.overlay.class_list().remove_1("active")?;
        state.composition.borrow_mut().cancel_for_immediate_send();
        let value = read_value(&state.textarea)?;
        let data = state.composition.borrow().immediate_text(&value);
        state.host.data(&data, true)?;
    }
    // The data callback above may throw or change keyCode. Preserve this final
    // source read after it, without a borrowed model spanning host code.
    if event.key_code()? == 229.0 {
        let pending = state.composition.borrow().is_textarea_change_pending();
        if !pending {
            let old_value = read_value(&state.textarea)?;
            schedule(state, Deferred::TextareaChange { old_value })?;
            state.composition.borrow_mut().textarea_change_scheduled();
        }
        return Ok(false);
    }
    Ok(true)
}

fn schedule(state: &Rc<State>, action: Deferred) -> Result<(), JsValue> {
    let weak = Rc::downgrade(state);
    state.timers.schedule(move || {
        let Some(state) = weak.upgrade() else {
            return Ok(());
        };
        if state.disposed.get() {
            return Ok(());
        }
        let _host = state.host.retain_host()?;
        let data = match action {
            Deferred::UpdateEnd => {
                let end = match state.textarea.selection_end()? {
                    Some(end) => end as usize,
                    None => read_value(&state.textarea)?.len(),
                };
                state.composition.borrow_mut().update_end(end);
                None
            }
            Deferred::Send { start, suffix } => {
                let start = state.composition.borrow_mut().begin_deferred_send(start);
                let Some(start) = start else {
                    return Ok(());
                };
                let composing = state.composition.borrow().is_composing;
                let value = read_value(&state.textarea)?;
                state
                    .composition
                    .borrow()
                    .text_for_send(start, &suffix, &value, composing)
            }
            Deferred::TextareaChange { old_value } => {
                let apply = state.composition.borrow_mut().begin_textarea_change();
                if !apply {
                    return Ok(());
                }
                let value = read_value(&state.textarea)?;
                state
                    .composition
                    .borrow_mut()
                    .textarea_diff(&old_value, &value)
            }
        };
        if let Some(data) = data {
            state.host.data(&data, true)?;
        }
        Ok(())
    })
}

pub(super) fn update_elements(state: &Rc<State>, recurse: bool) -> Result<(), JsValue> {
    if !state.composition.borrow().is_composing {
        return Ok(());
    }
    let layout = state.host.overlay_layout()?;
    if layout.cursor.in_viewport {
        // Overlay bounds must be measured after its cursor/font styles are set.
        let cursor = layout.cursor;
        if cursor.cols == 0 {
            return Err(js_sys::Error::new("Composition requires positive columns").into());
        }
        let style = state.overlay.style();
        for (key, value) in [
            (
                "left",
                format!(
                    "{}px",
                    cursor.x.min(cursor.cols - 1) as f64 * cursor.cell_width
                ),
            ),
            ("top", format!("{}px", cursor.y as f64 * cursor.cell_height)),
            ("height", format!("{}px", cursor.cell_height)),
            ("line-height", format!("{}px", cursor.cell_height)),
        ] {
            style.set_property(key, &value)?;
        }
        let set_property = js_sys::Reflect::get(style.as_ref(), &JsValue::from_str("setProperty"))?
            .dyn_into::<js_sys::Function>()?;
        set_property.call2(
            style.as_ref(),
            &JsValue::from_str("font-family"),
            &to_js_string(&layout.font_family),
        )?;
        for (key, value) in [
            ("font-size", format!("{}px", layout.font_size)),
            (
                "max-width",
                format!(
                    "{}px",
                    (cursor.cols - cursor.x.min(cursor.cols - 1)) as f64 * cursor.cell_width
                ),
            ),
            ("overflow", "hidden".into()),
            ("direction", "rtl".into()),
        ] {
            style.set_property(key, &value)?;
        }
        let bounds = state.overlay.get_bounding_client_rect();
        let geometry = composition_geometry(crate::input::composition_geometry::CursorGeometry {
            overlay_width: bounds.width(),
            overlay_height: bounds.height(),
            ..cursor
        })
        .ok_or_else(|| js_sys::Error::new("Invalid composition geometry"))?;
        let style = state.textarea.style();
        for (key, value) in [
            ("left", geometry.left),
            ("top", geometry.top),
            ("width", geometry.textarea_width),
            ("height", geometry.textarea_height),
            ("line-height", geometry.textarea_line_height),
        ] {
            style.set_property(key, &format!("{value}px"))?;
        }
    }
    if recurse {
        let weak = Rc::downgrade(state);
        state.timers.schedule(move || {
            if let Some(state) = weak.upgrade() {
                if !state.disposed.get() {
                    let _host = state.host.retain_host()?;
                    return update_elements(&state, false);
                }
            }
            Ok(())
        })?;
    }
    Ok(())
}
