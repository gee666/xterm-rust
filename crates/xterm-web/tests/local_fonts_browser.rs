#![cfg(target_arch = "wasm32")]

use js_sys::{Function, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::local_fonts::LocalFonts;

wasm_bindgen_test_configure!(run_in_browser);

fn environment(body: &str) -> JsValue {
    Function::new_no_args(body)
        .call0(&JsValue::UNDEFINED)
        .unwrap()
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn property(value: &JsValue, name: &str) -> JsValue {
    Reflect::get(value, &JsValue::from_str(name)).unwrap()
}

#[wasm_bindgen_test(async)]
async fn source_font_resolution_operation_traces_match_in_browser() {
    let cases = js_sys::JSON::parse(include_str!("fixtures/local-font-cases.json")).unwrap();
    for case in js_sys::Array::from(&cases) {
        let name = property(&case, "name").as_string().unwrap();
        let env = environment(&property(&case, "environment").as_string().unwrap());
        let catalog = LocalFonts::default();
        let calls = js_sys::Array::from(&property(&case, "calls"));
        let results = js_sys::Array::from(&property(&case, "results"));
        for (index, family) in calls.iter().enumerate() {
            let expected = results.get(index as u32);
            let actual = catalog
                .load_from(&env, &utf16(&family.as_string().unwrap()))
                .await;
            match property(&expected, "status").as_string().unwrap().as_str() {
                "font" => {
                    let actual = actual.unwrap().unwrap();
                    let expected_bytes = js_sys::Array::from(&property(&expected, "bytes"));
                    let bytes: Vec<_> = expected_bytes
                        .iter()
                        .map(|value| value.as_f64().unwrap() as u8)
                        .collect();
                    assert_eq!(actual.bytes, bytes, "{name}, call {index}");
                }
                "none" => assert!(actual.unwrap().is_none(), "{name}, call {index}"),
                "error" => {
                    let error = actual.err().unwrap();
                    assert_eq!(
                        property(&error, "name"),
                        property(&expected, "name"),
                        "{name}, call {index}"
                    );
                    assert_eq!(
                        property(&error, "message"),
                        property(&expected, "message"),
                        "{name}, call {index}"
                    );
                }
                other => panic!("invalid fixture status {other}"),
            }
        }
        let counters = property(&case, "counters");
        for key in js_sys::Object::keys(&js_sys::Object::from(counters.clone())) {
            assert_eq!(
                Reflect::get(&env, &key).unwrap(),
                Reflect::get(&counters, &key).unwrap(),
                "{name} counter"
            );
        }
    }
}

fn count(env: &JsValue, key: &str) -> f64 {
    Reflect::get(env, &key.into()).unwrap().as_f64().unwrap()
}

#[wasm_bindgen_test(async)]
async fn modern_queries_cache_metadata_and_read_first_blob() {
    let env = environment("const env={navigator:{},queries:0,blobs:0};env.window={queryLocalFonts:async()=>{env.queries++;return [{family:'CodeFont',blob:async()=>{env.blobs++;return new Blob([new Uint8Array([1,2,3])])}},{family:'CodeFont',blob:async()=>new Blob([new Uint8Array([9])])}]}};return env;");
    let catalog = LocalFonts::default();
    let font = catalog
        .load_from(&env, &utf16("CodeFont, monospace"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(font.family, utf16("CodeFont"));
    assert_eq!(font.bytes, [1, 2, 3]);
    let cloned = catalog.clone();
    assert_eq!(
        cloned
            .load_from(&env, &utf16("CodeFont"))
            .await
            .unwrap()
            .unwrap()
            .bytes,
        [1, 2, 3]
    );
    assert_eq!(count(&env, "queries"), 1.0);
    assert_eq!(count(&env, "blobs"), 2.0);
    assert!(catalog
        .load_from(&env, &utf16("monospace, CodeFont"))
        .await
        .unwrap()
        .is_none());
    assert!(catalog
        .load_from(&env, &utf16("\"CodeFont\""))
        .await
        .unwrap()
        .is_none());
}

#[wasm_bindgen_test(async)]
async fn legacy_permission_typeerror_is_ignored_and_takes_priority() {
    let env = environment("const env={permissions:0,legacy:0,modern:0};env.navigator={permissions:{request:async()=>{env.permissions++;throw new TypeError('unsupported')}},fonts:{query:async()=>{env.legacy++;return [{family:'Legacy',blob:async()=>new Blob([new Uint8Array([7])])}]}}};env.window={queryLocalFonts:async()=>{env.modern++;return []}};return env;");
    let catalog = LocalFonts::default();
    assert_eq!(
        catalog
            .load_from(&env, &utf16("Legacy"))
            .await
            .unwrap()
            .unwrap()
            .bytes,
        [7]
    );
    assert_eq!(count(&env, "permissions"), 1.0);
    assert_eq!(count(&env, "legacy"), 1.0);
    assert_eq!(count(&env, "modern"), 0.0);
}

#[wasm_bindgen_test(async)]
async fn denied_permission_rejects_and_remains_retryable() {
    let env = environment("const env={permissions:0,queries:0,window:{}};env.navigator={permissions:{request:async()=>{env.permissions++;return {state:'denied'}}},fonts:{query:async()=>{env.queries++;return []}}};return env;");
    let catalog = LocalFonts::default();
    for _ in 0..2 {
        let error = catalog
            .load_from(&env, &utf16("CodeFont"))
            .await
            .err()
            .unwrap();
        assert_eq!(
            Reflect::get(&error, &"message".into()).unwrap(),
            "Permission to access local fonts not granted."
        );
    }
    assert_eq!(count(&env, "permissions"), 2.0);
    assert_eq!(count(&env, "queries"), 0.0);
}

#[wasm_bindgen_test(async)]
async fn query_failure_caches_empty_catalog_and_does_not_retry() {
    let env = environment("const env={navigator:{},queries:0};env.window={queryLocalFonts:async()=>{env.queries++;throw new Error('intentional query failure')}};return env;");
    let catalog = LocalFonts::default();
    for _ in 0..2 {
        assert!(catalog
            .load_from(&env, &utf16("CodeFont"))
            .await
            .unwrap()
            .is_none());
    }
    assert_eq!(count(&env, "queries"), 1.0);
}

#[wasm_bindgen_test(async)]
async fn missing_blob_stops_resolution_and_blob_errors_propagate() {
    let env = environment("return {navigator:{},window:{queryLocalFonts:async()=>[{family:'NoBlob'},{family:'Broken',blob:async()=>{throw new Error('blob failure')}}]}};");
    let catalog = LocalFonts::default();
    assert!(catalog
        .load_from(&env, &utf16("NoBlob, Broken"))
        .await
        .unwrap()
        .is_none());
    let error = catalog
        .load_from(&env, &utf16("Broken"))
        .await
        .err()
        .unwrap();
    assert_eq!(
        Reflect::get(&error, &"message".into()).unwrap(),
        "blob failure"
    );
}

#[wasm_bindgen_test(async)]
async fn missing_apis_and_missing_families_resolve_without_a_font() {
    let env = environment("return {navigator:{},window:{}};");
    assert!(LocalFonts::default()
        .load_from(&env, &utf16("Missing, sans-serif"))
        .await
        .unwrap()
        .is_none());
    let env = environment("return {navigator:{fonts:{query:async()=>[]}},window:{}};");
    assert!(LocalFonts::default()
        .load_from(&env, &utf16("Missing"))
        .await
        .unwrap()
        .is_none());
}
