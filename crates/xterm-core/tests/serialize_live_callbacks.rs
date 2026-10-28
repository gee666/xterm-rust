use std::{cell::Cell, rc::Rc};
use xterm_core::{
    event::EmitError,
    headless::Terminal,
    options::{CoercionError, OptionError, OptionObject, OptionValue},
    serialize::{AddonSerializeError, HtmlOptions, SerializeAddon},
};

#[test]
fn html_reads_theme_getters_only_when_requested_and_releases_state_borrows() {
    let t = Rc::new(Terminal::new(10, 2, 20).unwrap());
    t.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    let a = Rc::new(SerializeAddon::new());
    t.load_addon(a.clone()).unwrap();
    let calls = Rc::new(Cell::new(0));
    let theme = OptionObject::with_fallible_properties(
        (Rc::downgrade(&t), calls.clone()),
        |_| Ok(OptionValue::from("[object Object]")),
        |(terminal, calls), key| {
            calls.set(calls.get() + 1);
            if key == "foreground" {
                terminal
                    .upgrade()
                    .unwrap()
                    .write("x")
                    .map_err(|e| OptionError::Dispatch(EmitError::callback(e)))?;
                Ok(OptionValue::from("#123456"))
            } else {
                Ok(OptionValue::Undefined)
            }
        },
    );
    t.options()
        .set("theme", OptionValue::Object(theme))
        .unwrap();
    a.serialize_as_html(&HtmlOptions::default()).unwrap();
    assert_eq!(calls.get(), 0);
    let output = String::from_utf16(
        &a.serialize_as_html(&HtmlOptions {
            include_global_background: true,
            ..HtmlOptions::default()
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(calls.get(), 2);
    assert!(output.contains("color: #123456;"));
    assert!(output.contains("<div><span>x         </span></div>"));
}

#[test]
fn theme_getter_errors_are_propagated_without_successful_empty_html() {
    let t = Terminal::new(10, 2, 20).unwrap();
    t.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    let a = Rc::new(SerializeAddon::new());
    t.load_addon(a.clone()).unwrap();
    let theme = OptionObject::with_properties(
        (),
        |_| Ok(OptionValue::from("[object Object]")),
        |_, _| Err(CoercionError::new("theme getter failed")),
    );
    t.options()
        .set("theme", OptionValue::Object(theme))
        .unwrap();
    let error = a
        .serialize_as_html(&HtmlOptions {
            include_global_background: true,
            ..HtmlOptions::default()
        })
        .unwrap_err();
    assert!(matches!(error, AddonSerializeError::Options(_)));
    assert_eq!(error.to_string(), "theme getter failed");
    assert!(a.serialize_as_html(&HtmlOptions::default()).is_ok());
}
