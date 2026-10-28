use std::rc::Rc;
use xterm_core::{
    event::EmitError,
    headless::Terminal,
    options::{OptionError, OptionObject, OptionValue},
    serialize::{HtmlOptions, SerializeAddon},
};

#[test]
fn header_reset_retargets_the_shared_public_view_and_activation_stays_lazy() {
    let t = Rc::new(Terminal::new(10, 2, 20).unwrap());
    let addon = Rc::new(SerializeAddon::new());
    let cache = t.buffer_cache_for_addon().upgrade().unwrap();
    assert!(cache.get().is_none());
    t.load_addon(addon.clone()).unwrap();
    assert!(
        cache.get().is_none(),
        "source activation does not initialize namespace"
    );
    t.write("normal").unwrap();
    let theme = OptionObject::with_fallible_properties(
        Rc::downgrade(&t),
        |_| Ok(OptionValue::from("[object Object]")),
        |weak, key| {
            if key == "foreground" {
                weak.upgrade()
                    .unwrap()
                    .reset()
                    .map_err(|e| OptionError::Dispatch(EmitError::callback(e)))?;
            }
            Ok(OptionValue::Undefined)
        },
    );
    t.options()
        .set("theme", OptionValue::Object(theme))
        .unwrap();
    let actual = addon
        .serialize_as_html(&HtmlOptions {
            include_global_background: true,
            ..HtmlOptions::default()
        })
        .unwrap();
    let expected:Vec<u16>="<html><body><!--StartFragment--><pre><div style='color: #ffffff; background-color: #000000; font-family: monospace; font-size: 15px;'><div><span>          </span></div><div><span>          </span></div></div></pre><!--EndFragment--></body></html>".encode_utf16().collect();
    assert_eq!(
        actual, expected,
        "reset retargets the source view rather than serializing stale normal cells"
    );
    assert!(
        Rc::ptr_eq(cache.get().unwrap(), &t.buffer().unwrap()),
        "actual public namespace, not a detached private cache"
    );
}
