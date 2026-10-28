use super::{browser_services::BrowserServices, Terminal};
use crate::{
    dom::{BrowserDom, DomOptions},
    js_error::to_js_error,
};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Document, HtmlElement};

#[wasm_bindgen]
impl Terminal {
    pub fn open(&self, parent: JsValue) -> Result<(), JsValue> {
        if !parent.is_truthy() {
            return Err(js_sys::Error::new("Terminal requires a parent element.").into());
        }
        // Upstream uses a TypeScript cast, not a realm-local instanceof check.
        // A legitimate iframe HTMLElement has a different global constructor.
        let parent: HtmlElement = parent.unchecked_into();
        if self.shared.dom.borrow().is_some() {
            return Ok(());
        }
        let options = &self.shared.options;
        let override_value = options
            .service()
            .get("documentOverride")
            .map_err(|error| to_js_error(&error))?;
        let document_override = super::options::into_js(&override_value)?
            .dyn_into::<Document>()
            .ok();
        let strings = self.shared.runtime.borrow().strings.clone();
        let dom = Rc::new(BrowserDom::new(
            &parent,
            &DomOptions {
                allow_transparency: options
                    .service()
                    .get("allowTransparency")
                    .map_err(|error| to_js_error(&error))?
                    .is_truthy(),
                disable_stdin: options
                    .service()
                    .get("disableStdin")
                    .map_err(|error| to_js_error(&error))?
                    .is_truthy(),
                is_chrome_os: self.shared.platform.is_chrome_os,
                prompt_label: strings.prompt_label_value(),
                document_override,
            },
        )?);
        *self.shared.dom.borrow_mut() = Some(Rc::clone(&dom));
        let services = BrowserServices::new(&self.shared, dom)?;
        *self.shared.services.borrow_mut() = Some(Rc::clone(&services));
        services.install(&self.shared)?;
        services.metrics.measure()?;
        let rows = self
            .shared
            .native
            .rows()
            .map_err(|error| to_js_error(&error))?;
        services.refresh(0.0, (rows - 1) as f64)?;
        services.bind_input()
    }

    pub fn refresh(&self, start: f64, end: f64) -> Result<(), JsValue> {
        crate::api::verify_integers(&[start, end]).map_err(|error| to_js_error(&error))?;
        let services = self.shared.services.borrow().clone();
        if let Some(services) = services {
            services.refresh(start, end)?;
        }
        Ok(())
    }

    #[wasm_bindgen(getter)]
    pub fn dimensions(&self) -> Result<JsValue, JsValue> {
        match self.shared.services.borrow().as_ref() {
            Some(services) => {
                super::events::dimensions_value(&services.render_service.dimensions()?)
            }
            None => Ok(JsValue::UNDEFINED),
        }
    }

    #[wasm_bindgen(js_name = attachCustomKeyEventHandler)]
    pub fn attach_custom_key_event_handler(&self, handler: js_sys::Function) {
        *self.shared.custom_key.borrow_mut() = Some(handler);
    }
    #[wasm_bindgen(js_name = attachCustomWheelEventHandler)]
    pub fn attach_custom_wheel_event_handler(&self, handler: js_sys::Function) {
        *self.shared.custom_wheel.borrow_mut() = Some(handler);
    }

    pub fn paste(&self, data: js_sys::JsString) -> Result<(), JsValue> {
        let services = self
            .shared
            .services
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("Terminal must be opened first"))?;
        let input = services
            .input
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("Input is not initialized"))?;
        input.paste(&crate::utf16::from_js_string(&data))
    }
}
impl Terminal {
    pub fn render_dimensions(&self) -> Option<crate::api::RenderDimensions> {
        self.shared
            .services
            .borrow()
            .as_ref()
            .and_then(|services| services.render_service.dimensions().ok())
    }
    pub fn linkifier_handle(
        &self,
    ) -> Option<Rc<crate::links::browser_linkifier::BrowserLinkifier>> {
        self.shared
            .services
            .borrow()
            .as_ref()
            .and_then(|services| services.links.borrow().clone())
    }
}
