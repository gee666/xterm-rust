// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    super::{js, terminal_host::TerminalCanvasHost, ImageCanvasHost},
    State,
};
use crate::js_error::{callback_error, to_js_error};
use xterm_addons::image::{
    policy::{
        GraphicsGeometry, ImageControlAction, ImageControlEffect, ImageControlPolicy,
        ImageParameter,
    },
    storage::CellSize,
};
use xterm_core::parser::{
    handler::{DispatchError, HandlerResult},
    params::ParamValue,
};

const DEC_SIXEL_DISPLAY: f64 = 80.0;
const GRAPHICS_SIXEL_GEOMETRY: f64 = 2.0;
const GRAPHICS_READ: f64 = 1.0;

#[derive(Clone, Copy)]
pub(super) enum Control {
    Set,
    Reset,
    Attributes,
    Graphics,
    SoftReset,
}
impl State {
    pub(super) fn reset(&self) -> Result<bool, JsValue> {
        self.control(Control::SoftReset, Vec::new())
    }
    pub(super) fn control(
        &self,
        control: Control,
        params: Vec<ParamValue>,
    ) -> Result<bool, JsValue> {
        let parameters: Vec<_> = params
            .into_iter()
            .map(|param| match param {
                ParamValue::Param(n) => ImageParameter::Number(n as f64),
                ParamValue::SubParams(values) => {
                    ImageParameter::Subparams(values.into_iter().map(|n| n as f64).collect())
                }
            })
            .collect();
        let mut policy = ImageControlPolicy::new(self.defaults);
        policy.options = self.options.get();
        let action = match control {
            Control::Set => policy.decset(&parameters),
            Control::Reset => policy.decrst(&parameters),
            Control::Attributes => policy.primary_device_attributes(&parameters),
            Control::Graphics => {
                let mut geometry = GraphicsGeometry::default();
                // Only READ SIXEL_GEO touches live geometry in the source.
                if parameters.first() == Some(&ImageParameter::Number(GRAPHICS_SIXEL_GEOMETRY))
                    && parameters.get(1) == Some(&ImageParameter::Number(GRAPHICS_READ))
                {
                    if let Some(terminal) = self.terminal.borrow().upgrade() {
                        geometry.columns =
                            terminal.native.cols().map_err(|e| to_js_error(&e))? as f64;
                        geometry.rows = terminal.native.rows().map_err(|e| to_js_error(&e))? as f64;
                        if let Some(dimensions) =
                            TerminalCanvasHost(Rc::downgrade(&terminal)).dimensions()?
                        {
                            geometry.canvas = Some(CellSize {
                                width: dimensions.canvas_width,
                                height: dimensions.canvas_height,
                            });
                        }
                    }
                }
                policy.graphics_attributes(&parameters, geometry)
            }
            Control::SoftReset => policy.reset(),
        };
        // These assignments precede storage/handler callbacks, just as source
        // mutates _opts before reset can reenter the addon.
        match control {
            Control::SoftReset => {
                self.flags.scrolling.replace(self.default_scrolling.clone());
            }
            Control::Set | Control::Reset
                if parameters.contains(&ImageParameter::Number(DEC_SIXEL_DISPLAY)) =>
            {
                self.flags
                    .scrolling
                    .replace(matches!(control, Control::Reset).into());
            }
            _ => {}
        }
        self.options.set(policy.options);
        self.execute(action)
    }
    fn execute(&self, action: ImageControlAction) -> Result<bool, JsValue> {
        for effect in action.effects {
            match effect {
                ImageControlEffect::Reply(data) => {
                    let terminal = self.terminal.borrow().upgrade();
                    if let Some(terminal) = terminal {
                        terminal
                            .native
                            .input(&data, false)
                            .map_err(|e| to_js_error(&e))?;
                    }
                }
                ImageControlEffect::ClearStorage => {
                    if let Some(storage) = self.storage() {
                        storage.reset().map_err(|e| to_js_error(&e))?;
                    }
                }
                ImageControlEffect::ResetHandlers => self.reset_handlers()?,
            }
        }
        Ok(action.handled)
    }
}
use std::rc::Rc;
use wasm_bindgen::JsValue;
pub(super) fn result(result: Result<bool, JsValue>) -> HandlerResult {
    match result {
        Ok(value) => HandlerResult::Ready(value),
        Err(error) => HandlerResult::Failed(DispatchError::Callback(callback_error(error))),
    }
}
pub(super) fn enable_reports(terminal: &crate::terminal::Terminal) -> Result<(), JsValue> {
    let options: JsValue = terminal.options().into();
    let mut reports = js::get(&options, "windowOptions")?;
    if js::nullish(&reports) {
        reports = js_sys::Object::new().into();
    }
    for key in ["getWinSizePixels", "getCellSizePixels", "getWinSizeChars"] {
        js::set(&reports, key, &true.into())?;
    }
    js::set(&options, "windowOptions", &reports)
}
