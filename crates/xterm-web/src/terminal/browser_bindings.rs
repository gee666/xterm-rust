use super::{
    browser_services::{BrowserServices, WeakBrowserServices},
    input_host::{EngineInputHost, InputPlatform},
    Shared,
};
fn option_callback(error: wasm_bindgen::JsValue) -> xterm_core::options::OptionError {
    xterm_core::options::OptionError::Dispatch(crate::js_error::callback_error(error))
}

use crate::{
    dom_listener::DomListener,
    input::dom::{InputBindings, InputHost},
    js_error::to_js_error,
    selection::dom::{SelectionDom, SelectionHost},
};
use std::rc::Rc;
use wasm_bindgen::JsValue;

impl BrowserServices {
    pub(crate) fn install(self: &Rc<Self>, shared: &Shared) -> Result<(), JsValue> {
        let links = super::link_services::install(Rc::clone(&shared.link_providers), self)?;
        *self.links.borrow_mut() = Some(links);
        self.install_default_renderer()?;
        let runtime = Rc::clone(&shared.runtime.borrow());
        let browser = Rc::clone(&self.core_browser);
        let source_runtime = Rc::clone(&runtime);
        let queue_source: crate::scroll::browser_scheduler::QueueSource =
            Rc::new(move || Ok(source_runtime.animation_queue(&browser.window())));
        let global_window =
            web_sys::window().ok_or_else(|| js_sys::Error::new("Viewport has no main window"))?;
        let environment = runtime.scroll_environment(self.platform, &global_window);
        let viewport = super::viewport_host::create_viewport(
            self,
            Rc::clone(&self.render_service),
            Rc::clone(&self.core_browser),
            queue_source,
            environment,
        )?;
        self.connect_viewport(viewport)?;
        let weak = Rc::downgrade(self);
        let geometry = Rc::new(move || {
            weak.upgrade()
                .ok_or_else(|| js_sys::Error::new("Selection owner has been disposed"))?
                .mouse_geometry()
        });
        let buffers = self
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        let selection_host: Rc<dyn SelectionHost> =
            Rc::new(WeakBrowserServices(Rc::downgrade(self)));
        let selection = Rc::new(SelectionDom::new(
            &self.dom,
            buffers,
            Rc::clone(&self.selection),
            geometry,
            selection_host,
        )?);
        *self.selection_dom.borrow_mut() = Some(selection);
        self.bind_events()?;
        self.bind_focus()?;
        self.bind_options()?;
        super::accessibility_host::update(self)
    }

    pub fn bind_input(self: &Rc<Self>) -> Result<(), JsValue> {
        let services: Rc<dyn super::input_host::BrowserInputServices> =
            Rc::new(WeakBrowserServices(Rc::downgrade(self)));
        let input: Rc<dyn InputHost> = Rc::new(EngineInputHost::new(
            Rc::clone(&self.native),
            Rc::clone(&self.dom),
            services,
            InputPlatform {
                mac: self.platform.is_mac,
                windows: self.platform.is_windows,
                linux: self.platform.is_linux,
                firefox: self.platform.is_firefox,
            },
        )?);
        let bindings = Rc::new(InputBindings::new_without_mouse(&self.dom, input)?);
        *self.input.borrow_mut() = Some(Rc::clone(&bindings));
        bindings.bind_mouse(&self.dom)
    }

    fn bind_focus(self: &Rc<Self>) -> Result<(), JsValue> {
        for (event_type, focused) in [("focus", true), ("blur", false)] {
            let weak = Rc::downgrade(self);
            let listener = DomListener::new_reentrant_fallible(
                self.dom.textarea.as_ref(),
                event_type,
                false,
                None,
                move |_| {
                    if let Some(services) = weak.upgrade() {
                        services.focus_changed(focused)?;
                    }
                    Ok(())
                },
            )?;
            self.dom_listeners.borrow_mut().push(listener);
        }
        Ok(())
    }

    fn bind_options(self: &Rc<Self>) -> Result<(), JsValue> {
        let options = self
            .native
            .with_state(|state| Rc::clone(&state.options))
            .map_err(|error| to_js_error(&error))?;
        let weak = Rc::downgrade(self);
        let transparency = options
            .try_on_specific_option_change("allowTransparency", move |value| {
                if let Some(services) = weak.upgrade() {
                    services
                        .dom
                        .set_allow_transparency(value.is_truthy())
                        .map_err(option_callback)?;
                }
                Ok(())
            })
            .map_err(|error| to_js_error(&error))?;
        self.listeners.borrow_mut().push(transparency);
        let weak = Rc::downgrade(self);
        let stdin = options
            .try_on_specific_option_change("disableStdin", move |value| {
                if let Some(services) = weak.upgrade() {
                    services.dom.set_disable_stdin(value.is_truthy());
                }
                Ok(())
            })
            .map_err(|error| to_js_error(&error))?;
        self.listeners.borrow_mut().push(stdin);
        let weak = Rc::downgrade(self);
        let accessibility = options
            .try_on_specific_option_change("screenReaderMode", move |_| {
                if let Some(services) = weak.upgrade() {
                    super::accessibility_host::update(&services).map_err(option_callback)?;
                }
                Ok(())
            })
            .map_err(|error| to_js_error(&error))?;
        self.listeners.borrow_mut().push(accessibility);
        let weak = Rc::downgrade(self);
        let contrast = options
            .try_on_specific_option_change("minimumContrastRatio", move |_| {
                if let Some(services) = weak.upgrade() {
                    services.theme.minimum_contrast_ratio_changed();
                }
                Ok(())
            })
            .map_err(|error| to_js_error(&error))?;
        self.listeners.borrow_mut().push(contrast);
        let weak = Rc::downgrade(self);
        let theme = options
            .try_on_specific_option_change("theme", move |value| {
                if let Some(services) = weak.upgrade() {
                    services
                        .theme
                        .set_theme_value(&value)
                        .map_err(|error| option_callback(to_js_error(&error)))?;
                }
                Ok(())
            })
            .map_err(|error| to_js_error(&error))?;
        self.listeners.borrow_mut().push(theme);
        Ok(())
    }
}
