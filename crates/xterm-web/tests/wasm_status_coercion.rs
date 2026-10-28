#![cfg(target_arch = "wasm32")]
//! Dynamic JS calls outside the typings still follow source getter/coercion and
//! partial-assignment semantics. The terminal and addon state remain Rust-owned.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{status_addons::ProgressAddon, terminal::Terminal};
wasm_bindgen_test_configure!(run_in_browser);
async fn run(body: &str) {
    let term: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = ProgressAddon::new().into();
    let setup="function assert(value,message){if(!value)throw Error(message);}return(async()=>{const parent=document.createElement('div');parent.style.cssText='width:800px;height:450px';document.body.appendChild(parent);try{term.open(parent);term._loadAddon(term,addon);";
    let cleanup = "}finally{try{term.dispose();}finally{parent.remove();}}})();";
    let source = format!("{setup}{body}{cleanup}");
    let promise = Function::new_with_args("term,addon", &source)
        .call2(&JsValue::UNDEFINED, &term, &addon)
        .unwrap()
        .dyn_into::<Promise>()
        .unwrap();
    JsFuture::from(promise).await.unwrap();
}
#[wasm_bindgen_test]
async fn source_reads_state_three_times_before_value_and_keeps_partial_assignment() {
    run(r#"
addon.progress={state:1,value:40};let events=0;addon.onChange(()=>events++);
let reads=[],n=0;const thrown={valueGetter:true};
try{addon.progress={get state(){reads.push('state');return ++n===3?7:1;},get value(){reads.push('value');throw thrown;}};throw Error('missing throw');}
catch(error){assert(error===thrown,'getter exception identity lost');}
assert(reads.join(',')==='state,state,state,value','property read order changed');
assert(addon.progress.state===7&&addon.progress.value===40,'partial state/value changed');assert(events===0,'failed setter emitted');
"#).await;
}
#[wasm_bindgen_test]
async fn invalid_state_short_circuits_without_value_read() {
    run(r#"
addon.progress={state:1,value:40};let reads=[];
addon.progress={get state(){reads.push('state');return -1;},get value(){throw Error('value must not be read');}};
assert(reads.length===1,'invalid state read twice');assert(addon.progress.state===1&&addon.progress.value===40,'invalid state mutated');
"#).await;
}
#[wasm_bindgen_test]
async fn source_coerces_values_but_retains_raw_state_types() {
    run(r#"
addon.progress={state:'2',value:'75'};assert(addon.progress.state==='2'&&addon.progress.value===75,'string state/value lost');
addon.progress={state:null,value:null};assert(addon.progress.state===null&&addon.progress.value===0,'null changed');
addon.progress={state:3n,value:true};assert(addon.progress.state===3n&&addon.progress.value===1,'BigInt state comparison changed');
addon.progress=1;assert(Object.hasOwn(addon.progress,'state')&&addon.progress.state===undefined&&Number.isNaN(addon.progress.value),'primitive property boxing changed');
const state={valueOf(){return 2;}};addon.progress={state,value:5};assert(addon.progress.state===state,'state object identity lost');
"#).await;
}
#[wasm_bindgen_test]
async fn bigint_value_throws_after_state_assignment_without_emitting() {
    run(r#"
addon.progress={state:1,value:40};let events=0;addon.onChange(()=>events++);let caught=false;
try{addon.progress={state:2,value:1n};}catch(error){assert(error instanceof TypeError,'wrong BigInt error type');caught=true;}
assert(caught,'BigInt value accepted');assert(addon.progress.state===2&&addon.progress.value===40,'BigInt partial update lost');assert(events===0,'failed coercion emitted');
"#).await;
}
#[wasm_bindgen_test]
async fn reentrant_value_getter_retains_inner_state_and_outer_value() {
    run(r#"
addon.progress={state:1,get value(){addon.progress={state:2,value:10};return 25;}};
assert(addon.progress.state===2&&addon.progress.value===25,'reentrant _st/_pr ordering changed');
"#)
    .await;
}
#[wasm_bindgen_test]
async fn progress_payload_is_shared_mutable_and_reentrancy_safe() {
    run(r#"
let first;let seen=0;
addon.onChange(p=>{if(p.state===1){first=p;p.value=71;addon.progress={state:2,value:10};}});
addon.onChange(p=>{if(p===first){assert(p.state===1&&p.value===71,'outer payload mutation lost');seen++;}});
addon.progress={state:1,value:20};assert(seen===1,'listeners received separate payload objects');
assert(addon.progress.state===2&&addon.progress.value===10,'payload mutation modified actual state');
"#).await;
}
#[wasm_bindgen_test]
async fn api_setter_and_protocol_zero_have_distinct_source_semantics() {
    run(r#"
addon.progress={state:0,value:15};assert(addon.progress.value===15,'API state0 incorrectly discarded value');
addon.progress={state:3,value:8};assert(addon.progress.value===8,'API indeterminate incorrectly preserved previous value');
addon.progress={state:1,value:-0};assert(Object.is(addon.progress.value,0),'API clamp retained negative zero');
await new Promise(resolve=>term.write('\x1b]9;4;0;90\x07',resolve));assert(addon.progress.state===0&&addon.progress.value===0,'protocol remove did not reset value');
"#).await;
}
