#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved. MIT license.
// Uses the real exported Rust Terminal, real jobs and MutationObserver deadlines.
#[path = "accessibility_support/real_jobs.rs"]
mod jobs;
#[path = "accessibility_support/text.rs"]
mod strings;
use js_sys::{Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use strings::{attribute, text, units};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, HtmlTextAreaElement};
use xterm_web::{BrowserRuntime, Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(
    inline_js = "export function accessibilityKey(textarea,key){const code=key.codePointAt(0);for(const [type,value] of [['keydown',key.toUpperCase().codePointAt(0)],['keypress',code],['keyup',key.toUpperCase().codePointAt(0)]]){const event=new KeyboardEvent(type,{key,code:'KeyX',bubbles:true,cancelable:true});Object.defineProperty(event,'keyCode',{value});Object.defineProperty(event,'which',{value});Object.defineProperty(event,'charCode',{value:type==='keypress'?code:0});textarea.dispatchEvent(event);}}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = accessibilityKey)]
    fn key(textarea: &HtmlTextAreaElement, value: &str) -> Result<(), JsValue>;
}
#[wasm_bindgen(module = "/tests/accessibility_support/reentry.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = observeResetRender)]
    fn observe_reset_render(
        root: &HtmlElement,
        callback: &Function,
    ) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch, js_name = resizeOnComparison)]
    fn resize_on_comparison(node: &web_sys::Node, callback: &Function)
        -> Result<Function, JsValue>;
}
struct Opened {
    terminal: Rc<Terminal>,
    parent: HtmlElement,
    style: HtmlElement,
}
impl Opened {
    fn new(runtime: &BrowserRuntime) -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let options = Object::new();
        for (name, value) in [
            ("cols", JsValue::from_f64(8.0)),
            ("rows", JsValue::from_f64(3.0)),
            ("screenReaderMode", JsValue::TRUE),
        ] {
            Reflect::set(&options, &name.into(), &value)?;
        }
        let terminal = Rc::new(Terminal::new(options.into())?);
        terminal.set_runtime(runtime)?;
        let style: HtmlElement = document.create_element("style")?.unchecked_into();
        style.set_text_content(Some(xterm_web::XTERM_CSS));
        document.body().unwrap().append_child(&style)?;
        let parent: HtmlElement = document.create_element("div")?.unchecked_into();
        document.body().unwrap().append_child(&parent)?;
        let opened = Self {
            terminal,
            parent,
            style,
        };
        opened.terminal.open(opened.parent.clone().into())?;
        Ok(opened)
    }
    fn row(&self) -> HtmlElement {
        self.terminal
            .element()
            .unwrap()
            .query_selector(".xterm-accessibility-tree > div")
            .unwrap()
            .unwrap()
            .unchecked_into()
    }
    fn live(&self) -> HtmlElement {
        self.terminal
            .element()
            .unwrap()
            .query_selector(".live-region")
            .unwrap()
            .unwrap()
            .unchecked_into()
    }
    async fn write(&self, value: &[u16]) {
        JsFuture::from(jobs::write(&self.terminal, value))
            .await
            .unwrap();
    }
    async fn expect(&self, element: &HtmlElement, value: &[u16]) {
        JsFuture::from(jobs::wait_text(element, &xterm_web::utf16::to_js_string(value)).unwrap())
            .await
            .unwrap();
        assert_eq!(text(element), value);
    }
}
impl Drop for Opened {
    fn drop(&mut self) {
        self.terminal.dispose().unwrap();
        self.parent.remove();
        self.style.remove();
    }
}

#[wasm_bindgen_test(async)]
async fn real_terminal_writes_deliver_utf16_to_tree_and_live_region_then_toggle_cleanly() {
    let runtime = BrowserRuntime::new();
    let f = Opened::new(&runtime).unwrap();
    let root = f.terminal.element().unwrap();
    let textarea = f.terminal.textarea().unwrap();
    let mut expected = units("A字");
    f.write(&expected).await;
    f.expect(&f.row(), &expected).await;
    f.expect(&f.live(), &expected).await;
    f.write(&[0xd800, 66]).await;
    expected.extend([0xd800, 66]);
    f.expect(&f.row(), &expected).await;
    f.expect(&f.live(), &expected).await;
    assert_eq!(attribute(&f.row(), "aria-posinset"), units("1"));
    let old = f.live();
    let old_text = text(&old);
    Reflect::set(
        &f.terminal.options(),
        &"screenReaderMode".into(),
        &JsValue::FALSE,
    )
    .unwrap();
    assert!(root
        .query_selector(".xterm-accessibility")
        .unwrap()
        .is_none());
    f.write(&[81]).await;
    expected.push(81);
    assert_eq!(text(&old), old_text);
    Reflect::set(
        &f.terminal.options(),
        &"screenReaderMode".into(),
        &JsValue::TRUE,
    )
    .unwrap();
    f.expect(&f.row(), &expected).await;
    assert_eq!(f.terminal.element().unwrap(), root);
    assert_eq!(f.terminal.textarea().unwrap(), textarea);
    assert_eq!(
        root.query_selector_all(".xterm-accessibility")
            .unwrap()
            .length(),
        1
    );
    f.terminal.dispose().unwrap();
    assert!(root.parent_node().is_none());
}
#[wasm_bindgen_test(async)]
async fn reset_from_first_row_setter_keeps_original_buffer_for_the_whole_render() {
    let f = Opened::new(&BrowserRuntime::new()).unwrap();
    f.write(&units("N0\r\nN1\r\nN2")).await;
    f.expect(&f.row(), &units("N0")).await;
    let terminal = Rc::clone(&f.terminal);
    let reset = Closure::<dyn FnMut()>::new(move || terminal.reset().unwrap());
    let snapshot = observe_reset_render(
        &f.terminal.element().unwrap(),
        reset.as_ref().unchecked_ref(),
    )
    .unwrap();
    f.terminal.refresh(0.0, 2.0).unwrap();
    let snapshot = js_sys::Array::from(&JsFuture::from(snapshot).await.unwrap());
    assert_eq!(snapshot.length(), 3);
    for (index, expected) in ["N0", "N1", "N2"].into_iter().enumerate() {
        assert_eq!(
            snapshot.get(index as u32).as_string().as_deref(),
            Some(expected)
        );
    }
}

#[wasm_bindgen_test(async)]
async fn selection_clamps_to_live_last_row_after_comparison_resizes_terminal() {
    let f = Opened::new(&BrowserRuntime::new()).unwrap();
    f.write(&units("N0\r\nN1\r\nN2")).await;
    f.expect(&f.row(), &units("N0")).await;
    let document = web_sys::window().unwrap().document().unwrap();
    let after = document.create_element("p").unwrap();
    after.set_text_content(Some("outside"));
    document.body().unwrap().append_child(&after).unwrap();
    let anchor = f.row().first_child().unwrap();
    let focus = after.first_child().unwrap();
    let selection = document.get_selection().unwrap().unwrap();
    selection
        .set_base_and_extent(&anchor, 0, &focus, 7)
        .unwrap();
    let terminal = Rc::clone(&f.terminal);
    let resize = Closure::<dyn FnMut()>::new(move || terminal.resize(8.0, 2.0).unwrap());
    let cleanup = resize_on_comparison(&anchor, resize.as_ref().unchecked_ref()).unwrap();
    document
        .dispatch_event(&web_sys::Event::new("selectionchange").unwrap())
        .unwrap();
    cleanup.call0(&JsValue::UNDEFINED).unwrap();
    assert_eq!(f.terminal.rows().unwrap(), 2.0);
    assert_eq!(
        f.terminal.get_selection().unwrap().as_string().as_deref(),
        Some("N0\r\nN1")
    );
    selection.remove_all_ranges().unwrap();
    after.remove();
}

#[wasm_bindgen_test(async)]
async fn actual_keyboard_on_key_suppresses_only_the_following_matching_output_echo() {
    let runtime = BrowserRuntime::new();
    let f = Opened::new(&runtime).unwrap();
    f.write(&units("prompt")).await;
    f.expect(&f.live(), &units("prompt")).await;
    let data = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&data);
    let callback = Closure::<dyn Fn(JsValue)>::new(move |value: JsValue| {
        assert!(value.is_string());
        let value = xterm_web::utf16::from_js_string(&value.unchecked_into());
        output.borrow_mut().push(value);
    });
    let subscription = f
        .terminal
        .on_data()
        .call1(&JsValue::UNDEFINED, callback.as_ref())
        .unwrap();
    f.terminal.focus().unwrap();
    key(&f.terminal.textarea().unwrap(), "x").unwrap();
    assert_eq!(*data.borrow(), vec![units("x")]);
    assert!(text(&f.live()).is_empty());
    f.write(&units("x")).await;
    f.expect(&f.row(), &units("promptx")).await;
    assert!(text(&f.live()).is_empty());
    let dispose = Reflect::get(&subscription, &"dispose".into())
        .unwrap()
        .unchecked_into::<Function>();
    dispose.call0(&subscription).unwrap();
}
#[wasm_bindgen_test]
fn runtime_strings_keep_raw_values_share_across_terminals_and_apply_prompt_only_on_open() {
    let runtime = BrowserRuntime::new();
    let first = runtime.strings();
    let second = runtime.strings();
    assert_ne!(first, second);
    let label = Function::new_no_args("return {calls:0,toString(){this.calls++;return 'Prompt';}}")
        .call0(&JsValue::UNDEFINED)
        .unwrap();
    Reflect::set(&first, &"promptLabel".into(), &label).unwrap();
    assert_eq!(Reflect::get(&second, &"promptLabel".into()).unwrap(), label);
    assert_eq!(
        Reflect::get(&label, &"calls".into()).unwrap().as_f64(),
        Some(0.0)
    );
    let one = Opened::new(&runtime).unwrap();
    let two = Opened::new(&runtime).unwrap();
    assert_eq!(
        Reflect::get(&label, &"calls".into()).unwrap().as_f64(),
        Some(2.0)
    );
    assert_eq!(
        attribute(
            one.terminal.textarea().unwrap().unchecked_ref(),
            "aria-label"
        ),
        units("Prompt")
    );
    let localized = xterm_web::utf16::to_js_string(&[0xd800, 0xfeff]);
    Reflect::set(&second, &"promptLabel".into(), &localized).unwrap();
    assert_eq!(
        attribute(
            two.terminal.textarea().unwrap().unchecked_ref(),
            "aria-label"
        ),
        units("Prompt")
    );
    let three = Opened::new(&runtime).unwrap();
    assert_eq!(
        attribute(
            three.terminal.textarea().unwrap().unchecked_ref(),
            "aria-label"
        ),
        [0xd800, 0xfeff]
    );
    let separate = BrowserRuntime::new();
    assert_eq!(
        Reflect::get(&separate.strings(), &"promptLabel".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("Terminal input")
    );
}
