use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    event::EmitError,
    headless::Terminal,
    options::{CoercionHint, NumericSemantics, OptionError, OptionObject, OptionValue},
    serialize::{HtmlOptions, SerializeAddon},
};

struct Value {
    label: &'static str,
    text: &'static str,
    trace: Rc<RefCell<Vec<String>>>,
}
impl NumericSemantics for Value {
    fn concat_utf16(&self) -> Option<Result<Vec<u16>, OptionError>> {
        self.trace
            .borrow_mut()
            .push(format!("{}:default", self.label));
        Some(Ok(self.text.encode_utf16().collect()))
    }
    fn less_than(&self, rhs: f64) -> Result<bool, OptionError> {
        Ok(15.0 < rhs)
    }
    fn less_equal(&self, rhs: f64) -> Result<bool, OptionError> {
        Ok(15.0 <= rhs)
    }
    fn multiply(&self, rhs: f64) -> Result<f64, OptionError> {
        Ok(15.0 * rhs)
    }
}
fn value(label: &'static str, text: &'static str, trace: &Rc<RefCell<Vec<String>>>) -> OptionValue {
    let log = trace.clone();
    OptionValue::Object(OptionObject::with_numeric_value_semantics(
        (),
        move |hint| {
            log.borrow_mut().push(format!("{label}:{hint:?}"));
            Ok(match hint {
                CoercionHint::String => OptionValue::from("wrong-string"),
                CoercionHint::Number => OptionValue::Number(15.0),
            })
        },
        |_, _| Ok(OptionValue::Undefined),
        true,
        |_, _| false,
        Rc::new(Value {
            label,
            text,
            trace: trace.clone(),
        }),
    ))
}
#[test]
fn getters_are_read_before_default_hint_coercions_and_theme_is_reloaded() {
    let t = Rc::new(Terminal::new(10, 2, 20).unwrap());
    t.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    let a = Rc::new(SerializeAddon::new());
    t.load_addon(a.clone()).unwrap();
    let trace = Rc::new(RefCell::new(Vec::<String>::new()));
    let fg = value("fg", "#112233", &trace);
    let bg = value("bg", "#445566", &trace);
    let second = OptionValue::Object(OptionObject::with_properties(
        (bg, trace.clone()),
        |_| Ok(OptionValue::from("[object Object]")),
        |(bg, trace), key| {
            trace.borrow_mut().push(format!("new:{key}"));
            Ok(if key == "background" {
                bg.clone()
            } else {
                OptionValue::Undefined
            })
        },
    ));
    let first = OptionObject::with_fallible_properties(
        (Rc::downgrade(&t), second, fg, trace.clone()),
        |_| Ok(OptionValue::from("[object Object]")),
        |(weak, new_theme, fg, trace), key| {
            trace.borrow_mut().push(format!("old:{key}"));
            if key == "foreground" {
                weak.upgrade()
                    .unwrap()
                    .options()
                    .set("theme", new_theme.clone())
                    .map_err(|e| OptionError::Dispatch(EmitError::callback(e)))?;
                Ok(fg.clone())
            } else {
                Ok(OptionValue::Undefined)
            }
        },
    );
    t.options()
        .set("theme", OptionValue::Object(first))
        .unwrap();
    t.options()
        .set("fontFamily", value("font", "fancy", &trace))
        .unwrap();
    t.options()
        .set("fontSize", value("size", "weird", &trace))
        .unwrap();
    trace.borrow_mut().clear();
    let actual = a
        .serialize_as_html(&HtmlOptions {
            include_global_background: true,
            ..HtmlOptions::default()
        })
        .unwrap();
    let expected:Vec<u16>="<html><body><!--StartFragment--><pre><div style='color: #112233; background-color: #445566; font-family: fancy; font-size: weirdpx;'><div><span>          </span></div><div><span>          </span></div></div></pre><!--EndFragment--></body></html>".encode_utf16().collect();
    assert_eq!(
        actual, expected,
        "source raw fontsize and exact default-hint string concatenation"
    );
    assert_eq!(
        *trace.borrow(),
        [
            "old:foreground",
            "new:background",
            "fg:default",
            "bg:default",
            "font:default",
            "size:default"
        ]
    );
}
