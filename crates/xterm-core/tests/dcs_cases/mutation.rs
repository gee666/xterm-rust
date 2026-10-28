// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Counterexample retained from independent review: JS throws, never skips.
use crate::dcs_support::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::dcs::{DcsHandlerTrait, DcsRegistrar, HandlerHandle};
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::params::Params;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Hook,
    Put,
    Unhook,
    Reset,
}

struct RemoveAll {
    stage: Stage,
    tokens: Weak<RefCell<Vec<DisposableHandle>>>,
}

impl RemoveAll {
    fn remove(&self, stage: Stage) {
        if self.stage == stage {
            let tokens = self
                .tokens
                .upgrade()
                .expect("test owns registration tokens");
            for token in tokens.borrow().iter() {
                token.dispose();
            }
        }
    }
}

impl DcsHandlerTrait for RemoveAll {
    fn hook(&mut self, _: &Params) -> Result<(), DispatchError> {
        self.remove(Stage::Hook);
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        self.remove(Stage::Put);
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.remove(if success { Stage::Unhook } else { Stage::Reset });
        HandlerResult::Ready(false)
    }
}

#[test]
fn removed_active_index_returns_error_for_every_callback_loop() {
    for stage in [Stage::Hook, Stage::Put, Stage::Unhook, Stage::Reset] {
        let (mut parser, reports) = parser();
        let tokens = Rc::new(RefCell::new(Vec::new()));
        for label in ["first", "second"] {
            tokens
                .borrow_mut()
                .push(parser.register_handler(IDENT, handler(&reports, label, false, false)));
        }
        let remove: HandlerHandle = Rc::new(RefCell::new(RemoveAll {
            stage,
            tokens: Rc::downgrade(&tokens),
        }));
        tokens
            .borrow_mut()
            .push(parser.register_handler(IDENT, remove));
        let hook = parser.hook(IDENT, &params(&[1, 2, 3]));
        let error = match stage {
            Stage::Hook => hook.err(),
            Stage::Put => {
                hook.unwrap();
                parser.put(&input("payload")).err()
            }
            Stage::Unhook => {
                hook.unwrap();
                parser.unhook(true, true).err()
            }
            Stage::Reset => {
                hook.unwrap();
                parser.reset().err()
            }
        };
        assert_eq!(error, Some(DispatchError::ActiveHandlerRemoved));
        parser.reset().unwrap();
    }
}

struct RegisterDuringHook {
    registrar: DcsRegistrar,
    target: HandlerHandle,
    reports: Reports,
}
impl DcsHandlerTrait for RegisterDuringHook {
    fn hook(&mut self, _: &Params) -> Result<(), DispatchError> {
        self.reports.borrow_mut().push("register:H".into());
        self.registrar
            .register_handler(IDENT, Rc::clone(&self.target))
            .unwrap();
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        self.reports.borrow_mut().push("register:P".into());
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.reports
            .borrow_mut()
            .push(format!("register:U:{success}"));
        HandlerResult::Ready(false)
    }
}

#[test]
fn callback_registrar_mutates_the_live_list_without_reborrowing_parser() {
    let (mut parser, reports) = parser();
    let registrar = parser.registrar();
    let target = handler(&reports, "new", true, false);
    parser.register_handler(
        IDENT,
        Rc::new(RefCell::new(RegisterDuringHook {
            registrar: registrar.clone(),
            target,
            reports: Rc::clone(&reports),
        })),
    );
    parser.hook(IDENT, &params(&[1])).unwrap();
    parser.put(&[65]).unwrap();
    unhook(&mut parser, true);
    assert_eq!(
        *reports.borrow(),
        [
            "register:H",
            "new:P:65",
            "register:P",
            "new:U:true",
            "register:U:false"
        ]
    );
    drop(parser);
    assert_eq!(registrar.clear_handler(IDENT), None);
    assert!(registrar
        .register_handler(IDENT, handler(&reports, "dropped", false, false))
        .is_none());
}

struct FailFirstReset(bool);
impl DcsHandlerTrait for FailFirstReset {
    fn hook(&mut self, _: &Params) -> Result<(), DispatchError> {
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        Ok(())
    }
    fn unhook(&mut self, _: bool) -> HandlerResult {
        if std::mem::replace(&mut self.0, false) {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        } else {
            HandlerResult::Ready(false)
        }
    }
}

#[test]
fn reset_failure_stops_older_cleanup_and_preserves_state_for_retry() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "older", true, false));
    parser.register_handler(IDENT, Rc::new(RefCell::new(FailFirstReset(true))));
    parser.hook(IDENT, &params(&[1])).unwrap();
    assert_eq!(parser.reset(), Err(DispatchError::CallbackFailed));
    assert_eq!(*reports.borrow(), ["older:H:1"]);
    parser.reset().unwrap();
    assert_eq!(*reports.borrow(), ["older:H:1", "older:U:false"]);
}

#[test]
fn idle_and_dispose_fallback_identifiers_follow_upstream() {
    let (mut parser, reports) = parser();
    parser.put(&[]).unwrap();
    unhook(&mut parser, false);
    assert_eq!(*reports.borrow(), ["0:P:", "0:U:false"]);
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.dispose();
    let output = Rc::clone(&reports);
    parser.set_handler_fallback(move |ident, _| output.borrow_mut().push(ident.to_string()));
    parser.put(&[]).unwrap();
    parser.reset().unwrap();
    parser.put(&[]).unwrap();
    assert_eq!(&reports.borrow()[3..], [IDENT.to_string(), "0".to_owned()]);
}

#[test]
fn registering_after_clear_does_not_replace_active_list() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "old", true, false));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.clear_handler(IDENT);
    parser.register_handler(IDENT, handler(&reports, "new", true, false));
    parser.put(&[]).unwrap();
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), ["old:H:1,2,3", "old:P:", "old:U:true"]);
    reports.borrow_mut().clear();
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["new"], true));
}
