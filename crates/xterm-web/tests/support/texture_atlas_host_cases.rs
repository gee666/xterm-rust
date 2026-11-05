// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
#[wasm_bindgen(module = "/tests/reference/texture-atlas-host.mjs")]
extern "C" {
    #[wasm_bindgen(catch,js_name=atlasHostPhase)]
    fn host_phase(kind: &str, spec: &str, invoke: &JsValue) -> Result<String, JsValue>;
}
#[wasm_bindgen_test]
fn host_errors_keep_identity_order_and_partial_canvas_state() {
    let source: Value = serde_json::from_str(&source_scenario("styles").unwrap()).unwrap();
    let spec = &source["spec"];
    let spec_json = serde_json::to_string(spec).unwrap();
    for kind in ["style", "method", "upload", "invisible", "imageCtor"] {
        let expected: Value =
            serde_json::from_str(&host_phase(kind, &spec_json, &JsValue::NULL).unwrap()).unwrap();
        let atlas = make_atlas(spec);
        let invoke_atlas = atlas.clone();
        let invoke = Closure::wrap(Box::new(move || -> Result<JsValue, JsValue> {
            let fg = if kind == "invisible" { 0x40000000 } else { 0 };
            let g = invoke_atlas.get_glyph(GlyphKey::Number(65), 0, fg, 0, false, None)?;
            Ok(serde_json::to_string(&glyph(&g)).unwrap().into())
        }) as Box<dyn Fn() -> Result<JsValue, JsValue>>);
        let actual = host_phase(kind, &spec_json, invoke.as_ref());
        atlas.dispose().unwrap();
        let actual: Value = serde_json::from_str(&actual.unwrap()).unwrap();
        assert_same(&actual, &expected, kind);
        assert_eq!(
            expected["identity"],
            kind != "invisible" && kind != "imageCtor"
        );
    }
}
