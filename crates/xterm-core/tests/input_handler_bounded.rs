// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    borrow::Cow,
    cell::RefCell,
    future::Future,
    ops::Range,
    pin::pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_core::{
    input::parse_source::{ParseChunk, ParseInputSource, ParseSourceError},
    log_service::LogValue,
    parser::handler::HandlerResult,
    public_api::parser::ParserApi,
    terminal::{CoreTerminal, DecodeBufferOverflow, TerminalError, MAX_PARSE_BUFFER_LENGTH},
};
fn row(terminal: &CoreTerminal) -> Vec<u16> {
    let active = terminal.state().active_buffer();
    let line = active.borrow().lines.get(0).unwrap();
    let text = line.borrow().translate_to_string(true, None, None, None);
    text
}
#[test]
fn original_seven_parse_storage_and_logical_length_pairs() {
    let mut terminal = CoreTerminal::new(10, 10, 1000).unwrap();
    let calls = Rc::new(RefCell::new(vec![]));
    let output = calls.clone();
    terminal.set_parse_buffer_observer(Some(Rc::new(move |storage, length| {
        output.borrow_mut().push((storage.len(), length));
        Ok(())
    })));
    terminal.write("12345").unwrap();
    terminal.write(&"a".repeat(10000)).unwrap();
    terminal.write(&"a".repeat(200000)).unwrap();
    terminal.write(&"a".repeat(300000)).unwrap();
    assert_eq!(
        *calls.borrow(),
        [
            (4096, 5),
            (10000, 10000),
            (131072, 131072),
            (131072, 68928),
            (131072, 131072),
            (131072, 131072),
            (131072, 37856)
        ]
    );
}
#[test]
fn retained_fixed_storage_keeps_unused_tail_between_short_writes() {
    let mut terminal = CoreTerminal::new(20, 3, 10).unwrap();
    terminal.write("abcdef").unwrap();
    let seen = Rc::new(RefCell::new(vec![]));
    let output = seen.clone();
    terminal.set_parse_buffer_observer(Some(Rc::new(move |storage, length| {
        assert_eq!(length, 1);
        output.borrow_mut().extend_from_slice(&storage[..6]);
        Ok(())
    })));
    terminal.write("x").unwrap();
    assert_eq!(*seen.borrow(), [120, 98, 99, 100, 101, 102]);
}
struct LiveBytes {
    bytes: RefCell<Vec<u8>>,
    ranges: RefCell<Vec<Range<usize>>>,
    whole: RefCell<usize>,
}
impl ParseInputSource for LiveBytes {
    fn len(&self) -> Result<usize, ParseSourceError> {
        Ok(self.bytes.borrow().len())
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        *self.whole.borrow_mut() += 1;
        Ok(ParseChunk::Bytes(Cow::Owned(self.bytes.borrow().clone())))
    }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        self.ranges.borrow_mut().push(range.clone());
        let data = self.bytes.borrow();
        let ParseChunk::Bytes(chunk) = ParseInputSource::read_range(&*data, range)? else {
            panic!("byte carrier changed kind")
        };
        Ok(ParseChunk::Bytes(Cow::Owned(chunk.into_owned())))
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        ParseInputSource::debug_units(&*self.bytes.borrow())
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        ParseInputSource::trace_value(&*self.bytes.borrow())
    }
}
fn resolve_now(result: HandlerResult) -> bool {
    struct NoWake;
    impl Wake for NoWake {
        fn wake(self: Arc<Self>) {}
    }
    let waker = Waker::from(Arc::new(NoWake));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(result.resolve());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => result.unwrap(),
        Poll::Pending => panic!("test future should resolve immediately"),
    }
}
#[test]
fn mutable_tail_reads_after_resume_but_decoded_prefix_does_not_replay() {
    let mut terminal = CoreTerminal::new(80, 30, 1000).unwrap();
    ParserApi::new(terminal.parser_mut())
        .register_osc_handler(1000.0, |data| {
            assert_eq!(data, "pause".encode_utf16().collect::<Vec<_>>());
            HandlerResult::Pending(Box::pin(async { true }))
        })
        .unwrap();
    let mut bytes = vec![0; MAX_PARSE_BUFFER_LENGTH + 1];
    let prefix = b"A\x1b]1000;pause\x07";
    bytes[..prefix.len()].copy_from_slice(prefix);
    bytes[MAX_PARSE_BUFFER_LENGTH] = b'B';
    let source = Rc::new(LiveBytes {
        bytes: RefCell::new(bytes),
        ranges: RefCell::new(vec![]),
        whole: RefCell::new(0),
    });
    assert!(matches!(
        terminal.write_source(source.clone()),
        Err(TerminalError::Pending)
    ));
    assert_eq!(row(&terminal), vec![65]);
    {
        let ranges = source.ranges.borrow();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], 0..MAX_PARSE_BUFFER_LENGTH);
    }
    source.bytes.borrow_mut()[100] = b'X';
    source.bytes.borrow_mut()[MAX_PARSE_BUFFER_LENGTH] = b'Z';
    let pending = terminal.take_pending().unwrap();
    terminal.resume(resolve_now(pending)).unwrap();
    assert_eq!(row(&terminal), vec![65, 90]);
    assert_eq!(*source.whole.borrow(), 0);
    assert_eq!(
        *source.ranges.borrow(),
        [
            0..MAX_PARSE_BUFFER_LENGTH,
            MAX_PARSE_BUFFER_LENGTH..MAX_PARSE_BUFFER_LENGTH + 1
        ]
    );
}
#[test]
fn overflow_guard_is_explicit_safety_deviation_and_keeps_source_prefix_cursor() {
    for length in [4096, MAX_PARSE_BUFFER_LENGTH] {
        let mut terminal = CoreTerminal::new(80, 24, 1000).unwrap();
        terminal.write_utf16(&[0xd800]).unwrap();
        let error = terminal.write(&"A".repeat(length)).unwrap_err();
        let TerminalError::Event(error) = error else {
            panic!("overflow lost typed cause")
        };
        assert_eq!(
            error.callback_ref::<DecodeBufferOverflow>(),
            Some(&DecodeBufferOverflow {
                decoded_length: length + 1,
                buffer_length: length
            })
        );
        let active = terminal.state().active_buffer();
        let active = active.borrow();
        assert_eq!((active.x, active.y), (length % 80, 23));
        // Source229 does not complete here. This assertion verifies the safe
        // Rust guard, not equivalence to the recorded nonterminating reference.
    }
}
