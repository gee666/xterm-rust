// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_addons::attach::*;
use xterm_core::lifecycle::{to_disposable, Disposable, DisposableHandle};

type Entry = (SocketEvent, SocketCallback, bool);
#[derive(Default)]
struct State {
    entries: RefCell<Vec<Entry>>,
    trace: RefCell<Vec<String>>,
    failure: RefCell<Option<(SocketEvent, bool, AttachError)>>,
    replacement: Cell<bool>,
    sent: RefCell<Vec<String>>,
}
struct Socket(Rc<State>);
fn event_name(event: SocketEvent) -> &'static str {
    match event {
        SocketEvent::Message => "message",
        SocketEvent::Close => "close",
        SocketEvent::Error => "error",
    }
}
impl State {
    fn callback(&self, event: SocketEvent) -> SocketCallback {
        self.entries
            .borrow()
            .iter()
            .find(|entry| entry.0 == event)
            .unwrap()
            .1
            .clone()
    }
}
impl AttachSocket for Socket {
    fn use_array_buffer(&self) -> Result<(), AttachError> {
        Ok(())
    }
    fn state(&self) -> Result<SocketState, AttachError> {
        Ok(SocketState::Open)
    }
    fn send(&self, value: SocketMessage) -> Result<(), AttachError> {
        let SocketMessage::Text(text) = value else {
            panic!("expected text")
        };
        self.0
            .sent
            .borrow_mut()
            .push(String::from_utf16(&text).unwrap());
        Ok(())
    }
    fn listen(
        &self,
        event: SocketEvent,
        callback: SocketCallback,
    ) -> Result<AttachListener, AttachError> {
        let index = self.0.entries.borrow().len();
        self.0.entries.borrow_mut().push((event, callback, true));
        let state = self.0.clone();
        Ok(AttachListener::new(move || {
            if state.replacement.get() {
                state
                    .trace
                    .borrow_mut()
                    .push(format!("replacement:{}", event_name(event)));
                if event == SocketEvent::Message {
                    state.callback(event)(Some(SocketMessage::Text(vec![66])))?;
                }
                return Ok(());
            }
            state.trace.borrow_mut().push("get".into());
            let failure = state.failure.borrow().clone();
            if let Some((kind, true, error)) = &failure {
                if *kind == event {
                    return Err(error.clone());
                }
            }
            state.trace.borrow_mut().push(event_name(event).into());
            if let Some((kind, false, error)) = failure {
                if kind == event {
                    return Err(error);
                }
            }
            state.entries.borrow_mut()[index].2 = false;
            Ok(())
        }))
    }
    fn warn_closing(&self) -> Result<(), AttachError> {
        panic!("open socket")
    }
}
type DataEntries = Rc<RefCell<Vec<Option<DataCallback>>>>;
#[derive(Default)]
struct Terminal {
    writes: RefCell<Vec<u16>>,
    data: DataEntries,
    binary: DataEntries,
}
fn subscribe(entries: &DataEntries, callback: DataCallback) -> DisposableHandle {
    let index = entries.borrow().len();
    entries.borrow_mut().push(Some(callback));
    let entries = entries.clone();
    to_disposable(move || entries.borrow_mut()[index] = None)
}
impl AttachTerminal for Terminal {
    fn write(&self, value: SocketMessage) -> Result<(), AttachError> {
        let SocketMessage::Text(text) = value else {
            panic!("expected text")
        };
        self.writes.borrow_mut().extend(text);
        Ok(())
    }
    fn on_data(&self, callback: DataCallback) -> DisposableHandle {
        subscribe(&self.data, callback)
    }
    fn on_binary(&self, callback: DataCallback) -> DisposableHandle {
        subscribe(&self.binary, callback)
    }
}
fn kind(name: &str) -> SocketEvent {
    match name {
        "message" => SocketEvent::Message,
        "close" => SocketEvent::Close,
        "error" => SocketEvent::Error,
        _ => panic!("fixture event"),
    }
}
fn active(state: &State) -> Vec<&'static str> {
    state
        .entries
        .borrow()
        .iter()
        .filter(|entry| entry.2)
        .map(|entry| event_name(entry.0))
        .collect()
}

#[test]
fn fallible_removal_matches_all_36_unchanged_source_traces() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../xterm-web/tests/fixtures/attach-removal.json"
    ))
    .unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 36);
    for row in rows {
        let parts: Vec<_> = row["name"].as_str().unwrap().split('/').collect();
        let state = Rc::new(State::default());
        let error = AttachError::host(std::io::Error::other("exact host identity"));
        *state.failure.borrow_mut() = Some((kind(parts[1]), parts[2] == "getter", error.clone()));
        let addon =
            AttachAddon::new(Rc::new(Socket(state.clone())), Some(parts[3] == "true")).unwrap();
        let terminal = Rc::new(Terminal::default());
        addon.activate(terminal.clone()).unwrap();
        let result = if parts[0] == "dispose" {
            addon.try_dispose()
        } else {
            state.callback(kind(parts[0]))(None)
        };
        assert_eq!(result, Err(error), "{}", row["name"]);
        assert_eq!(serde_json::json!(active(&state)), row["activeAfterFailure"]);
        state.callback(SocketEvent::Message)(Some(SocketMessage::Text(vec![65]))).unwrap();
        let data = terminal.data.borrow().clone();
        for callback in data.into_iter().flatten() {
            callback(&[120]).unwrap();
        }
        assert_eq!(
            String::from_utf16(&terminal.writes.borrow()).unwrap(),
            row["text"]
        );
        *state.failure.borrow_mut() = None;
        addon.try_dispose().unwrap();
        assert_eq!(serde_json::json!(active(&state)), row["activeAfterSuccess"]);
        state.replacement.set(true);
        addon.try_dispose().unwrap();
        assert_eq!(
            serde_json::json!(*state.trace.borrow()),
            row["trace"],
            "{}",
            row["name"]
        );
        assert_eq!(serde_json::json!(*state.sent.borrow()), row["sent"]);
        assert_eq!(
            String::from_utf16(&terminal.writes.borrow()).unwrap(),
            row["repeatedText"]
        );
    }
}

#[test]
fn infallible_compatibility_disposal_records_and_returns_exact_host_error() {
    let state = Rc::new(State::default());
    let error = AttachError::host(std::io::Error::other("removal"));
    *state.failure.borrow_mut() = Some((SocketEvent::Message, false, error.clone()));
    let addon = AttachAddon::new(Rc::new(Socket(state.clone())), Some(false)).unwrap();
    let terminal = Rc::new(Terminal::default());
    addon.activate(terminal).unwrap();
    addon.dispose();
    assert_eq!(addon.take_dispose_error(), Some(error));
    assert_eq!(addon.take_dispose_error(), None);
    *state.failure.borrow_mut() = None;
    addon.dispose();
    assert_eq!(addon.take_dispose_error(), None);
    assert!(active(&state).is_empty());
}
