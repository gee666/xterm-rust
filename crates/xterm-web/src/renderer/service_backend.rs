use super::*;
use crate::renderer::service_api::RendererChangeAction;
use xterm_core::lifecycle::{to_disposable, DisposableHandle};
type Interceptor = Rc<dyn Fn() -> Result<RendererChangeAction, JsValue>>;
pub(super) type RendererInterceptors = Rc<RefCell<Vec<Interceptor>>>;
impl RenderService {
    /// Native equivalent of addons wrapping setRenderer. Newest wrappers run
    /// first, before backend disposal and even for equal assignments. A thrown
    /// value stops the call. A wrapper disposed during its own callback can
    /// stop delegation, as the source's cleared old-method slot does.
    /// Registration changes affect the next invocation.
    pub fn intercept_renderer_change(
        &self,
        callback: impl Fn() -> Result<RendererChangeAction, JsValue> + 'static,
    ) -> DisposableHandle {
        let callback: Interceptor = Rc::new(callback);
        self.before_set_renderer
            .borrow_mut()
            .push(Rc::clone(&callback));
        let hooks = Rc::downgrade(&self.before_set_renderer);
        to_disposable(move || {
            if let Some(hooks) = hooks.upgrade() {
                hooks
                    .borrow_mut()
                    .retain(|entry| !Rc::ptr_eq(entry, &callback));
            }
        })
    }

    pub fn set_renderer(self: &Rc<Self>, renderer: Rc<dyn Renderer>) -> Result<(), JsValue> {
        let hooks = self.before_set_renderer.borrow().clone();
        for hook in hooks.iter().rev() {
            if hook()? == RendererChangeAction::Stop {
                return Ok(());
            }
        }
        if !self.disposed.get() {
            let old = self.current_renderer();
            if !old.as_ref().is_some_and(|old| Rc::ptr_eq(old, &renderer)) {
                if let Some(old) = old {
                    old.dispose()?;
                }
                *self.renderer.borrow_mut() = Some(renderer);
            }
            // MutableDisposable skips equal assignment, but source still wires
            // another redraw listener and requests a full selection refresh.
            if let Some(renderer) = self.current_renderer() {
                let weak = Rc::downgrade(self);
                // The renderer owns this event's lifetime, as in source. Dropping
                // a native subscription handle does not unregister its callback.
                let _subscription = renderer.on_request_redraw().try_subscribe(move |event| {
                    if let Some(service) = weak.upgrade() {
                        service
                            .refresh_rows(event.start, event.end, event.sync, true)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                });
                self.selection_dirty.set(true);
                self.full_refresh()?;
            }
        }
        Ok(())
    }
}
