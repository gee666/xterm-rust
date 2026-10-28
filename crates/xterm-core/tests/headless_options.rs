use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    headless::PublicOptions,
    options::{OptionValue, OptionsService},
};

#[test]
fn pa02_view_is_live_and_copies_share_the_actual_service() {
    let service = Rc::new(OptionsService::default());
    let view = PublicOptions::new(Rc::clone(&service));
    let second = view.clone();
    assert_eq!(view.get("cols").unwrap(), OptionValue::Number(80.0));
    assert_eq!(view.get("rows").unwrap(), OptionValue::Number(24.0));
    service.set_raw("cols", OptionValue::Number(132.0)).unwrap();
    assert_eq!(view.get("cols").unwrap(), OptionValue::Number(132.0));
    second.set("scrollback", OptionValue::Number(1.0)).unwrap();
    assert_eq!(service.get("scrollback").unwrap(), OptionValue::Number(1.0));
    assert_eq!(view.get("scrollback").unwrap(), OptionValue::Number(1.0));
    assert_eq!(view.keys(), service.keys());
}

#[test]
fn pa02_bulk_assignment_preserves_partial_updates_and_event_order() {
    let service = Rc::new(OptionsService::default());
    let view = PublicOptions::new(Rc::clone(&service));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    service
        .on_option_change()
        .subscribe(move |key| output.borrow_mut().push(*key));
    let error = view
        .assign([
            ("fontSize", OptionValue::Number(12.0)),
            ("fontFamily", OptionValue::from("Arial")),
            ("cols", OptionValue::Number(80.0)),
            ("scrollback", OptionValue::Number(1.0)),
        ])
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Option \"cols\" can only be set in the constructor"
    );
    assert_eq!(view.get("fontSize").unwrap(), OptionValue::Number(12.0));
    assert_eq!(view.get("fontFamily").unwrap(), OptionValue::from("Arial"));
    assert_eq!(view.get("scrollback").unwrap(), OptionValue::Number(1000.0));
    assert_eq!(*calls.borrow(), ["fontSize", "fontFamily"]);
    for key in ["cols", "rows"] {
        assert!(view.set(key, view.get(key).unwrap()).is_err());
    }
}

#[test]
fn pa02_proposed_access_reads_current_option_and_unknown_native_keys_reject() {
    let service = Rc::new(OptionsService::default());
    let view = PublicOptions::new(Rc::clone(&service));
    assert_eq!(
        view.check_proposed_api().unwrap_err().to_string(),
        "You must set the allowProposedApi option to true to use proposed API"
    );
    view.set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    assert!(view.check_proposed_api().is_ok());
    service
        .set("allowProposedApi", OptionValue::Bool(false))
        .unwrap();
    assert!(view.check_proposed_api().is_err());
    assert!(view.set("unknown", OptionValue::Null).is_err());
    assert!(view.get("unknown").is_err());
}
