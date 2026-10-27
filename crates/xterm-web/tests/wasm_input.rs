#![cfg(target_arch = "wasm32")]

#[path = "support/input_dom.rs"]
mod support;
use support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::utf16::{from_js_string, to_js_string};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn paste_normalizes_brackets_sanitizes_and_clears_actual_textarea() {
    let setup = Setup::new();
    setup.host.options.set(xterm_web::input::dom::InputOptions {
        bracketed_paste: true,
        ..Default::default()
    });
    setup.dom.textarea.set_value("old value");
    setup.input.paste(&units("A\r\nB\n\x1b[201~")).unwrap();
    assert_eq!(
        *setup.host.data.borrow(),
        vec![(units("\x1b[200~A\rB\r␛[201~\x1b[201~"), true)]
    );
    assert_eq!(setup.dom.textarea.value(), "");
    setup.host.disabled.set(true);
    setup.input.paste(&units("not sent")).unwrap();
    assert_eq!(setup.host.data.borrow().len(), 1);
}

#[wasm_bindgen_test]
fn actual_clipboard_events_copy_selection_paste_once_and_stop_bubbling() {
    let setup = Setup::new();
    *setup.host.selection.borrow_mut() = units("selected 😀");
    let clipboard = web_sys::DataTransfer::new().unwrap();
    let copy = event(
        "ClipboardEvent",
        "copy",
        &[("clipboardData", clipboard.clone().into())],
    );
    setup.dom.element.dispatch_event(&copy).unwrap();
    assert!(copy.default_prevented());
    assert_eq!(clipboard.get_data("text/plain").unwrap(), "selected 😀");
    clipboard.set_data("text/plain", "one\ntwo").unwrap();
    let paste = event(
        "ClipboardEvent",
        "paste",
        &[("clipboardData", clipboard.into())],
    );
    setup.dom.textarea.dispatch_event(&paste).unwrap();
    assert_eq!(*setup.host.data.borrow(), vec![(units("one\rtwo"), true)]);
    // Copy without a terminal selection is left for the host application.
    setup.host.selection.borrow_mut().clear();
    let copy = event("ClipboardEvent", "copy", &[]);
    setup.dom.element.dispatch_event(&copy).unwrap();
    assert!(!copy.default_prevented());
}

#[wasm_bindgen_test]
fn context_menu_positions_focuses_and_selects_actual_textarea() {
    let setup = Setup::new();
    *setup.host.selection.borrow_mut() = units("menu selection");
    setup.host.options.set(xterm_web::input::dom::InputOptions {
        right_click_selects_word: true,
        ..Default::default()
    });
    let rect = setup.dom.screen.get_bounding_client_rect();
    let event = event(
        "MouseEvent",
        "contextmenu",
        &[("clientX", 31.into()), ("clientY", 52.into())],
    );
    setup.dom.element.dispatch_event(&event).unwrap();
    assert_eq!(
        setup
            .dom
            .textarea
            .style()
            .get_property_value("left")
            .unwrap(),
        format!("{}px", 21.0 - rect.left())
    );
    assert_eq!(
        setup
            .dom
            .textarea
            .style()
            .get_property_value("top")
            .unwrap(),
        format!("{}px", 42.0 - rect.top())
    );
    assert_eq!(
        setup
            .dom
            .textarea
            .style()
            .get_property_value("width")
            .unwrap(),
        "20px"
    );
    assert_eq!(
        setup
            .dom
            .textarea
            .style()
            .get_property_value("z-index")
            .unwrap(),
        "1000"
    );
    assert_eq!(setup.dom.textarea.value(), "menu selection");
    assert_eq!(setup.dom.textarea.selection_start().unwrap(), Some(0));
    assert_eq!(setup.dom.textarea.selection_end().unwrap(), Some(14));
    assert_eq!(
        setup.dom.document.active_element().unwrap(),
        setup.dom.textarea
    );
    assert_eq!(*setup.host.effects.borrow(), vec!["right-click-select"]);
}

#[wasm_bindgen_test(async)]
async fn composition_overlay_deferred_suffix_and_real_event_order() {
    let setup = Setup::new();
    setup.dom.textarea.set_value("一二");
    setup.dom.textarea.set_selection_range(1, 1).unwrap();
    setup
        .dom
        .textarea
        .dispatch_event(&event("CompositionEvent", "compositionstart", &[]))
        .unwrap();
    assert!(setup.input.is_composing());
    assert!(setup.dom.composition.class_list().contains("active"));
    assert_eq!(
        setup
            .dom
            .composition
            .style()
            .get_property_value("left")
            .unwrap(),
        "711px"
    );
    assert_eq!(
        setup
            .dom
            .composition
            .style()
            .get_property_value("max-width")
            .unwrap(),
        "9px"
    );
    assert_eq!(
        setup
            .dom
            .textarea
            .style()
            .get_property_value("top")
            .unwrap(),
        "36px"
    );
    setup
        .dom
        .textarea
        .dispatch_event(&event(
            "CompositionEvent",
            "compositionupdate",
            &[("data", "一".into())],
        ))
        .unwrap();
    assert_eq!(
        setup.dom.composition.text_content().unwrap(),
        "\u{200e}一\u{200e}"
    );
    setup.dom.textarea.set_value("一一二");
    setup.dom.textarea.set_selection_range(2, 2).unwrap();
    next_turn().await;
    setup
        .dom
        .textarea
        .dispatch_event(&event("CompositionEvent", "compositionend", &[]))
        .unwrap();
    assert!(!setup.dom.composition.class_list().contains("active"));
    assert!(setup.host.data.borrow().is_empty());
    setup.dom.textarea.set_value("一一1二");
    next_turn().await;
    assert_eq!(*setup.host.data.borrow(), vec![(units("一1"), true)]);
}

#[wasm_bindgen_test(async)]
async fn dispose_cancels_pending_composition_and_removes_all_input_listeners() {
    let mut setup = Setup::new();
    setup
        .dom
        .textarea
        .dispatch_event(&event("CompositionEvent", "compositionstart", &[]))
        .unwrap();
    setup
        .dom
        .textarea
        .dispatch_event(&event(
            "CompositionEvent",
            "compositionupdate",
            &[("data", "한".into())],
        ))
        .unwrap();
    setup.dom.textarea.set_value("한");
    next_turn().await;
    setup
        .dom
        .textarea
        .dispatch_event(&event("CompositionEvent", "compositionend", &[]))
        .unwrap();
    setup.input.dispose().unwrap();
    next_turn().await;
    assert!(setup.host.data.borrow().is_empty());
    setup
        .dom
        .textarea
        .dispatch_event(&event(
            "KeyboardEvent",
            "keydown",
            &[("keyCode", 13.into()), ("key", "Enter".into())],
        ))
        .unwrap();
    assert!(setup.host.data.borrow().is_empty());
    assert!(setup.input.paste(&units("disposed")).is_err());
}

#[wasm_bindgen_test]
fn keyboard_data_onkey_order_cancellation_veto_and_input_suppression() {
    let setup = Setup::new();
    let down = event(
        "KeyboardEvent",
        "keydown",
        &[
            ("keyCode", 13.into()),
            ("key", "Enter".into()),
            ("code", "Enter".into()),
        ],
    );
    setup.dom.textarea.dispatch_event(&down).unwrap();
    assert!(down.default_prevented());
    assert_eq!(*setup.host.data.borrow(), vec![(vec![13], true)]);
    assert_eq!(
        *setup.host.effects.borrow(),
        vec!["UpdateCursorStyle", "OnKey([13])", "ShowCursor"]
    );
    let input = event(
        "InputEvent",
        "input",
        &[
            ("inputType", "insertText".into()),
            ("data", "duplicate".into()),
            ("composed", true.into()),
        ],
    );
    setup.dom.textarea.dispatch_event(&input).unwrap();
    assert_eq!(setup.host.data.borrow().len(), 1);
    setup
        .dom
        .textarea
        .dispatch_event(&event(
            "KeyboardEvent",
            "keyup",
            &[("keyCode", 13.into()), ("key", "Enter".into())],
        ))
        .unwrap();
    let emoji = event(
        "InputEvent",
        "input",
        &[
            ("inputType", "insertText".into()),
            ("data", "😀".into()),
            ("composed", true.into()),
        ],
    );
    setup.dom.textarea.dispatch_event(&emoji).unwrap();
    assert_eq!(setup.host.data.borrow()[1].0, units("😀"));
    setup.host.veto.set(true);
    setup.dom.textarea.dispatch_event(&down).unwrap();
    assert_eq!(setup.host.data.borrow().len(), 2);
}

#[wasm_bindgen_test]
fn keyboard_uses_lossless_utf16_and_win32_priority() {
    let setup = Setup::new();
    let mut snapshot = setup.host.keyboard.get();
    snapshot.modes.win32_extension = true;
    snapshot.modes.win32_mode = true;
    snapshot.modes.kitty_extension = true;
    snapshot.modes.kitty_flags = 8;
    setup.host.keyboard.set(snapshot);
    let key = to_js_string(&[0xd800]);
    let down = event(
        "KeyboardEvent",
        "keydown",
        &[
            ("keyCode", 65.into()),
            ("key", key.clone().into()),
            ("code", "KeyA".into()),
        ],
    );
    assert_eq!(
        from_js_string(
            &js_sys::Reflect::get(down.as_ref(), &JsValue::from_str("key"))
                .unwrap()
                .dyn_into::<js_sys::JsString>()
                .unwrap()
        ),
        vec![0xd800]
    );
    setup.dom.textarea.dispatch_event(&down).unwrap();
    let output = String::from_utf16(&setup.host.data.borrow()[0].0).unwrap();
    assert_eq!(output, "\x1b[65;30;55296;1;0;1_");
    assert!(down.default_prevented());
}
