// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{cell::RefCell, rc::Rc};
use xterm_addons::attach::*;
use xterm_core::lifecycle::{to_disposable, Disposable, DisposableHandle};

type Entries = Rc<RefCell<Vec<Option<(SocketEvent, SocketCallback)>>>>;
type RemovalHook = Rc<dyn Fn(SocketEvent)>;
#[derive(Default)]
struct Socket {
    entries: Entries,
    removal: Rc<RefCell<Option<RemovalHook>>>,
    sent: RefCell<Vec<SocketMessage>>,
}
impl Socket {
    fn emit(&self, event: SocketEvent) {
        let snapshot = self.entries.borrow().clone();
        for (kind, callback) in snapshot.into_iter().flatten() {
            if kind == event {
                callback(None).unwrap();
            }
        }
    }
}
impl AttachSocket for Socket {
    fn use_array_buffer(&self) -> Result<(), AttachError> {
        Ok(())
    }
    fn state(&self) -> Result<SocketState, AttachError> {
        Ok(SocketState::Open)
    }
    fn send(&self, message: SocketMessage) -> Result<(), AttachError> {
        self.sent.borrow_mut().push(message);
        Ok(())
    }
    fn listen(
        &self,
        event: SocketEvent,
        callback: SocketCallback,
    ) -> Result<DisposableHandle, AttachError> {
        let index = self.entries.borrow().len();
        self.entries.borrow_mut().push(Some((event, callback)));
        let entries = self.entries.clone();
        let removal = self.removal.clone();
        Ok(to_disposable(move || {
            entries.borrow_mut()[index] = None;
            let hook = removal.borrow().clone();
            if let Some(hook) = hook {
                hook(event);
            }
        }))
    }
    fn warn_closing(&self) -> Result<(), AttachError> {
        panic!("open socket must not warn")
    }
}

type DataEntries = Rc<RefCell<Vec<Option<DataCallback>>>>;
#[derive(Default)]
struct Terminal {
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
    fn write(&self, _: SocketMessage) -> Result<(), AttachError> {
        panic!("disposed socket handler must not write")
    }
    fn on_data(&self, callback: DataCallback) -> DisposableHandle {
        subscribe(&self.data, callback)
    }
    fn on_binary(&self, callback: DataCallback) -> DisposableHandle {
        subscribe(&self.binary, callback)
    }
}

#[test]
fn disposal_visits_socket_and_terminal_handles_appended_by_reactivation() {
    for trigger in [None, Some(SocketEvent::Close), Some(SocketEvent::Error)] {
        for append_at in [SocketEvent::Message, SocketEvent::Close, SocketEvent::Error] {
            for count in [0, 1, 3] {
                for bidirectional in [false, true] {
                    let socket = Rc::new(Socket::default());
                    let terminal = Rc::new(Terminal::default());
                    let addon =
                        Rc::new(AttachAddon::new(socket.clone(), Some(bidirectional)).unwrap());
                    addon.activate(terminal.clone()).unwrap();
                    let remaining = Rc::new(std::cell::Cell::new(count));
                    let left = remaining.clone();
                    let weak = Rc::downgrade(&addon);
                    let host = terminal.clone();
                    *socket.removal.borrow_mut() = Some(Rc::new(move |event| {
                        if event == append_at && left.get() > 0 {
                            left.set(left.get() - 1);
                            weak.upgrade().unwrap().activate(host.clone()).unwrap();
                        }
                    }));
                    if let Some(trigger) = trigger {
                        socket.emit(trigger);
                    } else {
                        addon.dispose();
                    }
                    assert_eq!(remaining.get(), 0, "{trigger:?}/{append_at:?}/{count}");
                    assert!(
                        socket.entries.borrow().iter().all(Option::is_none),
                        "live socket handles after {trigger:?}/{append_at:?}/{count}"
                    );
                    assert!(terminal.data.borrow().iter().all(Option::is_none));
                    assert!(terminal.binary.borrow().iter().all(Option::is_none));
                    assert!(socket.sent.borrow().is_empty());
                    // The callback only retains a weak addon; cleanup cannot
                    // create an addon/socket cycle or append another handle.
                    socket.removal.borrow_mut().take();
                    addon.dispose();
                    let weak = Rc::downgrade(&addon);
                    drop(addon);
                    assert!(weak.upgrade().is_none());
                }
            }
        }
    }
}
