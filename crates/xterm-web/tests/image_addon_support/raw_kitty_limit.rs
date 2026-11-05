// Copyright (c) 2026 The xterm.js authors. MIT license.
use crate::support::*;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use xterm_web::{image::ImageAddon, Terminal};

fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}
fn token(value: &JsValue) -> JsValue {
    js_sys::Function::new_with_args("v", "return v?.token")
        .call1(&JsValue::UNDEFINED, value)
        .unwrap()
}
fn raw(addon: &JsValue) -> JsValue {
    get(&get(addon, "_opts"), "kittySizeLimit")
}
struct Activation {
    addon: ImageAddon,
    terminal: Terminal,
    source: JsValue,
}
impl Activation {
    fn new(expression: &str) -> Self {
        // Construct before allocating either terminal, including in old-code controls.
        let addon = ImageAddon::new(eval(expression)).unwrap();
        let source = js_sys::Function::new_with_args(
            "options",
            "return {addon:new SourceImageAddon(options),terminal:new SourceTerminal({cols:20,rows:10,allowProposedApi:true,logLevel:'off'})}",
        )
        .call1(&JsValue::UNDEFINED, &eval(expression))
        .unwrap();
        let terminal = Terminal::new(eval(
            "({cols:20,rows:10,allowProposedApi:true,logLevel:'off'})",
        ))
        .unwrap();
        Self {
            addon,
            terminal,
            source,
        }
    }
    fn activate(&self) -> (Result<(), JsValue>, Result<JsValue, JsValue>) {
        let actual = self.addon.activate(&self.terminal);
        let expected = js_sys::Function::new_with_args("s", "return s.addon.activate(s.terminal)")
            .call1(&JsValue::UNDEFINED, &self.source);
        (actual, expected)
    }
    fn value(&self, rust: bool) -> JsValue {
        if rust {
            get(&self.addon.option_inspection(), "kittySizeLimit")
        } else {
            raw(&get(&self.source, "addon"))
        }
    }
    fn compare_membership(&self) {
        let source = get(&get(&self.source, "addon"), "_handlers");
        let actual = self.addon.handler_inspection();
        assert_eq!(get(&actual, "size"), get(&source, "size"));
        for key in ["sixel", "iip", "kitty"] {
            assert_eq!(
                call(&actual, "has", &[key.into()]),
                call(&source, "has", &[key.into()])
            );
        }
    }
}
impl Drop for Activation {
    fn drop(&mut self) {
        self.addon.dispose().unwrap();
        self.terminal.dispose().unwrap();
        call(&get(&self.source, "addon"), "dispose", &[]);
        call(&get(&self.source, "terminal"), "dispose", &[]);
    }
}

#[wasm_bindgen_test]
fn raw_kitty_limit_constructor_retains_values_and_first_copy() {
    for expression in [
        "undefined",
        "null",
        "false",
        "true",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "-Infinity",
        "1",
        "-1",
        "''",
        "'1000'",
        "'no'",
        "0n",
        "1000n",
        "Symbol('size')",
        "{}",
        "[]",
        "new Number(1000)",
    ] {
        let value = eval(expression);
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"kittySizeLimit".into(), &value).unwrap();
        let actual = ImageAddon::new(options.clone().into()).unwrap();
        let expected = js_sys::Function::new_with_args("o", "return new SourceImageAddon(o)")
            .call1(&JsValue::UNDEFINED, &options)
            .unwrap();
        assert!(js_sys::Object::is(
            &get(&actual.option_inspection(), "kittySizeLimit"),
            &value
        ));
        assert!(js_sys::Object::is(&raw(&expected), &value));
        actual.reset().unwrap();
        actual.dispose().unwrap();
        assert!(js_sys::Object::is(
            &get(&actual.option_inspection(), "kittySizeLimit"),
            &value
        ));
        call(&expected, "dispose", &[]);
    }
    for rust in [false, true] {
        let input = eval("(()=>{let reads=0;const first={[Symbol.toPrimitive](){throw Error('eager')}},second={};return {first,get reads(){return reads},options:{get kittySizeLimit(){return ++reads===1?first:second}}}})()");
        let addon: JsValue = if rust {
            ImageAddon::new(get(&input, "options")).unwrap().into()
        } else {
            js_sys::Function::new_with_args("o", "return new SourceImageAddon(o)")
                .call1(&JsValue::UNDEFINED, &get(&input, "options"))
                .unwrap()
        };
        assert_eq!(get(&input, "reads"), JsValue::from(2));
        assert_eq!(raw(&addon), get(&input, "first"));
        call(&addon, "dispose", &[]);
    }
}

#[wasm_bindgen_test(async)]
async fn raw_kitty_limit_disabled_never_converts() {
    for expression in [
        "Symbol('size')",
        "1000n",
        "({[Symbol.toPrimitive](){throw Error('disabled')}})",
    ] {
        let options = format!("({{sixelSupport:false,iipSupport:false,kittySupport:false,showPlaceholder:false,kittySizeLimit:{expression}}})");
        let preflight = ImageAddon::new(eval(&options)).unwrap();
        preflight.dispose().unwrap();
        let pair = Pair::new(false, &options);
        pair.write(kitty()).await;
        pair.compare();
        pair.addon.reset().unwrap();
        call(&get(&pair.source, "addon"), "reset", &[]);
        pair.addon.dispose().unwrap();
        call(&get(&pair.source, "addon"), "dispose", &[]);
        pair.compare();
    }
}

#[wasm_bindgen_test]
fn raw_kitty_limit_activation_converts_first_copy_once_per_activation() {
    let trial = Activation::new("(()=>{let reads=0;const first={log:[],[Symbol.toPrimitive](hint){this.log.push(hint);return '1000'}},second={[Symbol.toPrimitive](){throw Error('second copy')}};return {sixelSupport:false,iipSupport:false,showPlaceholder:false,get kittySizeLimit(){return ++reads===1?first:second}}})()");
    for count in 1..=2 {
        let (actual, expected) = trial.activate();
        actual.unwrap();
        expected.unwrap();
        trial.compare_membership();
        for rust in [false, true] {
            assert_eq!(
                get(&get(&trial.value(rust), "log"), "length"),
                JsValue::from(count)
            );
            assert_eq!(
                get(&get(&trial.value(rust), "log"), "0"),
                JsValue::from("number")
            );
        }
        trial.addon.reset().unwrap();
        call(&get(&trial.source, "addon"), "reset", &[]);
    }
}

#[wasm_bindgen_test]
fn raw_kitty_limit_activation_preserves_errors_and_prior_registrations() {
    for expression in [
        "Symbol('size')",
        "1000n",
        "({token:{},[Symbol.toPrimitive](){throw this.token}})",
        "({token:{},valueOf(){throw this.token}})",
        "({[Symbol.toPrimitive](){return {}}})",
    ] {
        // IIP is already registered when Kitty conversion throws.
        let trial = Activation::new(&format!(
            "({{sixelSupport:false,showPlaceholder:false,kittySizeLimit:{expression}}})"
        ));
        let (actual, expected) = trial.activate();
        let actual = actual.unwrap_err();
        let expected = expected.unwrap_err();
        if token(&trial.value(true)).is_undefined() {
            assert_eq!(get(&actual, "name"), get(&expected, "name"));
            assert_eq!(get(&actual, "message"), get(&expected, "message"));
        } else {
            assert_eq!(actual, token(&trial.value(true)));
            assert_eq!(expected, token(&trial.value(false)));
        }
        trial.compare_membership();
        assert_eq!(
            get(&trial.addon.handler_inspection(), "size"),
            JsValue::from(1)
        );
    }
}

#[wasm_bindgen_test]
fn raw_kitty_limit_activation_callbacks_can_reset_or_dispose() {
    for dispose in [false, true] {
        let trial = Rc::new(Activation::new("({sixelSupport:false,iipSupport:false,showPlaceholder:false,kittySizeLimit:{callback:null,log:[],[Symbol.toPrimitive](hint){this.log.push(hint);this.callback();return 1000}}})"));
        let weak = Rc::downgrade(&trial);
        let callback = Closure::<dyn Fn()>::new(move || {
            let trial = weak.upgrade().unwrap();
            if dispose {
                trial.addon.dispose().unwrap();
            } else {
                trial.addon.reset().unwrap();
            }
        });
        js_sys::Reflect::set(&trial.value(true), &"callback".into(), callback.as_ref()).unwrap();
        let source_callback = js_sys::Function::new_with_args(
            "a,dispose",
            "return ()=>dispose?a.dispose():a.reset()",
        )
        .call2(
            &JsValue::UNDEFINED,
            &get(&trial.source, "addon"),
            &dispose.into(),
        )
        .unwrap();
        js_sys::Reflect::set(&trial.value(false), &"callback".into(), &source_callback).unwrap();
        let (actual, expected) = trial.activate();
        actual.unwrap();
        expected.unwrap();
        trial.compare_membership();
        assert_eq!(json(&get(&trial.value(true), "log")), "[\"number\"]");
        assert_eq!(json(&get(&trial.value(false), "log")), "[\"number\"]");
        js_sys::Reflect::set(&trial.value(true), &"callback".into(), &JsValue::NULL).unwrap();
        js_sys::Reflect::set(&trial.value(false), &"callback".into(), &JsValue::NULL).unwrap();
    }
}

#[wasm_bindgen_test(async)]
async fn raw_kitty_limit_numeric_forms_enforce_real_stream_limit_and_pixels() {
    for expression in [
        "null",
        "false",
        "true",
        "-1",
        "0",
        "-0",
        "1.5",
        "'5'",
        "'6'",
        "'1000'",
        "[]",
        "[1000]",
        "new Number(1000)",
        "({valueOf(){return 1000}})",
    ] {
        let options = format!("({{sixelSupport:false,iipSupport:false,showPlaceholder:false,kittySizeLimit:{expression}}})");
        let preflight = ImageAddon::new(eval(&options)).unwrap();
        preflight.dispose().unwrap();
        let pair = Pair::new(true, &options);
        pair.write(kitty()).await;
        pair.compare();
        let inserted = matches!(
            expression,
            "'6'" | "'1000'" | "[1000]" | "new Number(1000)" | "({valueOf(){return 1000}})"
        );
        assert_eq!(
            get(&pair.events, "count"),
            JsValue::from(u32::from(inserted))
        );
        if inserted {
            pair.compare_image(0.0, 0.0);
        }
    }
}
