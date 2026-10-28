// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_addons::attach::*;
use xterm_core::lifecycle::{to_disposable, Disposable, DisposableHandle};

type SocketSlots = Rc<RefCell<Vec<Option<(SocketEvent, SocketCallback)>>>>;
#[derive(Default)]
struct Socket {
    state: Cell<Option<SocketState>>,
    binary: Cell<bool>,
    sent: RefCell<Vec<SocketMessage>>,
    listeners: SocketSlots,
    warnings: Cell<usize>,
    state_error: RefCell<Option<AttachError>>,
    warning_error: RefCell<Option<AttachError>>,
}
impl AttachSocket for Socket {
    fn use_array_buffer(&self) -> Result<(), AttachError> {
        self.binary.set(true);
        Ok(())
    }
    fn state(&self) -> Result<SocketState, AttachError> {
        if let Some(error) = self.state_error.borrow().clone() {
            Err(error)
        } else {
            Ok(self.state.get().unwrap_or(SocketState::Open))
        }
    }
    fn send(&self, data: SocketMessage) -> Result<(), AttachError> {
        self.sent.borrow_mut().push(data);
        Ok(())
    }
    fn listen(
        &self,
        event: SocketEvent,
        callback: SocketCallback,
    ) -> Result<DisposableHandle, AttachError> {
        let index = self.listeners.borrow().len();
        self.listeners.borrow_mut().push(Some((event, callback)));
        let listeners = self.listeners.clone();
        Ok(to_disposable(move || {
            listeners.borrow_mut()[index] = None;
        }))
    }
    fn warn_closing(&self) -> Result<(), AttachError> {
        self.warnings.set(self.warnings.get() + 1);
        if let Some(error) = self.warning_error.borrow().clone() {
            Err(error)
        } else {
            Ok(())
        }
    }
}
impl Socket {
    fn fire(&self, event: SocketEvent, data: Option<SocketMessage>) -> Result<(), AttachError> {
        let snapshot = self.listeners.borrow().clone();
        for (kind, callback) in snapshot.into_iter().flatten() {
            if kind == event {
                callback(data.clone())?;
            }
        }
        Ok(())
    }
}
type DataSlots = Rc<RefCell<Vec<Option<DataCallback>>>>;
#[derive(Default)]
struct Terminal {
    writes: RefCell<Vec<SocketMessage>>,
    data: DataSlots,
    binary: DataSlots,
}
fn subscribe(slots: &DataSlots, callback: DataCallback) -> DisposableHandle {
    let index = slots.borrow().len();
    slots.borrow_mut().push(Some(callback));
    let slots = slots.clone();
    to_disposable(move || {
        slots.borrow_mut()[index] = None;
    })
}
impl AttachTerminal for Terminal {
    fn write(&self, data: SocketMessage) -> Result<(), AttachError> {
        self.writes.borrow_mut().push(data);
        Ok(())
    }
    fn on_data(&self, callback: DataCallback) -> DisposableHandle {
        subscribe(&self.data, callback)
    }
    fn on_binary(&self, callback: DataCallback) -> DisposableHandle {
        subscribe(&self.binary, callback)
    }
}
fn fire(slots: &DataSlots, data: &[u16]) -> Result<(), AttachError> {
    let snapshot = slots.borrow().clone();
    for callback in snapshot.into_iter().flatten() {
        callback(data)?;
    }
    Ok(())
}
#[test]
fn arraybuffer_configuration_precedes_option_getters_and_survives_getter_failure() {
    let socket = Rc::new(Socket::default());
    let source = socket.clone();
    let result = AttachAddon::with_options(socket.clone(), move || {
        assert!(source.binary.get());
        Err(AttachError::Adapter("option getter failed".into()))
    });
    assert!(matches!(result,Err(AttachError::Adapter(message)) if message=="option getter failed"));
    assert!(socket.binary.get());
    assert!(socket.listeners.borrow().is_empty());
}
#[test]
fn socket_state_getter_and_closing_warning_failures_propagate_without_sending() {
    let socket = Rc::new(Socket::default());
    let terminal = Rc::new(Terminal::default());
    let addon = AttachAddon::new(socket.clone(), None).unwrap();
    addon.activate(terminal.clone()).unwrap();
    *socket.state_error.borrow_mut() = Some(AttachError::Adapter("state getter failed".into()));
    assert_eq!(
        fire(&terminal.data, &[97]),
        Err(AttachError::Adapter("state getter failed".into()))
    );
    *socket.state_error.borrow_mut() = None;
    socket.state.set(Some(SocketState::Closing));
    *socket.warning_error.borrow_mut() = Some(AttachError::Adapter("warning failed".into()));
    assert_eq!(
        fire(&terminal.binary, &[97]),
        Err(AttachError::Adapter("warning failed".into()))
    );
    assert_eq!(socket.warnings.get(), 1);
    assert!(socket.sent.borrow().is_empty());
}
#[test]
fn source_attach_string_and_utf8_payload_methods() {
    let socket = Rc::new(Socket::default());
    let terminal = Rc::new(Terminal::default());
    let addon = AttachAddon::new(socket.clone(), None).unwrap();
    assert!(socket.binary.get());
    addon.activate(terminal.clone()).unwrap();
    socket
        .fire(
            SocketEvent::Message,
            Some(SocketMessage::Text("foo".encode_utf16().collect())),
        )
        .unwrap();
    socket
        .fire(
            SocketEvent::Message,
            Some(SocketMessage::Binary(vec![102, 111, 111])),
        )
        .unwrap();
    assert_eq!(
        *terminal.writes.borrow(),
        [
            SocketMessage::Text(vec![102, 111, 111]),
            SocketMessage::Binary(vec![102, 111, 111])
        ]
    );
}
#[test]
fn exact_socket_states_and_binary_utf16_masking() {
    let socket = Rc::new(Socket::default());
    let terminal = Rc::new(Terminal::default());
    let addon = AttachAddon::new(socket.clone(), None).unwrap();
    addon.activate(terminal.clone()).unwrap();
    for (state, error) in [
        (SocketState::Connecting, AttachError::Connecting),
        (SocketState::Closed, AttachError::Closed),
        (SocketState::Unknown, AttachError::UnexpectedState),
    ] {
        socket.state.set(Some(state));
        assert_eq!(fire(&terminal.data, &[97]), Err(error));
    }
    socket.state.set(Some(SocketState::Closing));
    fire(&terminal.data, &[97]).unwrap();
    assert_eq!(socket.warnings.get(), 1);
    assert!(socket.sent.borrow().is_empty());
    socket.state.set(Some(SocketState::Open));
    fire(&terminal.data, &[0xd800, 0, 65535]).unwrap();
    fire(&terminal.binary, &[0xd800, 0, 65535]).unwrap();
    assert_eq!(
        *socket.sent.borrow(),
        [
            SocketMessage::Text(vec![0xd800, 0, 65535]),
            SocketMessage::Binary(vec![0, 0, 255])
        ]
    );
}
#[test]
fn unidirectional_close_error_dispose_and_drop_remove_all_listeners() {
    for event in [SocketEvent::Close, SocketEvent::Error] {
        let socket = Rc::new(Socket::default());
        let terminal = Rc::new(Terminal::default());
        let addon = AttachAddon::new(socket.clone(), Some(false)).unwrap();
        addon.activate(terminal.clone()).unwrap();
        assert!(terminal.data.borrow().is_empty() && terminal.binary.borrow().is_empty());
        socket.fire(event, None).unwrap();
        addon.dispose();
        assert!(socket.listeners.borrow().iter().all(Option::is_none));
    }
    let socket = Rc::new(Socket::default());
    let terminal = Rc::new(Terminal::default());
    let addon = AttachAddon::new(socket.clone(), None).unwrap();
    addon.activate(terminal.clone()).unwrap();
    drop(addon);
    assert!(socket.listeners.borrow().iter().all(Option::is_none));
    assert!(terminal.data.borrow().iter().all(Option::is_none));
    assert!(terminal.binary.borrow().iter().all(Option::is_none));
}
#[test]
fn attachment_does_not_form_socket_terminal_ownership_cycle() {
    let socket = Rc::new(Socket::default());
    let terminal = Rc::new(Terminal::default());
    let weak_terminal = Rc::downgrade(&terminal);
    let weak_socket = Rc::downgrade(&socket);
    let addon = AttachAddon::new(socket.clone(), None).unwrap();
    addon.activate(terminal).unwrap();
    assert!(weak_terminal.upgrade().is_none());
    drop(addon);
    drop(socket);
    assert!(weak_socket.upgrade().is_none());
}
