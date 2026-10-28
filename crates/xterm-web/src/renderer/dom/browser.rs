use super::*;
impl DomRenderer {
    pub(super) fn browser_dpr(&self) -> Result<f64, JsValue> {
        if let Some(browser) = &self.browser {
            return browser.dpr();
        }
        js_sys::Reflect::get(self.window.as_ref(), &"devicePixelRatio".into())?
            .as_f64()
            .ok_or_else(|| js_sys::TypeError::new("devicePixelRatio must be numeric").into())
    }
    pub(super) fn browser_focused(&self) -> Result<bool, JsValue> {
        match &self.browser {
            Some(browser) => browser.is_focused(),
            None => Ok(self.focused.get()),
        }
    }
    pub(super) fn window_source(&self) -> Rc<dyn Fn() -> Window> {
        match &self.browser {
            Some(browser) => {
                let browser = Rc::clone(browser);
                Rc::new(move || browser.window())
            }
            None => {
                let window = self.window.clone();
                Rc::new(move || window.clone())
            }
        }
    }
}
