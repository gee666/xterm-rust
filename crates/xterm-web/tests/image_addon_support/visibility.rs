// Copyright (c) 2020 The xterm.js authors. MIT license.
use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

struct LongOutput(JsValue);
impl Drop for LongOutput {
    fn drop(&mut self) {
        call(&self.0, "remove", &[]);
    }
}

#[wasm_bindgen_test(async)]
async fn paired_images_remain_visible_when_harness_output_exceeds_viewport() {
    let _output = LongOutput(eval(
        "(()=>{const e=document.createElement('div');e.style.height='4096px';document.body.appendChild(e);return e})()",
    ));
    let pair = Pair::new(true, "({showPlaceholder:false})");
    pair.ready().await;
    pair.write(sixel()).await;
    let source_terminal = get(&pair.source, "terminal");
    let source_renderer = get(&get(&source_terminal, "_core"), "_renderService");
    assert_eq!(get(&source_renderer, "_isPaused"), JsValue::FALSE);
    let visible = js_sys::Function::new_with_args(
        "element",
        "const r=element.getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight&&r.left>=0&&r.right<=innerWidth",
    );
    for element in [
        JsValue::from(pair.terminal.element().unwrap()),
        get(&source_terminal, "element"),
    ] {
        assert_eq!(
            visible.call1(&JsValue::UNDEFINED, &element).unwrap(),
            JsValue::TRUE
        );
    }
    pair.terminal.resize(24.0, 12.0).unwrap();
    call(&source_terminal, "resize", &[24.into(), 12.into()]);
    pair.frame().await;
    pair.compare();
}
