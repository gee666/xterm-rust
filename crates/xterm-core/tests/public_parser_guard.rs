use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    parser::{
        escape::{EscapeSequenceParser, FunctionIdentifier, ParseError},
        handler::{DispatchError, HandlerResult},
    },
    public_api::ParserApi,
};

#[test]
fn bare_window_csi_guard_is_live_and_consumes_blocked_custom_sequences() {
    let mut parser = EscapeSequenceParser::new();
    let allowed = Rc::new(Cell::new(false));
    let captured = allowed.clone();
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    ParserApi::with_window_option_guard(&mut parser, move |_| Ok(captured.get()))
        .register_csi_handler(FunctionIdentifier::new("t"), move |_| {
            count.set(count.get() + 1);
            HandlerResult::Ready(false)
        })
        .unwrap();
    assert!(parser
        .parse(&[0x1b, 0x5b, 0x31, 0x34, 0x74], None)
        .unwrap()
        .is_none());
    assert_eq!(calls.get(), 0);
    allowed.set(true);
    assert!(parser
        .parse(&[0x1b, 0x5b, 0x31, 0x34, 0x74], None)
        .unwrap()
        .is_none());
    assert_eq!(calls.get(), 1);
    let count = calls.clone();
    let identifier = FunctionIdentifier {
        prefix: Some("?"),
        intermediates: None,
        final_byte: "t",
    };
    ParserApi::with_window_option_guard(&mut parser, |_| panic!("prefixed CSI must bypass guard"))
        .register_csi_handler(identifier, move |_| {
            count.set(count.get() + 1);
            HandlerResult::Ready(true)
        })
        .unwrap();
    assert!(parser
        .parse(&[0x1b, 0x5b, 0x3f, 0x31, 0x74], None)
        .unwrap()
        .is_none());
    assert_eq!(calls.get(), 2);
}

#[test]
fn window_guard_error_preserves_cause_and_stops_following_print() {
    let mut parser = EscapeSequenceParser::new();
    let printed = Rc::new(RefCell::new(Vec::new()));
    let captured = printed.clone();
    parser.set_print_handler(move |data| captured.borrow_mut().extend_from_slice(data));
    ParserApi::with_window_option_guard(&mut parser, |_| {
        Err(EmitError::callback(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "window option getter",
        )))
    })
    .register_csi_handler(FunctionIdentifier::new("t"), |_| {
        panic!("callback must not run after guard error")
    })
    .unwrap();
    let error = parser
        .parse(&[0x1b, 0x5b, 0x31, 0x74, 0x61], None)
        .err()
        .unwrap();
    assert!(matches!(
        &error,
        ParseError::Dispatch(DispatchError::Callback(_))
    ));
    assert!(error.to_string().contains("window option getter"));
    assert!(printed.borrow().is_empty());
}
