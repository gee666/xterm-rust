//! Source-shared wheel classification and iframe cache, owned by BrowserRuntime.
use super::{
    iframe::IframeCache, interop, wheel_classifier::MouseWheelClassifier,
    wheel_event::StandardWheelEvent,
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::Window;
use xterm_core::platform::Platform;

const LAST_DPR_FACTOR_CHROME_MAJOR: u32 = 122;

pub struct ScrollEnvironment {
    pub platform: Platform,
    pub main_window: Window,
    pub iframe: Rc<IframeCache>,
    classifier: RefCell<MouseWheelClassifier>,
}
impl ScrollEnvironment {
    pub fn new(platform: Platform, main_window: Window) -> Self {
        Self {
            platform,
            main_window,
            iframe: Rc::new(IframeCache::new()),
            classifier: RefCell::new(MouseWheelClassifier::default()),
        }
    }
    pub(crate) fn factor_legacy_dpr(&self) -> Result<bool, JsValue> {
        if !self.platform.is_chrome {
            return Ok(false);
        }
        let agent = self.main_window.navigator().user_agent()?;
        // Regex Chrome/(\d+) chooses the first occurrence with actual digits,
        // not merely the first Chrome/ substring. Overflow is necessarily >122.
        for (index, _) in agent.match_indices("Chrome/") {
            let text = &agent[index + "Chrome/".len()..];
            let count = text.bytes().take_while(u8::is_ascii_digit).count();
            if count != 0 {
                return Ok(text[..count]
                    .parse::<u32>()
                    .is_ok_and(|version| version <= LAST_DPR_FACTOR_CHROME_MAJOR));
            }
        }
        // Source defaults an unrecognized Chrome version to 123.
        Ok(false)
    }
    pub(crate) fn accept_wheel(&self, event: &StandardWheelEvent) -> Result<(), JsValue> {
        let (x, y) = if self.platform.is_chrome {
            let raw = event
                .browser_event
                .as_ref()
                .map_or(JsValue::NULL, |event| event.clone().into());
            let _target_window = interop::event_window(&raw, &self.main_window)?;
            // Supplied Platform.getZoomFactor always returns one. Still read the
            // event window and sample Date.now at the source call sites.
            let timestamp = interop::clock()?.as_f64().unwrap_or(f64::NAN);
            let one = JsValue::from_f64(1.0);
            let x = interop::multiply(&event.delta_x, &one)?;
            let y = interop::multiply(&event.delta_y, &one)?;
            let x = crate::terminal::options::from_js(&x)?;
            let y = crate::terminal::options::from_js(&y)?;
            self.classifier
                .borrow_mut()
                .accept_values(timestamp, x, y)
                .map_err(|error| crate::terminal::options::error_to_js(&error))?;
            return Ok(());
        } else {
            (&event.delta_x, &event.delta_y)
        };
        let timestamp = interop::clock()?.as_f64().unwrap_or(f64::NAN);
        let x = crate::terminal::options::from_js(x)?;
        let y = crate::terminal::options::from_js(y)?;
        self.classifier
            .borrow_mut()
            .accept_values(timestamp, x, y)
            .map_err(|error| crate::terminal::options::error_to_js(&error))
    }
    pub(crate) fn is_physical_mouse_wheel(&self) -> bool {
        self.classifier.borrow().is_physical_mouse_wheel()
    }
}
