// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from browser/input/CompositionHelper.ts.

use super::{listen, snapshot, InputBindings, State};
use crate::input::composition::{Deferred, KeydownResult};
use crate::input::composition_geometry::composition_geometry;
use crate::utf16::{from_js_string, to_js_string};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CompositionEvent;

pub(super) fn bind(bindings: &mut InputBindings) -> Result<(), JsValue> {
    let area = bindings.state.textarea.clone();
    listen(
        bindings,
        area.as_ref(),
        "compositionstart",
        false,
        |state, _| {
            state.host.sync_textarea()?;
            state
                .composition
                .borrow_mut()
                .start(&snapshot(&state.textarea)?);
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
            let event: CompositionEvent = event.dyn_into()?;
            let data = js_sys::Reflect::get(event.as_ref(), &JsValue::from_str("data"))?
                .dyn_into::<js_sys::JsString>()?;
            let action = state
                .composition
                .borrow_mut()
                .update(&from_js_string(&data));
            let overlay = state.composition.borrow().overlay_text.clone();
            if !js_sys::Reflect::set(
                state.overlay.as_ref(),
                &JsValue::from_str("textContent"),
                &to_js_string(&overlay),
            )? {
                return Err(js_sys::Error::new("Cannot set composition text").into());
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

pub(super) fn keydown(state: &Rc<State>, code: u32) -> Result<bool, JsValue> {
    let KeydownResult {
        continue_processing,
        data,
        deferred,
    } = state
        .composition
        .borrow_mut()
        .keydown(code, &snapshot(&state.textarea)?);
    if !state.composition.borrow().is_composing {
        state.overlay.class_list().remove_1("active")?;
    }
    if let Some(data) = data {
        state.host.data(&data, true);
    }
    if let Some(action) = deferred {
        schedule(state, action)?;
    }
    Ok(continue_processing)
}

fn schedule(state: &Rc<State>, action: Deferred) -> Result<(), JsValue> {
    let weak = Rc::downgrade(state);
    state.timers.schedule(move || {
        if let Some(state) = weak.upgrade() {
            if state.disposed.get() {
                return;
            }
            let result = (|| {
                let snapshot = snapshot(&state.textarea)?;
                let data = state
                    .composition
                    .borrow_mut()
                    .run_deferred(action, &snapshot);
                if let Some(data) = data {
                    state.host.data(&data, true);
                }
                Ok::<_, JsValue>(())
            })();
            if let Err(error) = result {
                state.host.report_error(error);
            }
        }
    })
}

pub(super) fn update_elements(state: &Rc<State>, recurse: bool) -> Result<(), JsValue> {
    if !state.composition.borrow().is_composing {
        return Ok(());
    }
    let layout = state.host.overlay_layout();
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
            ("font-family", layout.font_family),
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
                    if let Err(error) = update_elements(&state, false) {
                        state.host.report_error(error);
                    }
                }
            }
        })?;
    }
    Ok(())
}
