// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Escape parser error, reentrancy and lifecycle boundaries.
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_core::parser::{
    escape::{
        EscapeSequenceParser, FunctionIdentifier, ParseError, ParseStackState, RegistrarError,
    },
    handler::{DispatchError, HandlerResult},
    params::Params,
    table::ParserState,
};
type Log = Rc<RefCell<Vec<String>>>;
fn data(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}
struct Gate {
    ready: Rc<Cell<bool>>,
}
impl Future for Gate {
    type Output = bool;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<bool> {
        if self.ready.get() {
            Poll::Ready(true)
        } else {
            Poll::Pending
        }
    }
}
struct Noop;
impl Wake for Noop {
    fn wake(self: Arc<Self>) {}
}
#[test]
fn unresolved_future_stops_suffix_without_sleep_or_executor() {
    let mut parser = EscapeSequenceParser::new();
    let ready = Rc::new(Cell::new(false));
    let log: Log = Rc::default();
    let target = Rc::clone(&log);
    parser.set_print_handler(move |_| target.borrow_mut().push("suffix".to_owned()));
    let gate = Rc::clone(&ready);
    parser
        .register_csi_handler(
            FunctionIdentifier::new("m"),
            Rc::new(RefCell::new(move |_: &Params| {
                HandlerResult::Pending(Box::pin(Gate {
                    ready: Rc::clone(&gate),
                }))
            })),
        )
        .unwrap();
    let chunk = data("\x1b[mX");
    let Some(HandlerResult::Pending(mut pending)) = parser.parse(&chunk, None).unwrap() else {
        panic!("expected pending")
    };
    let waker = Waker::from(Arc::new(Noop));
    assert!(pending
        .as_mut()
        .poll(&mut Context::from_waker(&waker))
        .is_pending());
    assert!(log.borrow().is_empty());
    ready.set(true);
    let Poll::Ready(previous) = pending.as_mut().poll(&mut Context::from_waker(&waker)) else {
        panic!("gate ready")
    };
    assert!(parser.parse(&chunk, Some(previous)).unwrap().is_none());
    assert_eq!(*log.borrow(), ["suffix"]);
}

#[test]
fn checked_callbacks_abort_before_suffix_and_leave_join_state_unchanged() {
    let mut parser = EscapeSequenceParser::new();
    parser.preceding_join_state.set(123);
    parser.set_print_handler_checked(|_| Err(ParseError::HandlerAborted));
    assert!(matches!(
        parser.parse(&data("text\n"), None),
        Err(ParseError::HandlerAborted)
    ));
    assert_eq!(parser.preceding_join_state.get(), 123);
    parser.clear_print_handler();
    parser.set_execute_handler_checked(10, |_| Err(ParseError::HandlerAborted));
    assert!(matches!(
        parser.parse(&data("\ntext"), None),
        Err(ParseError::HandlerAborted)
    ));
    assert_eq!(parser.preceding_join_state.get(), 123);
    assert_eq!(parser.current_state, ParserState::Ground);
}

fn ready(result: HandlerResult) -> Result<bool, DispatchError> {
    let mut future = Box::pin(result.resolve());
    let waker = Waker::from(Arc::new(Noop));
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("test future should resolve immediately"),
    }
}

#[test]
fn failed_command_stops_bubbling_and_resumed_errors_retain_saved_frame() {
    for asynchronous in [false, true] {
        let mut parser = EscapeSequenceParser::new();
        let calls = Rc::new(Cell::new(0));
        let target = Rc::clone(&calls);
        parser.set_print_handler(move |_| target.set(target.get() + 1));
        parser
            .register_csi_handler(
                FunctionIdentifier::new("m"),
                Rc::new(RefCell::new(|_: &Params| {
                    HandlerResult::Failed(DispatchError::CallbackFailed)
                })),
            )
            .unwrap();
        if asynchronous {
            parser
                .register_csi_handler(
                    FunctionIdentifier::new("m"),
                    Rc::new(RefCell::new(|_: &Params| {
                        HandlerResult::Pending(Box::pin(async { false }))
                    })),
                )
                .unwrap();
        }
        let chunk = data("\x1b[mSUFFIX");
        if asynchronous {
            let pending = parser.parse(&chunk, None).unwrap().unwrap();
            let saved = parser.pause_info().unwrap();
            assert!(matches!(
                parser.parse(&chunk, Some(ready(pending).unwrap())),
                Err(ParseError::Dispatch(DispatchError::CallbackFailed))
            ));
            assert_eq!(parser.pause_info(), Some(saved));
            assert_eq!(parser.parse_stack_state(), ParseStackState::Paused);
        } else {
            assert!(matches!(
                parser.parse(&chunk, None),
                Err(ParseError::Dispatch(DispatchError::CallbackFailed))
            ));
            assert_eq!(parser.parse_stack_state(), ParseStackState::None);
        }
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn fallible_future_rejection_leaves_pause_until_explicit_reset() {
    for rejection in [false, true] {
        let mut parser = EscapeSequenceParser::new();
        let printed = Rc::new(Cell::new(false));
        let target = Rc::clone(&printed);
        parser.set_print_handler(move |_| target.set(true));
        parser
            .register_csi_handler(
                FunctionIdentifier::new("m"),
                Rc::new(RefCell::new(move |_: &Params| {
                    HandlerResult::PendingFallible(Box::pin(async move {
                        if rejection {
                            Err(DispatchError::CallbackFailed)
                        } else {
                            Ok(true)
                        }
                    }))
                })),
            )
            .unwrap();
        let chunk = data("\x1b[mSUFFIX");
        let pending = parser.parse(&chunk, None).unwrap().unwrap();
        let resolved = ready(pending);
        assert!(!printed.get());
        assert_eq!(parser.parse_stack_state(), ParseStackState::Paused);
        if rejection {
            assert_eq!(resolved, Err(DispatchError::CallbackFailed));
            parser.reset().unwrap();
            assert!(parser.parse(&chunk, None).unwrap().is_none());
        } else {
            assert!(parser
                .parse(&chunk, Some(resolved.unwrap()))
                .unwrap()
                .is_none());
        }
        assert!(printed.get());
        assert_eq!(parser.parse_stack_state(), ParseStackState::None);
    }
}

#[test]
fn registrar_registers_reentrantly_without_changing_current_reverse_loop() {
    let mut parser = EscapeSequenceParser::new();
    let registrar = parser.csi_registrar();
    let log: Log = Rc::default();
    let target = Rc::clone(&log);
    registrar
        .register(
            FunctionIdentifier::new("m"),
            Rc::new(RefCell::new(move |_: &Params| {
                target.borrow_mut().push("low".to_owned());
                HandlerResult::Ready(false)
            })),
        )
        .unwrap();
    let add = registrar.clone();
    let target = Rc::clone(&log);
    registrar
        .register(
            FunctionIdentifier::new("m"),
            Rc::new(RefCell::new(move |_: &Params| {
                target.borrow_mut().push("top".to_owned());
                let target = Rc::clone(&target);
                add.register(
                    FunctionIdentifier::new("m"),
                    Rc::new(RefCell::new(move |_: &Params| {
                        target.borrow_mut().push("new".to_owned());
                        HandlerResult::Ready(false)
                    })),
                )
                .unwrap();
                HandlerResult::Ready(false)
            })),
        )
        .unwrap();
    assert!(parser.parse(&data("\x1b[m"), None).unwrap().is_none());
    assert_eq!(*log.borrow(), ["top", "low"]);
    log.borrow_mut().clear();
    assert!(parser.parse(&data("\x1b[m"), None).unwrap().is_none());
    assert_eq!(*log.borrow(), ["new", "top", "low"]);
}

#[test]
fn weak_registrar_does_not_create_callback_cycles_and_reports_dropped_parser() {
    let parser = EscapeSequenceParser::new();
    let registrar = parser.esc_registrar();
    let captured_registrar = registrar.clone();
    let sentinel = Rc::new(());
    let observer = Rc::downgrade(&sentinel);
    registrar
        .register(
            FunctionIdentifier::new("E"),
            Rc::new(RefCell::new(move |_: &Params| {
                let _keep_alive = &sentinel;
                captured_registrar
                    .clear(FunctionIdentifier::new("F"))
                    .unwrap();
                HandlerResult::Ready(true)
            })),
        )
        .unwrap();
    drop(parser);
    assert!(observer.upgrade().is_none());
    assert_eq!(
        registrar.clear(FunctionIdentifier::new("E")),
        Err(RegistrarError::ParserDropped)
    );
    assert!(matches!(
        registrar.register(
            FunctionIdentifier::new("E"),
            Rc::new(RefCell::new(|_: &Params| HandlerResult::Ready(true)))
        ),
        Err(RegistrarError::ParserDropped)
    ));
}

#[test]
fn synchronous_boundaries_deliver_events_before_later_actions() {
    let mut parser = EscapeSequenceParser::new();
    let log: Log = Rc::default();
    let target = Rc::clone(&log);
    parser.set_print_handler(move |_| target.borrow_mut().push("print".to_owned()));
    let target = Rc::clone(&log);
    parser.set_execute_handler(10, move |_| {
        target.borrow_mut().push("execute".to_owned());
        true
    });
    let target = Rc::clone(&log);
    parser
        .register_csi_handler(
            FunctionIdentifier::new("m"),
            Rc::new(RefCell::new(move |_: &Params| {
                target.borrow_mut().push("csi".to_owned());
                HandlerResult::Ready(true)
            })),
        )
        .unwrap();
    let target = Rc::clone(&log);
    parser.set_action_boundary_handler(move || {
        target.borrow_mut().push("boundary".to_owned());
        Ok(())
    });
    assert!(parser.parse(&data("A\n\x1b[mB"), None).unwrap().is_none());
    assert_eq!(
        *log.borrow(),
        ["print", "boundary", "execute", "boundary", "csi", "boundary", "print", "boundary"]
    );
    parser.set_action_boundary_handler(|| Err(ParseError::HandlerAborted));
    log.borrow_mut().clear();
    assert!(matches!(
        parser.parse(&data("A\nB"), None),
        Err(ParseError::HandlerAborted)
    ));
    assert_eq!(*log.borrow(), ["print"]);
}

#[test]
fn callback_error_source_chain_retains_original_host_error() {
    use std::error::Error;
    let original: Rc<dyn Error> = Rc::new(std::io::Error::other("host sentinel"));
    let error = ParseError::Dispatch(DispatchError::Callback(
        xterm_core::event::EmitError::Callback(Rc::clone(&original)),
    ));
    let mut source: &dyn Error = &error;
    while let Some(next) = source.source() {
        source = next;
    }
    assert!(std::ptr::eq(source, original.as_ref()));
    assert_eq!(source.to_string(), "host sentinel");
}

#[test]
fn short_custom_table_missing_entries_match_js_undefined_bitwise_coercion() {
    let table = xterm_core::parser::table::TransitionTable::new(10);
    let mut parser = EscapeSequenceParser::with_transitions(table);
    parser.current_state = ParserState::Escape;
    parser.preceding_join_state.set(123);
    let printed = Rc::new(Cell::new(false));
    let flag = Rc::clone(&printed);
    parser.set_print_handler(move |_| flag.set(true));
    assert!(parser.parse(&data("A"), None).unwrap().is_none());
    assert_eq!(parser.current_state, ParserState::Ground);
    assert_eq!(parser.preceding_join_state.get(), 123);
    assert!(!printed.get());
}
