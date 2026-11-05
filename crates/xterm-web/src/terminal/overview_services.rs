use super::browser_services::BrowserServices;
use crate::{
    js_error::to_js_error,
    renderer::overview_ruler::{OverviewRulerRenderer, OverviewServices},
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::options::{OptionError, OptionValue};

impl BrowserServices {
    pub(crate) fn install_overview(self: &Rc<Self>) -> Result<(), JsValue> {
        let options = self
            .native
            .with_state(|state| Rc::clone(&state.options))
            .map_err(|error| to_js_error(&error))?;
        let show = property(
            &options
                .get("scrollbar")
                .map_err(|error| to_js_error(&error))?,
            "showScrollbar",
        )?;
        let show = matches!(show, OptionValue::Undefined | OptionValue::Null) || show.is_truthy();
        // Open reads width even when hidden. Read the live option again because
        // the visibility getter can replace it. Later changes short-circuit.
        let width = property(
            &options
                .get("scrollbar")
                .map_err(|error| to_js_error(&error))?,
            "width",
        )?;
        self.ensure_overview(show && width.is_truthy())?;
        let weak = Rc::downgrade(self);
        let listener = options
            .try_on_specific_option_change("scrollbar", move |value| {
                if let Some(services) = weak.upgrade() {
                    services.update_overview(&value).map_err(|error| {
                        OptionError::Dispatch(crate::js_error::callback_error(error))
                    })?;
                }
                Ok(())
            })
            .map_err(|error| to_js_error(&error))?;
        self.listeners.borrow_mut().push(listener);
        Ok(())
    }

    fn update_overview(&self, scrollbar: &OptionValue) -> Result<(), JsValue> {
        let show = property(scrollbar, "showScrollbar")?;
        let show = matches!(show, OptionValue::Undefined | OptionValue::Null) || show.is_truthy();
        self.ensure_overview(show && property(scrollbar, "width")?.is_truthy())
    }

    fn ensure_overview(&self, should_show: bool) -> Result<(), JsValue> {
        // Source keeps an existing renderer when hidden; its listener zeroes width.
        if should_show && self.overview.borrow().is_none() {
            let (buffer, options) = self
                .native
                .with_state(|state| (Rc::clone(&state.buffer_service), Rc::clone(&state.options)))
                .map_err(|error| to_js_error(&error))?;
            let renderer = OverviewRulerRenderer::new(
                &self.dom.viewport,
                &self.dom.document,
                OverviewServices {
                    buffer,
                    decorations: self.decorations.as_ref().clone(),
                    render: Rc::clone(&self.render_service),
                    options,
                    theme: Rc::clone(&self.theme),
                    browser: self.core_browser.clone(),
                },
            )?;
            *self.overview.borrow_mut() = Some(renderer);
        }
        Ok(())
    }
}

fn property(value: &OptionValue, name: &str) -> Result<OptionValue, JsValue> {
    if matches!(value, OptionValue::Undefined | OptionValue::Null) {
        Ok(OptionValue::Undefined)
    } else {
        value
            .get_property(name)
            .map_err(|error| to_js_error(&error))
    }
}
