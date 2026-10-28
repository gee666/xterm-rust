// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use serde_json::{json, Value};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_addons::clipboard::*;
use xterm_core::lifecycle::{DisposableHandle, NoopDisposable};

type Events = Rc<RefCell<Vec<Value>>>;
struct Provider {
    events: Events,
    asynchronous: bool,
}
impl ClipboardProvider for Provider {
    fn read_text(&self, selection: &[u16]) -> Completion<Vec<u16>> {
        self.events.borrow_mut().push(json!(["read", selection]));
        let value = "€mläütß".encode_utf16().collect();
        if self.asynchronous {
            Completion::Pending(Box::pin(async move { Ok(value) }))
        } else {
            Completion::Ready(Ok(value))
        }
    }
    fn write_text(&self, selection: &[u16], text: &[u16]) -> Completion<()> {
        self.events
            .borrow_mut()
            .push(json!(["write", selection, text]));
        if self.asynchronous {
            Completion::Pending(Box::pin(async { Ok(()) }))
        } else {
            Completion::Ready(Ok(()))
        }
    }
}
struct Terminal(Events);
impl ClipboardTerminal for Terminal {
    fn register_osc_handler(&self, _: u16, _: OscCallback) -> DisposableHandle {
        Rc::new(NoopDisposable)
    }
    fn input(&self, data: &[u16], user: bool) -> Result<(), ClipboardError> {
        self.0.borrow_mut().push(json!(["input", data, user]));
        Ok(())
    }
}
struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}
fn settle(
    mut future: Pin<Box<dyn Future<Output = Result<bool, ClipboardError>>>>,
) -> Result<bool, ClipboardError> {
    let waker = Waker::from(Arc::new(NoWake));
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture future must resolve on first poll"),
    }
}

#[test]
fn clipboard_source_provider_order_and_exact_osc_replies() {
    let corpus: Value = serde_json::from_str(include_str!("fixtures/utility-addons.json")).unwrap();
    let cases = corpus["clipboard"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    for (i, case) in cases.iter().enumerate() {
        let events = Events::default();
        let provider = Rc::new(Provider {
            events: events.clone(),
            asynchronous: case["async"].as_bool().unwrap(),
        });
        let terminal = Rc::new(Terminal(events.clone()));
        let data = case["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| u16::try_from(v.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let result = handle_osc(&data, Rc::new(Base64), provider, terminal);
        assert_eq!(
            json!(*events.borrow()),
            case["immediate"],
            "immediate case {i}"
        );
        let result = match result {
            Completion::Ready(value) => value,
            Completion::Pending(future) => settle(future),
        };
        assert_eq!(result, Ok(true));
        assert_eq!(json!(*events.borrow()), case["events"], "settled case {i}");
    }
}

struct RejectProvider;
impl ClipboardProvider for RejectProvider {
    fn read_text(&self, _: &[u16]) -> Completion<Vec<u16>> {
        Completion::Pending(Box::pin(async {
            Err(ClipboardError::Message("permission denied".into()))
        }))
    }
    fn write_text(&self, _: &[u16], _: &[u16]) -> Completion<()> {
        Completion::Ready(Err(ClipboardError::Message("permission denied".into())))
    }
}
#[test]
fn clipboard_provider_rejections_are_not_success() {
    for data in ["c;?", "c;aA=="] {
        let terminal = Rc::new(Terminal(Events::default()));
        let completion = handle_osc(
            &data.encode_utf16().collect::<Vec<_>>(),
            Rc::new(Base64),
            Rc::new(RejectProvider),
            terminal.clone(),
        );
        let result = match completion {
            Completion::Ready(value) => value,
            Completion::Pending(future) => settle(future),
        };
        assert_eq!(
            result,
            Err(ClipboardError::Message("permission denied".into()))
        );
        assert!(terminal.0.borrow().is_empty());
    }
}
