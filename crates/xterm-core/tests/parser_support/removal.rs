// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Shared callback-driven registration mutation fixture.
use std::cell::RefCell;
use std::rc::Weak;
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::handler::HandlerResult;
use xterm_core::parser::osc::OscHandlerTrait;

pub struct RemoveAll {
    pub phase: &'static str,
    pub tokens: Weak<RefCell<Vec<DisposableHandle>>>,
}
impl RemoveAll {
    fn remove(&self, phase: &str) {
        if self.phase == phase {
            for token in self
                .tokens
                .upgrade()
                .expect("test owns token list")
                .borrow()
                .iter()
            {
                token.dispose();
            }
        }
    }
}
impl OscHandlerTrait for RemoveAll {
    fn start(&mut self) {
        self.remove("start");
    }
    fn put(&mut self, _: &[u32]) {
        self.remove("put");
    }
    fn end(&mut self, _: bool) -> HandlerResult {
        self.remove("end");
        HandlerResult::Ready(false)
    }
}
