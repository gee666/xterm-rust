// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

/// An engine call retains its receiver even if a host callback activates the
/// addon elsewhere. Its cache and subscriptions must never migrate to that host.
pub(super) struct EngineContext {
    pub host: Rc<dyn SearchHost>,
    pub cache: RefCell<SearchLineCache>,
    timeout: RefCell<Option<DisposableHandle>>,
    listener: RefCell<Option<DisposableHandle>>,
    disposed: Cell<bool>,
}
impl EngineContext {
    pub fn new(host: Rc<dyn SearchHost>) -> Rc<Self> {
        Rc::new(Self {
            host,
            cache: RefCell::new(SearchLineCache::default()),
            timeout: RefCell::new(None),
            listener: RefCell::new(None),
            disposed: Cell::new(false),
        })
    }
    pub fn dispose(&self) {
        self.disposed.set(true);
        self.invalidate();
    }
    fn invalidate(&self) {
        self.cache.borrow_mut().invalidate();
        SearchAddon::cancel(&self.timeout);
        SearchAddon::cancel(&self.listener);
    }
    pub fn init(self: &Rc<Self>) -> Result<(), SearchAddonError> {
        let now = self.host.now_ms();
        self.cache.borrow_mut().init(now);
        // A captured call may continue after replacement. Retired contexts
        // must not install jobs whose handles would have no live owner. Source
        // post-disposal registration behavior remains a documented parity gap.
        if self.disposed.get() {
            return Ok(());
        }
        if self.listener.borrow().is_none() {
            let weak = Rc::downgrade(self);
            let listener = self.host.terminal().on_event().try_subscribe(move |event| {
                if matches!(
                    event,
                    TerminalEvent::LineFeed
                        | TerminalEvent::CursorMove
                        | TerminalEvent::Resize { .. }
                ) {
                    if let Some(engine) = weak.upgrade() {
                        engine.invalidate();
                    }
                }
                Ok(())
            });
            *self.listener.borrow_mut() = Some(listener);
        }
        if self.timeout.borrow().is_none() {
            self.schedule_timeout()?;
        }
        Ok(())
    }
    fn schedule_timeout(self: &Rc<Self>) -> Result<(), SearchAddonError> {
        let deadline = self.cache.borrow().deadline_ms();
        if let Some(deadline) = deadline {
            let weak = Rc::downgrade(self);
            let timer = self.host.timeout(
                deadline.saturating_sub(self.host.now_ms()),
                Rc::new(move || {
                    if let Some(engine) = weak.upgrade() {
                        SearchAddon::cancel(&engine.timeout);
                        engine.cache.borrow_mut().expire(engine.host.now_ms());
                        if engine.cache.borrow().deadline_ms().is_none() {
                            engine.invalidate();
                        } else {
                            engine.schedule_timeout()?;
                        }
                    }
                    Ok(())
                }),
            )?;
            if self.disposed.get() {
                timer.dispose();
            } else {
                *self.timeout.borrow_mut() = Some(timer);
            }
        }
        Ok(())
    }
}
impl Drop for EngineContext {
    fn drop(&mut self) {
        self.dispose();
    }
}
