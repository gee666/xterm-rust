use std::rc::Rc;
use xterm_core::{
    event::EmitError,
    headless::Terminal,
    options::{OptionError, OptionObject, OptionValue},
    serialize::{HtmlOptions, SerializeAddon},
};

#[test]
fn html_retains_original_buffer_and_range_across_header_reentrancy() {
    let t = Rc::new(Terminal::new(10, 2, 20).unwrap());
    t.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    t.write("normal").unwrap();
    let a = Rc::new(SerializeAddon::new());
    t.load_addon(a.clone()).unwrap();
    let theme = OptionObject::with_fallible_properties(
        Rc::downgrade(&t),
        |_| Ok(OptionValue::from("[object Object]")),
        |weak, key| {
            if key == "foreground" {
                let terminal = weak.upgrade().unwrap();
                terminal
                    .write("\x1b[?1049halt")
                    .map_err(|e| OptionError::Dispatch(EmitError::callback(e)))?;
                terminal
                    .resize(12, 3)
                    .map_err(|e| OptionError::Dispatch(EmitError::callback(e)))?;
                terminal
                    .options()
                    .set("allowProposedApi", OptionValue::Bool(false))
                    .map_err(|e| OptionError::Dispatch(EmitError::callback(e)))?;
            }
            Ok(OptionValue::Undefined)
        },
    );
    t.options()
        .set("theme", OptionValue::Object(theme))
        .unwrap();
    let actual = a
        .serialize_as_html(&HtmlOptions {
            include_global_background: true,
            ..HtmlOptions::default()
        })
        .unwrap();
    let expected: Vec<u16> = include_str!("fixtures/serialize-html-reentrant.tsv")
        .trim()
        .split(',')
        .map(|s| u16::from_str_radix(s, 16).unwrap())
        .collect();
    assert_eq!(
        actual, expected,
        "exact source HTML with synchronous header callbacks"
    );
    let output = String::from_utf16(&actual).unwrap();
    assert_eq!(
        output.matches("<div><span>").count(),
        2,
        "source range established before resize"
    );
    assert!(
        output.contains("<div><span>normal      </span></div>"),
        "{output}"
    );
    assert!(
        output.contains("<div><span>          </span></div>"),
        "last source end column established before resize: {output}"
    );
    assert!(
        !output.contains("alt"),
        "source buffer captured before alternate activation"
    );
    assert!(
        a.serialize_as_html(&HtmlOptions::default()).is_err(),
        "later calls do observe changed proposed-API option"
    );
}
