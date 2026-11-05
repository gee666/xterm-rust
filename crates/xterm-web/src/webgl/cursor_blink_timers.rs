// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;

impl State {
    pub(super) fn callback(self: &Rc<Self>, kind: Callback) -> JsValue {
        let weak = Rc::downgrade(self);
        // JS owns the function while the timer host retains it. Its weak state
        // prevents a callback -> manager -> browser-service retention cycle.
        Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
            let Some(state) = weak.upgrade() else {
                return Ok(());
            };
            match kind {
                Callback::Render => {
                    (state.render)()?;
                    state.frame.replace(JsValue::UNDEFINED);
                    Ok(())
                }
                Callback::Start => state.start_tick(),
                Callback::Interval => state.interval_tick(),
                Callback::Idle => state.stop_idle(),
            }
        })
        .into_js_value()
    }

    pub(super) fn restart_interval(self: &Rc<Self>, delay: JsValue) -> Result<(), JsValue> {
        self.clear_handle(&self.interval, "clearInterval")?;
        let window = get(&self.service, "window")?;
        let callback = self.callback(Callback::Start);
        self.start
            .replace(method2(&window, "setTimeout", &callback, &delay)?);
        Ok(())
    }

    fn start_tick(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.restarted.borrow().is_truthy() {
            let remaining = self.remaining()?;
            if positive(&remaining)? {
                self.restart_interval(remaining)?;
                return Ok(());
            }
        }
        self.visible.set(false);
        self.request_frame()?;
        let window = get(&self.service, "window")?;
        let callback = self.callback(Callback::Interval);
        self.interval.replace(method2(
            &window,
            "setInterval",
            &callback,
            &BLINK_INTERVAL_MS.into(),
        )?);
        Ok(())
    }

    fn interval_tick(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.restarted.borrow().is_truthy() {
            let remaining = self.remaining()?;
            // Unlike the initial timeout, source forwards zero/negative delays.
            self.restart_interval(remaining)?;
            return Ok(());
        }
        self.visible.set(!self.visible.get());
        self.request_frame()
    }

    pub(super) fn reset_idle(self: &Rc<Self>) -> Result<(), JsValue> {
        self.idle_paused.set(false);
        if self.idle.borrow().is_truthy() {
            let window = get(&self.service, "window")?;
            let method = get(&window, "clearTimeout")?;
            let handle = self.idle.borrow().clone();
            call1(&method, &window, &handle)?;
        }
        let window = get(&self.service, "window")?;
        let callback = self.callback(Callback::Idle);
        self.idle.replace(method2(
            &window,
            "setTimeout",
            &callback,
            &IDLE_TIMEOUT_MS.into(),
        )?);
        Ok(())
    }

    fn stop_idle(&self) -> Result<(), JsValue> {
        self.visible.set(true);
        self.idle_paused.set(true);
        self.clear_handle(&self.interval, "clearInterval")?;
        self.clear_handle(&self.start, "clearTimeout")?;
        self.clear_handle(&self.frame, "cancelAnimationFrame")?;
        // Idle completion always clears this handle, even if it is undefined.
        let window = get(&self.service, "window")?;
        let method = get(&window, "clearTimeout")?;
        let handle = self.idle.borrow().clone();
        call1(&method, &window, &handle)?;
        self.idle.replace(JsValue::UNDEFINED);
        (self.render)()?;
        Ok(())
    }
}
