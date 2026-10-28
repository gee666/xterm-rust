// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Event-driven waits for the real Terminal. The deadline fails; it is not a sleep.
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use xterm_web::Terminal;

#[wasm_bindgen(
    inline_js = "export function waitAccessibilityText(element,expected){return new Promise((resolve,reject)=>{const DEADLINE_MS=10000;let timer;const observer=new MutationObserver(check);function finish(error){observer.disconnect();clearTimeout(timer);error?reject(error):resolve();}function check(){if(element.textContent===expected)finish();}observer.observe(element,{childList:true,characterData:true,subtree:true});timer=setTimeout(()=>finish(new Error('Accessibility text deadline: '+JSON.stringify(element.textContent)+' expected '+JSON.stringify(expected))),DEADLINE_MS);check();});}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = waitAccessibilityText)]
    pub fn wait_text(
        element: &HtmlElement,
        expected: &js_sys::JsString,
    ) -> Result<Promise, JsValue>;
}
pub fn write(terminal: &Terminal, data: &[u16]) -> Promise {
    let value = JsValue::from(xterm_web::utf16::to_js_string(data));
    Promise::new(&mut |resolve, reject| {
        if let Err(error) = terminal.write(value.clone(), Some(resolve)) {
            reject.call1(&JsValue::UNDEFINED, &error).unwrap();
        }
    })
}
