#![cfg(target_arch = "wasm32")]
#[path = "options_support/mod.rs"]
mod support;
use js_sys::{Array, Function, Reflect};
use support::{referenceOptions, run};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::headless::PublicOptions;
use xterm_wasm_support::options::{constructor_options, error_to_js, from_js, OptionsBinding};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn inherited_input_order_and_unknown_getter_skipping() {
    run(
        r#"
const trace=[];const proto={get rows(){trace.push('rows');return 40;}};
const input=Object.create(proto);
Object.defineProperty(input,'cursorWidth',{enumerable:true,get(){trace.push('width');return 2.9;}});
Object.defineProperty(input,'unknown',{enumerable:true,get(){throw Error('must not read unknown');}});
return {input,trace};
"#,
        "return [get('rows'),get('cursorWidth'),Object.keys(view).length];",
    );
}

#[wasm_bindgen_test]
fn inherited_object_prototype_names_are_real_constructor_keys() {
    run(
        "return {input:{toString:7,constructor:false,hasOwnProperty:'hello'},trace:[]};",
        r#"
set('toString',9);set('constructor',true);view.hasOwnProperty='changed';
return [get('toString'),view.toString,get('constructor'),view.hasOwnProperty,Object.keys(view).slice(-3)];
"#,
    );
}

#[wasm_bindgen_test]
fn dynamic_default_prototype_members_keep_full_string_key_events() {
    run(
        r#"
Object.defineProperty(Object.prototype,'__xtermEdge',{value:1,enumerable:true,writable:true,configurable:true});
return {input:{__xtermEdge:2},trace:[],cleanup(){delete Object.prototype.__xtermEdge;}};
"#,
        r#"
view.__xtermEdge=3;set('__xtermEdge',4);const value=get('__xtermEdge');
delete Object.prototype.__xtermEdge;let error;try{view.__xtermEdge=5;}catch(e){error=e.message;}
return [value,error,Object.keys(view).includes('__xtermEdge')];
"#,
    );
}

#[wasm_bindgen_test]
fn nested_default_bags_remain_live_without_root_assignment_events() {
    run(
        "return {input:{},trace:[]};",
        r#"
const bag=view.windowOptions;
try {
 bag.getWinSizePixels=true;const first=get('windowOptions').getWinSizePixels;
 bag.getWinSizePixels=false;const second=get('windowOptions').getWinSizePixels;
 return [bag===get('windowOptions'),first,second];
} finally {delete bag.getWinSizePixels;}
"#,
    );
}

#[wasm_bindgen_test]
fn custom_prototype_inherited_accessors_stay_non_enumerable() {
    run(
        r#"
const input={};Object.defineProperty(input,'__proto__',{enumerable:true,value:{toString:9,custom:33}});
return {input,trace:[]};
"#,
        r#"
let rejected;try{get('custom');}catch(e){rejected=e.message;}
return [get('toString'),typeof view.toString,view.custom,rejected,Object.keys(view).length];
"#,
    );
}

#[wasm_bindgen_test]
fn inherited_unknown_then_own_value_fails_public_spread() {
    run(
        r#"
const input={};Object.defineProperty(input,'__proto__',{enumerable:true,value:{custom:1}});input.custom=42;
return {input,trace:[]};
"#,
        "let error;try{get('custom');}catch(e){error=e.message;}return error;",
    );
}

#[wasm_bindgen_test]
fn null_and_primitive_prototype_assignment_change_membership() {
    for value in ["null", "7"] {
        run(
            &format!(
                r#"
const input={{}};Object.defineProperty(input,'__proto__',{{enumerable:true,value:{value}}});input.toString=7;
return {{input,trace:[]}};
"#
            ),
            "return [typeof view.toString,Object.keys(view).length];",
        );
    }
}

#[wasm_bindgen_test]
fn prototype_getter_setter_receiver_and_post_assignment_value() {
    run(
        r#"
const trace=[];let hidden=0;
const proto={get toString(){trace.push('get:'+this.rows);return this.rows+hidden;},set toString(v){trace.push('set:'+v);hidden=v;}};
const input={};Object.defineProperty(input,'__proto__',{enumerable:true,value:proto});input.toString=7;
return {input,trace};
"#,
        "set('toString',3);set('rows',30);return [get('toString'),typeof view.toString];",
    );
}

#[wasm_bindgen_test]
fn opaque_symbol_bigint_identity_truthiness_and_operators() {
    run(
        "return {input:{lineHeight:1n,fontFamily:Symbol.for('font')},trace:[]};",
        r#"
const results=[];for(const [key,value] of [['lineHeight',2n],['tabStopWidth',1n],['scrollSensitivity',1n],['wordSeparator',0n]]) {
 set(key,value);results.push([key,typeof get(key),String(get(key))]);
}
for(const key of ['minimumContrastRatio','cursorWidth']) {
 let kind;try{set(key,1n);}catch(error){kind=error.name;}results.push([key,kind]);
}
const font=get('fontFamily');set('fontFamily',font);results.push(['same',view.fontFamily===font]);return results;
"#,
    );
}

#[wasm_bindgen_test]
fn same_accessor_reentrancy_and_original_thrown_value_survive() {
    run(
        "return {input:{},trace:[],marker:{tag:'throw'}};",
        r#"
let entered=false;view.cursorWidth={valueOf(){context.trace.push('number');if(!entered){entered=true;view.cursorWidth=3;}return 4;}};
let caught;try{view.cursorWidth={valueOf(){throw context.marker;}};}catch(error){caught=error;}
return [get('cursorWidth'),caught===context.marker];
"#,
    );
}

#[wasm_bindgen_test]
fn constructor_membership_proxy_throw_retains_identity() {
    let factory = Function::new_no_args(
        r#"
const marker={tag:'membership'};const input={};
Object.defineProperty(input,'__proto__',{enumerable:true,value:new Proxy({}, {has(){throw marker;}})});input.custom=1;
return {input,marker};
"#,
    );
    let expected = factory.call0(&JsValue::UNDEFINED).unwrap();
    let actual = factory.call0(&JsValue::UNDEFINED).unwrap();
    let input = |ctx: &JsValue| Reflect::get(ctx, &"input".into()).unwrap();
    assert_eq!(
        referenceOptions(&input(&expected)).unwrap_err(),
        Reflect::get(&expected, &"marker".into()).unwrap()
    );
    let error = match constructor_options(&input(&actual), false) {
        Err(error) => error,
        Ok(_) => panic!("membership trap did not throw"),
    };
    assert_eq!(error, Reflect::get(&actual, &"marker".into()).unwrap());
}

#[wasm_bindgen_test]
fn default_hint_addition_and_concat_preserve_custom_primitive_order() {
    use xterm_core::options::OptionValue as V;
    let context=Function::new_no_args(r#"
const trace=[];const value={[Symbol.toPrimitive](hint){trace.push(hint);return hint==='number'?2:hint;}};
return {trace,value};
"#).call0(&JsValue::UNDEFINED).unwrap();
    let value = from_js(&Reflect::get(&context, &"value".into()).unwrap()).unwrap();
    assert_eq!(
        value.to_utf16().unwrap(),
        "string".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        value.concat_utf16().unwrap(),
        "default".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(value.to_number().unwrap(), 2.0);
    assert_eq!(V::Number(0.0).js_add(&value).unwrap(), V::from("0default"));
    assert_eq!(value.js_add(&V::Number(0.0)).unwrap(), V::from("default0"));
    let trace = Array::from(&Reflect::get(&context, &"trace".into()).unwrap())
        .iter()
        .map(|value| value.as_string().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(trace, ["string", "default", "number", "default", "default"]);
}

#[wasm_bindgen_test]
fn disposal_link_first_then_document_error_is_idempotent_and_lossless() {
    let factory = Function::new_no_args(
        r#"
const context={marker:{tag:'dispose'},trace:[],link:{},doc:{}};
const proto={set toString(value){context.raw=this;}};
const input={};Object.defineProperty(input,'__proto__',{value:proto,enumerable:true});input.toString=7;
context.input=input;return context;
"#,
    );
    let expected = factory.call0(&JsValue::UNDEFINED).unwrap();
    let actual = factory.call0(&JsValue::UNDEFINED).unwrap();
    let input = |context: &JsValue| Reflect::get(context, &"input".into()).unwrap();
    let reference = referenceOptions(&input(&expected)).unwrap();
    let service = constructor_options(&input(&actual), false).unwrap();
    let poison = Function::new_with_args(
        "context",
        r#"
Object.defineProperty(context.raw,'linkHandler',{configurable:true,get(){return context.link;},set(value){context.trace.push('link');context.link=value;}});
Object.defineProperty(context.raw,'documentOverride',{configurable:true,get(){return context.doc;},set(){context.trace.push('doc');throw context.marker;}});
"#,
    );
    poison.call1(&JsValue::UNDEFINED, &expected).unwrap();
    poison.call1(&JsValue::UNDEFINED, &actual).unwrap();
    let dispose = Reflect::get(&reference, &"dispose".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();
    assert_eq!(
        dispose.call0(&reference).unwrap_err(),
        Reflect::get(&expected, &"marker".into()).unwrap()
    );
    assert_eq!(
        error_to_js(&service.try_dispose().unwrap_err()),
        Reflect::get(&actual, &"marker".into()).unwrap()
    );
    dispose.call0(&reference).unwrap();
    service.try_dispose().unwrap();
    let check=Function::new_with_args("context","if(context.trace.join(',')!=='link,doc'||context.link!==null)throw Error('cleanup order or idempotence changed');");
    check.call1(&JsValue::UNDEFINED, &expected).unwrap();
    check.call1(&JsValue::UNDEFINED, &actual).unwrap();
}

#[wasm_bindgen_test]
fn shared_default_objects_survive_shallow_copies_without_user_value_leaks() {
    let first = OptionsBinding::new(PublicOptions::new(
        constructor_options(&JsValue::UNDEFINED, false).unwrap(),
    ))
    .unwrap();
    let second = OptionsBinding::new(PublicOptions::new(
        constructor_options(&JsValue::UNDEFINED, false).unwrap(),
    ))
    .unwrap();
    Function::new_with_args("first,second",r#"
if(first.theme!==second.theme||first.scrollbar!==second.scrollbar||first.windowsPty!==second.windowsPty)throw Error('default object identity changed across services');
first.theme.__testTag='shared';try{if(second.theme.__testTag!=='shared')throw Error('default mutation not shared');}finally{delete first.theme.__testTag;}
"#).call2(&JsValue::UNDEFINED,&first.object(),&second.object()).unwrap();
    first.service().try_dispose().unwrap();
    second.service().try_dispose().unwrap();
}

#[wasm_bindgen_test]
fn public_descriptors_readonly_partial_assign_utf16_and_cleanup() {
    run(
        "return {input:{documentOverride:{tag:'document'},linkHandler:{tag:'links'}},trace:[]};",
        r#"
const theme=view.theme;if(theme!==view.theme)throw Error('default object identity');
view.wordSeparator='\ud800';let error;try{view.cols=100;}catch(e){error=e.message;}
view.theme=theme;set('fontSize',NaN);set('fontSize',NaN);
let partial;try{assign({fontSize:20,rows:30,fontFamily:'must not apply'});}catch(e){partial=e.message;}
const inherited=Object.create({screenReaderMode:true});inherited.extra='expando';assign(inherited);
return [view.wordSeparator,error,view.cols,Object.getOwnPropertyDescriptor(view,'rows').configurable,view.fontSize,view.fontFamily,view.extra,view.screenReaderMode,partial];
"#,
    );
}
