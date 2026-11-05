use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    headless::Terminal,
    parser::{
        apc::ApcHandlerTrait,
        escape::FunctionIdentifier,
        handler::{DispatchError, HandlerResult},
    },
};
struct Capture(Rc<RefCell<Vec<String>>>);
impl ApcHandlerTrait for Capture {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.0.borrow_mut().push("start".into());
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.0.borrow_mut().push(format!("put:{data:?}"));
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.0.borrow_mut().push(format!("end:{success}"));
        HandlerResult::Ready(true)
    }
}
#[test]
fn streaming_apc_delivers_chunks_and_disposal_stops_active_and_future_delivery() {
    let terminal = Terminal::new(20, 10, 20).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let registration = terminal
        .with_parser(|parser| {
            parser.register_streaming_apc_handler(
                FunctionIdentifier::new("G"),
                Rc::new(RefCell::new(Capture(events.clone()))),
            )
        })
        .unwrap()
        .unwrap();
    terminal.write("\x1b_Gabc").unwrap();
    assert_eq!(*events.borrow(), ["start", "put:[97, 98, 99]"]);
    registration.dispose();
    terminal.write("def\x1b\\Z\x1b_Gother\x1b\\").unwrap();
    assert_eq!(events.borrow().len(), 2);
}
#[test]
fn streaming_apc_cancellation_prefix_removal_and_identifier_validation_match_contract() {
    let terminal = Terminal::new(20, 10, 20).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut id = FunctionIdentifier::new("G");
    id.prefix = Some("?");
    let registration = terminal
        .with_parser(|parser| {
            parser
                .register_streaming_apc_handler(id, Rc::new(RefCell::new(Capture(events.clone()))))
        })
        .unwrap()
        .unwrap();
    terminal.write("\x1b_Gabc\x18Z").unwrap();
    assert_eq!(events.borrow().last().unwrap(), "end:false");
    let result = terminal
        .with_parser(|parser| {
            parser.register_streaming_apc_handler(
                FunctionIdentifier::new("GG"),
                Rc::new(RefCell::new(Capture(events.clone()))),
            )
        })
        .unwrap();
    assert!(result.is_err());
    registration.dispose();
}
