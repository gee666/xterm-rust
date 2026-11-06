// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{super::js, State};
use crate::js_error::{callback_error, to_js_error};
use xterm_addons::image::policy::{
    GraphicsGeometry, ImageControlAction, ImageControlEffect, ImageControlPolicy, ImageParameter,
};
use xterm_core::parser::{
    handler::{DispatchError, HandlerResult},
    params::ParamValue,
};

const DEC_SIXEL_DISPLAY: f64 = 80.0;

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
        if matches!(control, Control::Graphics) {
            if let Some(result) = self.palette_control(&parameters) {
                return result;
            }
            if let Some(result) = self.pixel_control(&parameters) {
                return result;
            }
        }
        let mut policy = ImageControlPolicy::new(self.defaults);
        policy.options = self.options.get();
        let action = match control {
            Control::Set => policy.decset(&parameters),
            Control::Reset => policy.decrst(&parameters),
            Control::Attributes => policy.primary_device_attributes(&parameters),
            Control::Graphics => {
                policy.graphics_attributes(&parameters, GraphicsGeometry::default())
            }
            Control::SoftReset => policy.reset(),
        };
        // These assignments precede storage/handler callbacks, just as source
        // mutates _opts before reset can reenter the addon.
        match control {
            Control::SoftReset => {
                self.flags.scrolling.replace(self.default_scrolling.clone());
                self.palette.reset();
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
    pub(super) fn execute(&self, action: ImageControlAction) -> Result<bool, JsValue> {
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
