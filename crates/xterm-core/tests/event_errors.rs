//! Callback failures retain concrete error identity through event compositions.
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use xterm_core::event::{EmitError, Emitter, Event};

#[derive(Debug)]
struct CallbackFailure(Rc<Cell<i32>>);
impl fmt::Display for CallbackFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original callback error")
    }
}
impl Error for CallbackFailure {}

#[test]
fn callback_error_preserves_concrete_identity_and_error_source() {
    let identity = Rc::new(Cell::new(7));
    let error = EmitError::callback(CallbackFailure(Rc::clone(&identity)));
    assert_eq!(error, error.clone());
    assert_ne!(
        error,
        EmitError::callback(CallbackFailure(Rc::clone(&identity)))
    );
    assert_ne!(error, EmitError::ListenerArrayCleared);
    assert_eq!(error.to_string(), "original callback error");
    let actual = error.callback_ref::<CallbackFailure>().unwrap();
    assert!(Rc::ptr_eq(&actual.0, &identity));
    actual.0.set(8);
    assert_eq!(identity.get(), 8);
    assert!(error.source().unwrap().is::<CallbackFailure>());
    assert!(EmitError::ListenerArrayCleared.source().is_none());
    assert!(EmitError::ListenerArrayCleared
        .callback_ref::<CallbackFailure>()
        .is_none());
}

#[test]
fn callback_failure_propagates_through_map_any_and_forward() {
    let source = Emitter::<i32>::new();
    let target = Rc::new(Emitter::new());
    let error = EmitError::callback(CallbackFailure(Rc::new(Cell::new(9))));
    let original = error.clone();
    target
        .event()
        .try_subscribe(move |_: &i32| Err(error.clone()));
    target
        .event()
        .subscribe(|_| panic!("callback reached after failure"));
    Event::any([source.event()])
        .map(|value| value * 2)
        .forward(target);
    let actual = source.fire(&1).unwrap_err();
    assert_eq!(actual, original);
    assert_eq!(actual.callback_ref::<CallbackFailure>().unwrap().0.get(), 9);
}
