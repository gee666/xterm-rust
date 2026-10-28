// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. DCS upstream test helpers.

use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_core::parser::dcs::{DcsAction, DcsHandlerTrait, DcsParser, HandlerHandle};
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::params::{ParamValue, Params};

pub const IDENT: i32 = 0x2b70;
pub type Reports = Rc<RefCell<Vec<String>>>;

pub fn params(values: &[i32]) -> Params {
    Params::from_array(
        &values
            .iter()
            .copied()
            .map(ParamValue::Param)
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

pub fn units(data: &[u16]) -> String {
    data.iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn params_text(params: &Params) -> String {
    params
        .values()
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn input(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

pub fn parser() -> (DcsParser, Reports) {
    let mut parser = DcsParser::new();
    let reports: Reports = Rc::default();
    let output = Rc::clone(&reports);
    parser.set_handler_fallback(move |ident, action| {
        let action = match action {
            DcsAction::Hook(params) => format!("H:{}", params_text(&params)),
            DcsAction::Put(data) => format!("P:{}", units(&data)),
            DcsAction::Unhook(success) => format!("U:{success}"),
        };
        output.borrow_mut().push(format!("{ident}:{action}"));
    });
    (parser, reports)
}

struct TestHandler {
    reports: Reports,
    label: String,
    handled: bool,
    asynchronous: bool,
}

impl DcsHandlerTrait for TestHandler {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError> {
        self.reports
            .borrow_mut()
            .push(format!("{}:H:{}", self.label, params_text(params)));
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.reports.borrow_mut().push(format!(
            "{}:P:{}",
            self.label,
            units(&xterm_core::text_decoder::utf32_to_string(data))
        ));
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        let report = format!("{}:U:{success}", self.label);
        let handled = self.handled;
        if self.asynchronous {
            let output = Rc::clone(&self.reports);
            HandlerResult::Pending(Box::pin(async move {
                std::future::ready(()).await;
                output.borrow_mut().push(report);
                handled
            }))
        } else {
            self.reports.borrow_mut().push(report);
            HandlerResult::Ready(handled)
        }
    }
}

pub fn handler(reports: &Reports, label: &str, handled: bool, asynchronous: bool) -> HandlerHandle {
    Rc::new(RefCell::new(TestHandler {
        reports: Rc::clone(reports),
        label: label.into(),
        handled,
        asynchronous,
    }))
}

struct NoopWake;
impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

pub fn resolve(result: HandlerResult) -> bool {
    resolve_result(result).unwrap()
}

pub fn resolve_result(result: HandlerResult) -> Result<bool, DispatchError> {
    let mut future = Box::pin(result.resolve());
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    // Test futures are ready on their first poll, with no external IO.
    match Future::poll(future.as_mut(), &mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("test future unexpectedly required external scheduling"),
    }
}

pub fn unhook(parser: &mut DcsParser, success: bool) {
    let mut previous = true;
    while let Some(result) = parser.unhook(success, previous).unwrap() {
        previous = resolve(result);
    }
}

pub fn payload(parser: &mut DcsParser) {
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.put(&input("Here comes")).unwrap();
    parser.put(&input("the mouse!")).unwrap();
}

pub fn expected(labels: &[&str], handled: bool) -> Vec<String> {
    let mut result = Vec::new();
    for label in labels {
        result.push(format!("{label}:H:1,2,3"));
    }
    for text in ["Here comes", "the mouse!"] {
        for label in labels {
            result.push(format!(
                "{label}:P:{}",
                units(&text.encode_utf16().collect::<Vec<_>>())
            ));
        }
    }
    for (index, label) in labels.iter().enumerate() {
        result.push(format!("{label}:U:{}", index == 0 || !handled));
    }
    result
}
