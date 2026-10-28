//! Explicit browser-runtime owners for source-shared localization and per-window
//! gesture queues. The portable core has no browser globals.

use crate::{
    accessibility::LocalizableStrings, input::touch::GestureContext,
    links::animation_queue::WindowAnimationQueues,
};
use js_sys::Object;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*};
use web_sys::Document;

#[wasm_bindgen(inline_js = "
export function localizableStrings(pg, ps, og, os) {
  return {
    get promptLabel() { return pg(); },
    set promptLabel(value) { ps(value); },
    get tooMuchOutput() { return og(); },
    set tooMuchOutput(value) { os(value); }
  };
}
")]
extern "C" {
    #[wasm_bindgen(js_name = localizableStrings)]
    fn localizable_strings(pg: &JsValue, ps: &JsValue, og: &JsValue, os: &JsValue) -> Object;
}

pub struct RuntimeContext {
    pub strings: LocalizableStrings,
    gestures: RefCell<Vec<(Document, Rc<GestureContext>)>>,
    queues: RefCell<WindowAnimationQueues>,
    scroll_environment: RefCell<Option<Rc<crate::scroll::environment::ScrollEnvironment>>>,
}
impl Default for RuntimeContext {
    fn default() -> Self {
        Self {
            strings: LocalizableStrings::new(),
            gestures: RefCell::new(Vec::new()),
            queues: RefCell::new(WindowAnimationQueues::default()),
            scroll_environment: RefCell::new(None),
        }
    }
}
impl RuntimeContext {
    pub fn animation_queue(
        &self,
        window: &web_sys::Window,
    ) -> crate::links::animation_queue::WindowAnimationQueue {
        self.queues.borrow_mut().for_window(window)
    }
    pub fn scroll_environment(
        &self,
        platform: xterm_core::platform::Platform,
        window: &web_sys::Window,
    ) -> Rc<crate::scroll::environment::ScrollEnvironment> {
        if let Some(environment) = self.scroll_environment.borrow().as_ref() {
            return Rc::clone(environment);
        }
        let environment = Rc::new(crate::scroll::environment::ScrollEnvironment::new(
            platform,
            window.clone(),
        ));
        *self.scroll_environment.borrow_mut() = Some(Rc::clone(&environment));
        environment
    }
    pub fn gesture(&self, document: &Document) -> Result<Rc<GestureContext>, JsValue> {
        {
            let gestures = self.gestures.borrow();
            if let Some((_, context)) = gestures.iter().find(|(owner, _)| owner == document) {
                return Ok(Rc::clone(context));
            }
        }
        let window = document
            .default_view()
            .ok_or_else(|| js_sys::Error::new("Gesture document has no window"))?;
        let queue = self.queues.borrow_mut().for_window(&window);
        let context = GestureContext::new_with_queue(document.clone(), queue)?;
        self.gestures
            .borrow_mut()
            .push((document.clone(), Rc::clone(&context)));
        Ok(context)
    }
    fn strings_object(&self) -> Object {
        let read = self.strings.clone();
        let prompt_get =
            Closure::<dyn Fn() -> JsValue>::new(move || read.prompt_label_value()).into_js_value();
        let write = self.strings.clone();
        let prompt_set =
            Closure::<dyn Fn(JsValue)>::new(move |value| write.set_prompt_label_value(value))
                .into_js_value();
        let read = self.strings.clone();
        let overflow_get =
            Closure::<dyn Fn() -> JsValue>::new(move || read.too_much_output_value())
                .into_js_value();
        let write = self.strings.clone();
        let overflow_set =
            Closure::<dyn Fn(JsValue)>::new(move |value| write.set_too_much_output_value(value))
                .into_js_value();
        localizable_strings(&prompt_get, &prompt_set, &overflow_get, &overflow_set)
    }
}

#[wasm_bindgen]
pub struct BrowserRuntime {
    pub(crate) context: Rc<RuntimeContext>,
}
#[wasm_bindgen]
impl BrowserRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            context: Rc::new(RuntimeContext::default()),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn strings(&self) -> Object {
        self.context.strings_object()
    }
}
impl Default for BrowserRuntime {
    fn default() -> Self {
        Self::new()
    }
}
