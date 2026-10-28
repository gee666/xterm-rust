// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::parser::{
    handler::{DispatchError, HandlerResult},
    osc::{OscAction, OscDisposalHandle, OscHandler, OscHandlerTrait, OscParser},
};
type Log = Rc<RefCell<Vec<String>>>;
fn action(value: &OscAction) -> String {
    match value {
        OscAction::Start => "s".into(),
        OscAction::Put(data) => format!("p:{}", hex(data)),
        OscAction::End(ok) => format!("e:{}", u8::from(*ok)),
    }
}
fn hex(value: &[u16]) -> String {
    value
        .iter()
        .map(|unit| format!("{unit:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn fallback_can_dispose_without_borrow_conflict_or_callback_resurrection() {
    for phase in ['s', 'p', 'e'] {
        let log = Log::default();
        let mut parser = OscParser::new();
        let disposal = parser.disposal_handle();
        let registry = parser.registrar();
        let output = Rc::clone(&log);
        parser.set_handler_fallback_checked(move |_, value| {
            let message = action(&value);
            let matches = message.starts_with(phase);
            output.borrow_mut().push(message);
            if matches {
                disposal.dispose().unwrap();
                let output = Rc::clone(&output);
                registry
                    .register_handler(
                        2.0,
                        Rc::new(RefCell::new(OscHandler::new(move |data| {
                            output.borrow_mut().push(format!("new:{}", hex(&data)));
                            HandlerResult::Ready(true)
                        }))),
                    )
                    .unwrap();
            }
            Ok(())
        });
        parser.start().unwrap();
        parser.put(&[50, 59, 65]).unwrap();
        assert!(parser.end(true, true).unwrap().is_none());
        parser.start().unwrap();
        parser.put(&[50, 59, 66]).unwrap();
        assert!(parser.end(true, true).unwrap().is_none());
        let expected = match phase {
            's' => vec!["s", "new:42"],
            'p' => vec!["s", "p:41", "new:42"],
            _ => vec!["s", "p:41", "e:1", "new:42"],
        };
        assert_eq!(*log.borrow(), expected, "phase{phase}");
    }
}
struct Handler {
    name: u8,
    phase: char,
    disposal: OscDisposalHandle,
    log: Log,
}
impl OscHandlerTrait for Handler {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!("s{}", self.name));
        if self.name == 2 && self.phase == 's' {
            self.disposal.dispose().unwrap();
        }
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!("p{}", self.name));
        if self.name == 2 && self.phase == 'p' {
            self.disposal.dispose().unwrap();
        }
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.log
            .borrow_mut()
            .push(format!("e{}:{}", self.name, u8::from(success)));
        if self.name == 2 && self.phase == 'e' {
            self.disposal.dispose().unwrap();
        }
        HandlerResult::Ready(false)
    }
}
#[test]
fn active_disposal_is_observed_by_the_next_index_in_every_raw_phase() {
    for phase in ['s', 'p', 'e'] {
        let log = Log::default();
        let mut parser = OscParser::new();
        let disposal = parser.disposal_handle();
        for name in 0..3 {
            parser.register_handler(
                2.0,
                Rc::new(RefCell::new(Handler {
                    name,
                    phase,
                    disposal: disposal.clone(),
                    log: Rc::clone(&log),
                })),
            );
        }
        parser.start().unwrap();
        let result = if phase == 'e' {
            parser.put(&[50, 59, 65]).unwrap();
            parser.end(true, true).map(|_| ())
        } else {
            parser.put(&[50, 59, 65])
        };
        assert_eq!(result, Err(DispatchError::ActiveHandlerRemoved));
        let size = log.borrow().len();
        parser.reset().unwrap();
        assert_eq!(log.borrow().len(), size);
        let expected = match phase {
            's' => vec!["s2"],
            'p' => vec!["s2", "s1", "s0", "p2"],
            _ => vec!["s2", "s1", "s0", "p2", "p1", "p0", "e2:1"],
        };
        assert_eq!(*log.borrow(), expected);
    }
}
#[test]
fn disposal_retains_id_collection_and_new_registration_until_the_next_dispatch() {
    let log = Log::default();
    let mut parser = OscParser::new();
    let disposal = parser.disposal_handle();
    let registrar = parser.registrar();
    parser.start().unwrap();
    parser.put(&[50]).unwrap();
    disposal.dispose().unwrap();
    let output = Rc::clone(&log);
    registrar
        .register_handler(
            2.0,
            Rc::new(RefCell::new(OscHandler::new(move |data| {
                output.borrow_mut().push(hex(&data));
                HandlerResult::Ready(true)
            }))),
        )
        .unwrap();
    parser.put(&[59, 66]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["42"]);
    disposal.dispose().unwrap();
    parser.start().unwrap();
    parser.put(&[50, 59, 67]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["42"]);
    drop(parser);
    assert!(disposal.dispose().is_none());
}
#[test]
fn child_disposal_never_invents_active_handler_abort_callbacks() {
    let log = Log::default();
    let mut parser = OscParser::new();
    let disposal = parser.disposal_handle();
    parser.register_handler(
        2.0,
        Rc::new(RefCell::new(Handler {
            name: 0,
            phase: 'x',
            disposal: disposal.clone(),
            log: Rc::clone(&log),
        })),
    );
    parser.start().unwrap();
    parser.put(&[50, 59, 65]).unwrap();
    disposal.dispose().unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["s0", "p0"]);
}
