use super::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Window};
impl RenderService {
    pub fn register_observer(
        self: &Rc<Self>,
        window: &Window,
        screen: &HtmlElement,
    ) -> Result<(), JsValue> {
        if !js_sys::Reflect::has(window.as_ref(), &"IntersectionObserver".into())? {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let callback: ObserverCallback = Closure::wrap(Box::new(move |entries: js_sys::Array| {
            if let Some(service) = weak.upgrade() {
                if entries.length() == 0 {
                    return Err(js_sys::TypeError::new(
                        "IntersectionObserver callback requires an entry",
                    )
                    .into());
                }
                service.intersection_changed(entries.get(entries.length() - 1))?;
            }
            Ok(())
        }));
        let constructor = js_sys::Reflect::get(window.as_ref(), &"IntersectionObserver".into())?;
        if !constructor.is_function() {
            return Err(js_sys::TypeError::new("IntersectionObserver is not a constructor").into());
        }
        let constructor: js_sys::Function = constructor.unchecked_into();
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"threshold".into(), &0.0.into())?;
        let args = js_sys::Array::new();
        args.push(callback.as_ref());
        args.push(options.as_ref());
        // Construct in the actual owner window. A parent-realm instanceof test
        // would reject a valid iframe observer returned by this constructor.
        let observer: IntersectionObserver =
            js_sys::Reflect::construct(&constructor, &args)?.unchecked_into();
        self.disconnect_observer();
        *self.observer_callback.borrow_mut() = Some(callback);
        *self.observer.borrow_mut() = Some(observer.clone());
        observer.observe(screen.as_ref());
        Ok(())
    }
    fn intersection_changed(&self, entry: JsValue) -> Result<(), JsValue> {
        let intersecting = js_sys::Reflect::get(&entry, &"isIntersecting".into())?;
        let paused = if intersecting.is_undefined() {
            js_sys::Reflect::get(&entry, &"intersectionRatio".into())?.as_f64() == Some(0.0)
        } else {
            intersecting.is_falsy()
        };
        self.paused.set(paused);
        if let Some(renderer) = self.current_renderer() {
            if let Some(visibility) = renderer.visibility() {
                visibility.handle_viewport_visibility_change(!paused)?;
            }
        }
        if !paused && !self.metrics.state.has_valid_size() {
            self.metrics.measure()?;
        }
        if !paused && self.needs_full.get() {
            self.paused_resize.flush()?;
            self.refresh_rows(0.0, self.rows.get() as f64 - 1.0, false, false)?;
            self.needs_full.set(false);
        }
        Ok(())
    }
    pub(super) fn disconnect_observer(&self) {
        let observer = self.observer.borrow_mut().take();
        if let Some(observer) = observer {
            observer.disconnect();
        }
        self.observer_callback.borrow_mut().take();
    }
    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }
}
