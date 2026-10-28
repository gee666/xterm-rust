// Copyright (c) 2019-2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Shared OSC/APC callback trace assertions.
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use xterm_core::parser::handler::HandlerResult;
use xterm_core::parser::osc::{OscAction, OscHandlerTrait};
use xterm_core::text_decoder::utf32_to_string;

pub type Log = Rc<RefCell<Vec<String>>>;
pub fn units(data: &[u16]) -> String {
    data.iter()
        .map(|unit| format!("{unit:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
#[derive(Clone)]
pub struct Reporter {
    pub name: String,
    pub handled: bool,
    pub log: Log,
}
impl OscHandlerTrait for Reporter {
    fn start(&mut self) {
        self.log.borrow_mut().push(format!("h:{}", self.name));
    }
    fn put(&mut self, data: &[u32]) {
        self.log
            .borrow_mut()
            .push(format!("p:{}:{}", self.name, units(&utf32_to_string(data))));
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.log
            .borrow_mut()
            .push(format!("e:{}:{}", self.name, u8::from(success)));
        HandlerResult::Ready(self.handled)
    }
}
pub fn report(log: &Log, id: impl Display, action: OscAction) {
    let event = match action {
        OscAction::Start => "s".to_string(),
        OscAction::Put(data) => format!("p:{}", units(&data)),
        OscAction::End(success) => format!("e:{}", u8::from(success)),
    };
    log.borrow_mut().push(format!("f:{id}:{event}"));
}
