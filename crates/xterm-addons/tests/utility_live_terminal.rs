// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_addons::{
    attach::{AttachTerminal, SocketMessage},
    clipboard::*,
    native_utility::NativeUtilityTerminal,
};
use xterm_core::{
    lifecycle::Disposable,
    parser::handler::HandlerResult,
    terminal::{CoreTerminal, TerminalError, TerminalEvent},
};

#[derive(Default)]
struct Provider {
    value: RefCell<Vec<u16>>,
    asynchronous: bool,
}
impl ClipboardProvider for Provider {
    fn read_text(&self, _: &[u16]) -> Completion<Vec<u16>> {
        let value = self.value.borrow().clone();
        if self.asynchronous {
            Completion::Pending(Box::pin(async { Ok(value) }))
        } else {
            Completion::Ready(Ok(value))
        }
    }
    fn write_text(&self, _: &[u16], text: &[u16]) -> Completion<()> {
        *self.value.borrow_mut() = text.to_vec();
        Completion::Ready(Ok(()))
    }
}
fn text(engine: &CoreTerminal) -> String {
    let state = engine.state();
    let buffer = state.active_buffer();
    let buffer = buffer.borrow();
    let line = buffer.lines.get(buffer.ybase).unwrap();
    let text = line.borrow().translate_to_string(true, None, None, None);
    String::from_utf16(&text).unwrap()
}
fn replies(engine: &mut CoreTerminal) -> Vec<Vec<u16>> {
    engine
        .drain_events()
        .into_iter()
        .filter_map(|event| {
            if let TerminalEvent::Data(data) = event {
                Some(data)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn attach_real_parser_keeps_utf8_split_across_messages_and_ansi() {
    let engine = Rc::new(RefCell::new(CoreTerminal::new(80, 24, 1000).unwrap()));
    let adapter = NativeUtilityTerminal::new(engine.clone());
    adapter
        .write(SocketMessage::Text("\x1b[31mfoo".encode_utf16().collect()))
        .unwrap();
    adapter
        .write(SocketMessage::Binary(vec![0xe2, 0x82]))
        .unwrap();
    adapter.write(SocketMessage::Binary(vec![0xac])).unwrap();
    assert_eq!(text(&engine.borrow()), "foo€");
}
#[test]
fn clipboard_real_osc_parser_chunk_boundaries_replies_and_disposal() {
    let input = "\x1b]52;c;4oKsbWzDpMO8dMOf\x07\x1b]52;p;?\x1b\\";
    for split in 0..=input.len() {
        let engine = Rc::new(RefCell::new(CoreTerminal::new(80, 24, 1000).unwrap()));
        let adapter = Rc::new(NativeUtilityTerminal::new(engine.clone()));
        let provider = Rc::new(Provider::default());
        let addon = ClipboardAddon::new(provider.clone(), None);
        addon.activate(adapter.clone());
        engine
            .borrow_mut()
            .write_bytes(&input.as_bytes()[..split])
            .unwrap();
        engine
            .borrow_mut()
            .write_bytes(&input.as_bytes()[split..])
            .unwrap();
        assert_eq!(
            String::from_utf16(&provider.value.borrow()).unwrap(),
            "€mläütß",
            "split{split}"
        );
        assert_eq!(
            replies(&mut engine.borrow_mut()),
            ["\x1b]52;p;4oKsbWzDpMO8dMOf\x07"
                .encode_utf16()
                .collect::<Vec<_>>()]
        );
        assert_eq!(text(&engine.borrow()), "");
        addon.dispose();
        engine.borrow_mut().write("\x1b]52;c;?\x07").unwrap();
        assert!(replies(&mut engine.borrow_mut()).is_empty());
    }
}
struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}
fn settle(engine: &Rc<RefCell<CoreTerminal>>) {
    let pending = engine.borrow_mut().take_pending().unwrap();
    let HandlerResult::PendingFallible(mut future) = pending else {
        panic!("expected fallible clipboard future");
    };
    let waker = Waker::from(Arc::new(NoWake));
    let Poll::Ready(result) = future.as_mut().poll(&mut Context::from_waker(&waker)) else {
        panic!("provider fixture resolves first poll");
    };
    engine.borrow_mut().resume(result.unwrap()).unwrap();
}
#[test]
fn clipboard_async_parser_pause_reply_precedes_next_da1() {
    let engine = Rc::new(RefCell::new(CoreTerminal::new(80, 24, 1000).unwrap()));
    let adapter = Rc::new(NativeUtilityTerminal::new(engine.clone()));
    let provider = Rc::new(Provider {
        value: RefCell::new("hello".encode_utf16().collect()),
        asynchronous: true,
    });
    let addon = ClipboardAddon::new(provider, None);
    addon.activate(adapter.clone());
    assert!(matches!(
        engine.borrow_mut().write("\x1b]52;c;?\x07\x1b[c"),
        Err(TerminalError::Pending)
    ));
    assert!(replies(&mut engine.borrow_mut()).is_empty());
    settle(&engine);
    assert_eq!(
        replies(&mut engine.borrow_mut()),
        [
            "\x1b]52;c;aGVsbG8=\x07".encode_utf16().collect::<Vec<_>>(),
            "\x1b[?1;2c".encode_utf16().collect()
        ]
    );
}
#[test]
fn pending_clipboard_reply_uses_latest_activation_without_replaying_input() {
    let first = Rc::new(RefCell::new(CoreTerminal::new(80, 24, 1000).unwrap()));
    let second = Rc::new(RefCell::new(CoreTerminal::new(80, 24, 1000).unwrap()));
    let original = Rc::new(NativeUtilityTerminal::new(first.clone()));
    let replacement = Rc::new(NativeUtilityTerminal::new(second.clone()));
    let provider = Rc::new(Provider {
        value: RefCell::new("hello".encode_utf16().collect()),
        asynchronous: true,
    });
    let addon = ClipboardAddon::new(provider, None);
    addon.activate(original.clone());
    assert!(matches!(
        first.borrow_mut().write("\x1b]52;c;?\x07"),
        Err(TerminalError::Pending)
    ));
    addon.activate(replacement.clone());
    settle(&first);
    assert!(replies(&mut first.borrow_mut()).is_empty());
    assert_eq!(
        replies(&mut second.borrow_mut()),
        ["\x1b]52;c;aGVsbG8=\x07".encode_utf16().collect::<Vec<_>>()]
    );
}
