// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Callback-safe disposal follows source property traversal.
use crate::dcs_support::{handler, params, Reports, IDENT};
use std::{cell::RefCell, rc::Rc};
use xterm_core::parser::{
    dcs::{DcsDisposalHandle, DcsHandlerTrait, DcsParser, DcsRegistrar},
    handler::{DispatchError, HandlerResult},
    params::Params,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Hook,
    Put,
    Unhook,
    Reset,
}
struct DisposeOn {
    phase: Phase,
    disposal: DcsDisposalHandle,
    registrar: DcsRegistrar,
    reports: Reports,
    register_after: bool,
}
impl DisposeOn {
    fn call(&self, phase: Phase) {
        self.reports.borrow_mut().push(
            match phase {
                Phase::Hook => "H",
                Phase::Put => "P",
                Phase::Unhook => "U:true",
                Phase::Reset => "U:false",
            }
            .into(),
        );
        if self.phase == phase {
            assert_eq!(self.disposal.dispose(), Some(()));
            if self.register_after {
                self.registrar
                    .register_handler(IDENT, handler(&self.reports, "new", true, false))
                    .unwrap();
            }
        }
    }
}
impl DcsHandlerTrait for DisposeOn {
    fn hook(&mut self, _: &Params) -> Result<(), DispatchError> {
        self.call(Phase::Hook);
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        self.call(Phase::Put);
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.call(if success { Phase::Unhook } else { Phase::Reset });
        HandlerResult::Ready(false)
    }
}

#[test]
fn disposing_an_active_parser_replaces_the_property_not_a_captured_array() {
    for phase in [Phase::Hook, Phase::Put, Phase::Unhook, Phase::Reset] {
        let mut parser = DcsParser::new();
        let reports: Reports = Rc::default();
        parser.register_handler(IDENT, handler(&reports, "older", false, false));
        parser.register_handler(
            IDENT,
            Rc::new(RefCell::new(DisposeOn {
                phase,
                disposal: parser.disposal_handle(),
                registrar: parser.registrar(),
                reports: Rc::clone(&reports),
                register_after: false,
            })),
        );
        let hook = parser.hook(IDENT, &params(&[1]));
        let error = match phase {
            Phase::Hook => hook.err(),
            Phase::Put => {
                hook.unwrap();
                parser.put(&[65]).err()
            }
            Phase::Unhook => {
                hook.unwrap();
                parser.unhook(true, true).err()
            }
            Phase::Reset => {
                hook.unwrap();
                parser.reset().err()
            }
        };
        assert_eq!(error, Some(DispatchError::ActiveHandlerRemoved));
        parser.reset().unwrap();
    }
}

#[test]
fn single_callback_disposal_then_new_registration_survives_without_a_second_dispose() {
    let mut parser = DcsParser::new();
    let reports: Reports = Rc::default();
    let disposal = parser.disposal_handle();
    parser.register_handler(
        IDENT,
        Rc::new(RefCell::new(DisposeOn {
            phase: Phase::Hook,
            disposal: disposal.clone(),
            registrar: parser.registrar(),
            reports: Rc::clone(&reports),
            register_after: true,
        })),
    );
    parser.hook(IDENT, &params(&[1])).unwrap();
    parser.put(&[65]).unwrap();
    assert_eq!(*reports.borrow(), ["H"]);
    parser.hook(IDENT, &params(&[2])).unwrap();
    parser.put(&[66]).unwrap();
    assert!(parser.unhook(true, true).unwrap().is_none());
    assert_eq!(
        *reports.borrow(),
        ["H", "new:H:2", "new:P:66", "new:U:true"]
    );
    // Direct child disposal is repeatable, not idempotent. The parent terminal
    // must not perform a delayed second disposal after post-dispose registration.
    disposal.dispose().unwrap();
    reports.borrow_mut().clear();
    parser.hook(IDENT, &params(&[3])).unwrap();
    parser.put(&[67]).unwrap();
    assert!(reports.borrow().is_empty());
}

#[test]
fn fallback_can_dispose_itself_without_borrowing_its_container_in_callout() {
    let mut parser = DcsParser::new();
    let reports: Reports = Rc::default();
    let disposal = parser.disposal_handle();
    let handle = disposal.clone();
    let registrar = parser.registrar();
    let output = Rc::clone(&reports);
    parser.set_handler_fallback_checked(move |ident, _| {
        output.borrow_mut().push(format!("fallback:{ident}"));
        handle.dispose().unwrap();
        registrar
            .register_handler(IDENT, handler(&output, "new", true, false))
            .unwrap();
        Ok(())
    });
    parser.hook(IDENT, &params(&[1])).unwrap();
    parser.put(&[65]).unwrap();
    assert_eq!(*reports.borrow(), [format!("fallback:{IDENT}")]);
    parser.hook(IDENT, &params(&[2])).unwrap();
    parser.put(&[66]).unwrap();
    assert_eq!(
        reports.borrow().last().map(String::as_str),
        Some("new:P:66")
    );
    drop(parser);
    assert_eq!(disposal.dispose(), None);
}
