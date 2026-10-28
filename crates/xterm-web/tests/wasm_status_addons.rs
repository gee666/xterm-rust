#![cfg(target_arch = "wasm32")]
// Copyright (c) 2024-2026 The xterm.js authors. All rights reserved. MIT license.
// Assertion port of program-status and progress browser suites, using actual
// Rust Terminal/loadAddon/write bindings and the browser's JS event callbacks.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{
    status_addons::{ProgramStatusAddon, ProgressAddon},
    terminal::Terminal,
};
wasm_bindgen_test_configure!(run_in_browser);

async fn run(addon: JsValue, body: &str) {
    let term: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let setup = r#"
// Same one-line ABI bootstrap as browser/xterm.mjs. Raw wasm-bindgen instances
// cannot supply their own JS identity without that forwarding argument.
term.loadAddon=function(addon){return this._loadAddon(this,addon);};
const write=data=>new Promise(resolve=>term.write(data,resolve));
function canonical(v) { if(Array.isArray(v))return v.map(canonical); if(v&&typeof v==='object'){const o={};for(const k of Object.keys(v).sort())o[k]=canonical(v[k]);return o;} return v; }
function equal(actual,expected){if(JSON.stringify(canonical(actual))!==JSON.stringify(canonical(expected)))throw Error('actual '+JSON.stringify(actual)+' expected '+JSON.stringify(expected));}
return (async()=>{
const parent=document.createElement('div');parent.style.cssText='width:800px;height:450px';document.body.appendChild(parent);
try{term.open(parent);term.loadAddon(addon);
"#;
    let cleanup = "\n}finally{try{term.dispose();}finally{parent.remove();}}\n})();";
    let source = format!("{setup}{body}{cleanup}");
    let promise = Function::new_with_args("term,addon", &source)
        .call2(&JsValue::UNDEFINED, &term, &addon)
        .unwrap()
        .dyn_into::<Promise>()
        .unwrap();
    JsFuture::from(promise).await.unwrap();
}
#[wasm_bindgen_test]
async fn keeps_reported_status_without_printing() {
    run(ProgramStatusAddon::new().into(),r#"
let changes=[];addon.onChange(records=>changes.push(records));
await write('\x1b]7501;state=working:app=grok\x1b\\');
await write('\x1b]7501;state=blocked:kind=permission:id=deploy:msg=QXBwbHkgMyB0byBhZGQsIDEgdG8gY2hhbmdlLCAwIHRvIGRlc3Ryb3k/\x07');
equal(addon.records,[{id:'',state:'working',app:'grok',effectiveApp:'grok'},{id:'deploy',state:'blocked',kind:'permission',effectiveApp:'grok',message:'Apply 3 to add, 1 to change, 0 to destroy?'}]);
equal(changes.length,2);equal(term.buffer.active.getLine(0).translateToString(true),'');
if(changes[1]!==addon.records||!Object.isFrozen(addon.records)||!Object.isFrozen(addon.records[0]))throw Error('snapshot identity or freeze lost');
"#).await;
}
#[wasm_bindgen_test]
async fn answers_query_with_st_before_following_da1() {
    run(ProgramStatusAddon::new().into(),r#"let replies=[];term.onData(data=>replies.push(data));await write('\x1b]7501;?\x07\x1b[c');equal(replies,['\x1b]7501;?\x1b\\','\x1b[?1;2c']);"#).await;
}
#[wasm_bindgen_test]
async fn clears_every_record_on_ris() {
    run(ProgramStatusAddon::new().into(),r#"let changes=[];addon.onChange(records=>changes.push(records));await write('\x1b]7501;state=done:id=build\x1b\\\x1bc');equal(addon.records,[]);equal(changes.length,2);"#).await;
}
#[wasm_bindgen_test]
async fn prompt_removes_only_working_and_blocked() {
    run(ProgramStatusAddon::new().into(),r#"await write('\x1b]7501;state=working:id=a\x1b\\\x1b]7501;state=blocked:id=b\x1b\\\x1b]7501;state=done:id=c\x1b\\');addon.promptStarted();equal(addon.records.map(record=>record.id),['c']);"#).await;
}

#[wasm_bindgen_test]
async fn record_identity_freeze_and_callback_error_identity() {
    run(ProgramStatusAddon::new().into(),r#"
await write('\x1b]7501;state=working:id=plain\x07');const old=addon.records[0];
await write('\x1b]7501;state=done:id=other\x07');if(addon.records[0]!==old)throw Error('unchanged raw record identity lost');
await write('\x1b]7501;state=working:id=plain\x07');if(addon.records[1]===old)throw Error('replacement reused raw identity');
const thrown={statusCallback:true};addon.onChange(()=>{throw thrown;});let caught=false;
try{addon.clear();}catch(error){if(error!==thrown)throw Error('callback error identity lost');caught=true;}if(!caught)throw Error('callback error swallowed');
"#).await;
}

#[wasm_bindgen_test]
async fn reentrant_status_listeners_keep_outer_payload_and_current_snapshot_identity() {
    run(ProgramStatusAddon::new().into(),r#"
let outer,inner,seen=0;
addon.onChange(records=>{if(records.length){outer=records;addon.clear();}else{inner=records;}});
addon.onChange(records=>{if(records.length){if(records!==outer)throw Error('outer status payload identity lost');seen++;}else if(records!==inner){throw Error('inner payload identity lost');}});
await write('\x1b]7501;state=working\x07');
if(seen!==1||addon.records!==inner||addon.records.length!==0)throw Error('reentrant snapshot cache changed current records');
"#).await;
}

#[wasm_bindgen_test]
async fn public_addon_activation_inside_a_parser_callback_is_reentrant() {
    run(ProgramStatusAddon::new().into(),r#"
const child=new addon.constructor();
addon.onChange(()=>term.loadAddon(child));
await write('\x1b]7501;state=working\x07\x1b]7501;state=done:id=second\x07');
if(addon.records.length!==1||child.records.length!==1||child.records[0].id!=='second')throw Error('reentrant loadAddon failed');
"#).await;
}

#[wasm_bindgen_test]
async fn event_this_args_and_disposable_array_follow_source() {
    run(ProgramStatusAddon::new().into(),r#"
const receiver={marker:true},subscriptions=[];let count=0;
const token=addon.onChange(function(records){if(this!==receiver)throw Error('event thisArgs lost');count++;},receiver,subscriptions);
if(subscriptions[0]!==token||Object.getPrototypeOf(token)!==Object.prototype)throw Error('event disposable shape/array insertion changed');
await write('\x1b]7501;state=working\x07');if(count!==1)throw Error('listener missing');
const dispose=token.dispose;dispose();await write('\x1b]7501;state=done\x07');if(count!==1)throw Error('extracted disposer failed');
"#).await;
}
#[wasm_bindgen_test]
async fn event_store_failure_retains_listener_and_closed_registration_skips_store() {
    run(ProgramStatusAddon::new().into(),r#"
const thrown={registration:true};let saved,count=0;
const store={add(token){if(this!==store)throw Error('store this lost');saved=token;throw thrown;}};
try{addon.onChange(()=>count++,null,store);throw Error('missing store throw');}catch(error){if(error!==thrown)throw Error('store failure identity lost');}
await write('\x1b]7501;state=working\x07');if(count!==1)throw Error('failed store insertion rolled listener back');saved.dispose();
addon.dispose();const reject={get add(){throw Error('disposed event read store');}};
const a=addon.onChange(()=>{},null,reject),b=addon.onChange(()=>{},null,reject);
if(a===b||a.dispose===b.dispose||Object.isFrozen(a))throw Error('disposed registration used singleton/frozen token');a.dispose();b.dispose();
"#).await;
}

macro_rules! progress_case {
    ($name:ident,$body:expr) => {
        #[wasm_bindgen_test]
        async fn $name() {
            run(
                ProgressAddon::new().into(),
                concat!("let stack=[];addon.onChange(p=>stack.push(p));", $body),
            )
            .await;
        }
    };
}
progress_case!(
    initial_progress_zero_zero,
    "equal(addon.progress,{state:0,value:0});"
);
progress_case!(remove_ignores_progress,"await write('\x1b]9;4;0\x1b\\');equal(stack,[{state:0,value:0}]);await write('\x1b]9;4;0;12\x1b\\');equal(stack,[{state:0,value:0},{state:0,value:0}]);");
progress_case!(set_progress_values,"await write('\x1b]9;4;1;10\x1b\\');equal(stack,[{state:1,value:10}]);await write('\x1b]9;4;1;50\x1b\\');equal(stack,[{state:1,value:10},{state:1,value:50}]);await write('\x1b]9;4;1;23\x1b\\');equal(stack,[{state:1,value:10},{state:1,value:50},{state:1,value:23}]);");
progress_case!(set_special_sequences,"await write('\x1b]9;4;1\x1b\\');equal(stack,[{state:1,value:0}]);await write('\x1b]9;4;1;12x\x1b\\');equal(stack,[{state:1,value:0}]);await write('\x1b]9;4;1;123\x1b\\');equal(stack,[{state:1,value:0},{state:1,value:100}]);");
progress_case!(error_preserves_empty_zero,"await write('\x1b]9;4;1;12\x1b\\');await write('\x1b]9;4;2\x1b\\');await write('\x1b]9;4;2;\x1b\\');await write('\x1b]9;4;2;0\x1b\\');equal(stack,[{state:1,value:12},{state:2,value:12},{state:2,value:12},{state:2,value:12}]);");
progress_case!(error_accepts_new_progress,"await write('\x1b]9;4;1;12\x1b\\');await write('\x1b]9;4;2;25\x1b\\');await write('\x1b]9;4;2;123\x1b\\');equal(stack,[{state:1,value:12},{state:2,value:25},{state:2,value:100}]);");
progress_case!(indeterminate_keeps_progress,"await write('\x1b]9;4;1;12\x1b\\');await write('\x1b]9;4;3\x1b\\');await write('\x1b]9;4;3;123\x1b\\');equal(stack,[{state:1,value:12},{state:3,value:12},{state:3,value:12}]);");
progress_case!(pause_preserves_empty_zero,"await write('\x1b]9;4;1;12\x1b\\');await write('\x1b]9;4;4\x1b\\');await write('\x1b]9;4;4;\x1b\\');await write('\x1b]9;4;4;0\x1b\\');equal(stack,[{state:1,value:12},{state:4,value:12},{state:4,value:12},{state:4,value:12}]);");
progress_case!(pause_accepts_new_progress,"await write('\x1b]9;4;1;12\x1b\\');await write('\x1b]9;4;4;25\x1b\\');await write('\x1b]9;4;4;123\x1b\\');equal(stack,[{state:1,value:12},{state:4,value:25},{state:4,value:100}]);");
progress_case!(
    invalid_sequences_do_not_emit,
    "await write('\x1b]9;4;5;12\x1b\\');await write('\x1b]9;4;1; 123xxxx\x1b\\');equal(stack,[]);"
);
