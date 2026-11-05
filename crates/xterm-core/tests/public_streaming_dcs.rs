use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    headless::Terminal,
    parser::{
        dcs::DcsHandlerTrait,
        escape::FunctionIdentifier,
        handler::{DispatchError, HandlerResult},
        params::Params,
    },
};

struct Capture(Rc<RefCell<Vec<String>>>);
impl DcsHandlerTrait for Capture {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError> {
        self.0
            .borrow_mut()
            .push(format!("hook:{:?}", params.to_array()));
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.0.borrow_mut().push(format!("put:{data:?}"));
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.0.borrow_mut().push(format!("unhook:{success}"));
        HandlerResult::Ready(true)
    }
}

#[test]
fn streaming_registration_delivers_chunks_and_disposal_stops_delivery() {
    let terminal = Terminal::new(20, 10, 20).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let registration = terminal
        .with_parser(|parser| {
            parser.register_streaming_dcs_handler(
                FunctionIdentifier::new("q"),
                Rc::new(RefCell::new(Capture(events.clone()))),
            )
        })
        .unwrap()
        .unwrap();
    terminal.write("\x1bP0;1qabc").unwrap();
    assert_eq!(events.borrow().len(), 2);
    registration.dispose();
    terminal.write("def\x1b\\Z").unwrap();
    assert_eq!(
        *events.borrow(),
        ["hook:[Param(0), Param(1)]", "put:[97, 98, 99]"]
    );
    terminal.write("\x1bPqother\x1b\\").unwrap();
    assert_eq!(events.borrow().len(), 2);
}

#[test]
fn cancellation_and_identifier_validation_use_the_real_parser_contract() {
    let terminal = Terminal::new(20, 10, 20).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let registration = terminal
        .with_parser(|parser| {
            parser.register_streaming_dcs_handler(
                FunctionIdentifier::new("q"),
                Rc::new(RefCell::new(Capture(events.clone()))),
            )
        })
        .unwrap()
        .unwrap();
    terminal.write("\x1bPqabc\x18Z").unwrap();
    assert_eq!(events.borrow().last().unwrap(), "unhook:false");
    let result = terminal
        .with_parser(|parser| {
            parser.register_streaming_dcs_handler(
                FunctionIdentifier::new("qq"),
                Rc::new(RefCell::new(Capture(events.clone()))),
            )
        })
        .unwrap();
    assert!(result.is_err());
    registration.dispose();
}
