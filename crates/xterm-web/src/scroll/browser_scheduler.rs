//! Animation delivery for Scrollable using the browser's shared window queue.
use super::scrollable::{ScrollError, Scrollable, WeakScrollable};
use crate::{
    js_error::to_js_error,
    links::{
        animation_queue::{AnimationItem, WindowAnimationQueue},
        debounce::FrameScheduler,
    },
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;
use xterm_core::lifecycle::Disposable;

type Target = Rc<RefCell<Option<WeakScrollable<BrowserScrollScheduler>>>>;
pub type QueueSource = Rc<dyn Fn() -> Result<WindowAnimationQueue, JsValue>>;
pub struct BrowserScrollScheduler {
    queue: QueueSource,
    target: Target,
}
pub type BrowserScrollable = Scrollable<BrowserScrollScheduler>;

impl FrameScheduler for BrowserScrollScheduler {
    type Handle = AnimationItem;
    type Error = JsValue;
    fn request_frame(&mut self) -> Result<AnimationItem, JsValue> {
        let target = self.target.clone();
        (self.queue)()?.schedule(0.0, move |_| {
            let model = target.borrow().as_ref().and_then(WeakScrollable::upgrade);
            if let Some(model) = model {
                model.frame().map_err(scroll_error)?;
            }
            Ok(())
        })
    }
    fn cancel_frame(&mut self, handle: AnimationItem) -> Result<(), JsValue> {
        handle.dispose();
        Ok(())
    }
    fn cleanup_error(&mut self, error: JsValue) {
        web_sys::console::error_1(&error);
    }
}

pub fn new_scrollable(
    queue: WindowAnimationQueue,
    integer_values: bool,
    duration: f64,
) -> BrowserScrollable {
    new_scrollable_with_queue_source(Rc::new(move || Ok(queue.clone())), integer_values, duration)
}

/// Viewport uses the current CoreBrowserService window at each animation request,
/// while ordinary component callers can keep the original fixed queue constructor.
pub fn new_scrollable_with_queue_source(
    queue: QueueSource,
    integer_values: bool,
    duration: f64,
) -> BrowserScrollable {
    let target = Rc::new(RefCell::new(None));
    let model = Scrollable::new(
        BrowserScrollScheduler {
            queue,
            target: target.clone(),
        },
        integer_values,
        duration,
        js_sys::Date::now,
    );
    *target.borrow_mut() = Some(model.downgrade());
    model
}
pub fn scroll_error(error: ScrollError<JsValue>) -> JsValue {
    match error {
        ScrollError::Scheduler(error) => error,
        ScrollError::Dispatch(error) => to_js_error(&error),
    }
}
