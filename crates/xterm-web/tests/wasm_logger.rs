#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_core::{
    log_service::{LogLevel, LogParameter, LogService, LogValue},
    options::OptionsService,
};
use xterm_web::terminal::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

fn run(initial: &str, body: &str) {
    let options = Function::new_no_args(initial)
        .call0(&JsValue::UNDEFINED)
        .unwrap();
    let terminal: JsValue = Terminal::new(options).unwrap().into();
    let code = format!("try {{ {body} }} finally {{ term.dispose(); }}");
    Function::new_with_args("term", &code)
        .call1(&JsValue::UNDEFINED, &terminal)
        .unwrap();
}
const LOGGER: &str = r#"
const calls=[];const logger={calls};
for(const name of ['trace','debug','info','warn','error']) logger[name]=function(message,...params){calls.push({name,message,params,thisValue:this});};
return {logger,logLevel:'trace'};
"#;

#[wasm_bindgen_test]
fn constructor_logger_retains_identity_this_utf16_and_trace_units() {
    run(
        LOGGER,
        r#"
const logger=term.options.logger;
if(logger!==term.options.logger)throw Error('logger identity lost');
const input='\0\ud800\ud83d\ude00';term.input(input,false);
const calls=logger.calls;
if(calls.length!==2||calls[0].name!=='debug'||calls[1].name!=='trace')throw Error('level/order mismatch');
if(calls.some(call=>call.thisValue!==logger))throw Error('logger this lost');
if(calls[0].message!=='sending data "'+input+'"')throw Error('UTF16 message or prefix mismatch');
const actual=calls[1].params[0];
if(actual.length!==input.length||actual.some((v,i)=>v!==input.charCodeAt(i)))throw Error('trace code units mismatch');
term.options.logLevel='off';term.input('filtered',false);
if(calls.length!==2)throw Error('off still logged');
term.options.logLevel='trace';term.options.logger=logger;
if(term.options.logger!==logger)throw Error('set/get identity lost');
term.input('',false);if(calls.length!==4)throw Error('empty input not logged');
"#,
    );
}

#[wasm_bindgen_test]
fn getters_bind_once_and_preserve_thrown_js_value_identity() {
    run(
        "return {logLevel:'trace'};",
        r#"
const marker={identity:'getter'};let calls=0;
term.options.logger={get debug(){calls++;throw marker;}};
let caught;try{term.input('x',false);}catch(error){caught=error;}
if(caught!==marker||calls!==1)throw Error('getter exception identity/call count lost');
const callMarker={identity:'call'};
term.options.logger={debug(){throw callMarker;}};
caught=undefined;try{term.input('x',false);}catch(error){caught=error;}
if(caught!==callMarker)throw Error('method exception identity lost');
term.options.logLevel='off';calls=0;term.options.logger={get debug(){calls++;throw marker;}};
term.input('filtered',false);if(calls!==0)throw Error('filtered method getter accessed');
"#,
    );
}

#[wasm_bindgen_test]
fn method_getter_replacement_changes_the_bind_receiver_before_lazy_values() {
    run(
        "return {logLevel:'debug'};",
        r#"
let calls=0;
const replacement={debug(){throw Error('wrong target method selected');}};
term.options.logger={get debug(){
 calls++;term.options.logger=replacement;
 return function(message){'use strict';if(this!==replacement)throw Error('live receiver lost');if(message!=='sending data "x"')throw Error('message prefix changed');};
}};
term.input('x',false);
if(calls!==1||term.options.logger!==replacement)throw Error('getter replacement failed');
"#,
    );
}

#[wasm_bindgen_test]
fn default_console_routes_prefixes_and_live_replacement() {
    run(
        "return {logLevel:'trace'};",
        r#"
const calls=[];const original=console.log;
console.log=function(...args){calls.push({args,thisValue:this});};
try {
 term.input('\ud800',false);
 if(calls.length!==2)throw Error('default trace/debug console route missing');
 if(calls.some(call=>call.thisValue!==console))throw Error('console this lost');
 if(calls[0].args[0]!=='xterm.js: sending data "\ud800"')throw Error('default prefix/UTF16 lost');
 const custom={debug(){calls.push({custom:true});},trace(){calls.push({custom:true});}};
 term.options.logger=custom;term.input('x',false);
 if(calls.length!==4||!calls[2].custom||!calls[3].custom)throw Error('replacement not used');
 term.options.logger=null;term.input('x',false);
 if(calls.length!==6||!calls[4].args[0].startsWith('xterm.js: '))throw Error('null fallback not restored');
} finally {console.log=original;}
"#,
    );
}

#[wasm_bindgen_test]
fn logger_can_read_state_and_reenter_input_with_source_ordering() {
    run(
        "return {logLevel:'trace'};",
        r#"
const calls=[];let first=true;
const subscription=term.onData(value=>calls.push('data:'+value));
term.options.logger={
 debug(message){calls.push('debug:'+message);if(term.cols!==80)throw Error('state getter failed');if(first){first=false;term.input('inner',false);}},
 trace(message){calls.push('trace:'+message);}
};
term.input('outer',false);
const expected=['debug:sending data "outer"','debug:sending data "inner"','trace:sending data (codes)','data:inner','trace:sending data (codes)','data:outer'];
if(JSON.stringify(calls)!==JSON.stringify(expected))throw Error('reentrant logger order '+JSON.stringify(calls));
subscription.dispose();
"#,
    );
}

#[wasm_bindgen_test]
fn console_callable_failure_is_after_lazy_but_custom_bind_failure_is_before() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let logger = LogService::new(Rc::clone(&options), xterm_web::logger::console_sink()).unwrap();
    let console = Reflect::get(&js_sys::global(), &"console".into()).unwrap();
    let original = Reflect::get(&console, &"log".into()).unwrap();
    Reflect::set(&console, &"log".into(), &JsValue::NULL).unwrap();
    let evaluated = Cell::new(false);
    let default_error = logger.log(
        LogLevel::Debug,
        &[],
        vec![LogParameter::Lazy(Box::new(|| {
            evaluated.set(true);
            LogValue::CodeUnits(vec![])
        }))],
    );
    Reflect::set(&console, &"log".into(), &original).unwrap();
    assert!(
        xterm_web::js_error::to_js_error(&default_error.unwrap_err())
            .is_instance_of::<js_sys::TypeError>()
    );
    assert!(
        evaluated.get(),
        "console .call validation follows lazy parameters"
    );
    let invalid = Function::new_no_args("return {debug:1};")
        .call0(&JsValue::UNDEFINED)
        .unwrap();
    options
        .set("logger", xterm_web::logger::from_js(&invalid).unwrap())
        .unwrap();
    evaluated.set(false);
    let custom_error = logger.log(
        LogLevel::Debug,
        &[],
        vec![LogParameter::Lazy(Box::new(|| {
            evaluated.set(true);
            LogValue::CodeUnits(vec![])
        }))],
    );
    assert!(xterm_web::js_error::to_js_error(&custom_error.unwrap_err())
        .is_instance_of::<js_sys::TypeError>());
    assert!(
        !evaluated.get(),
        "custom .bind validation precedes lazy parameters"
    );
}

#[wasm_bindgen_test]
fn invalid_primitive_logger_defers_typeerror_until_admitted_message() {
    run(
        "return {logLevel:'off',logger:false};",
        r#"
if(term.options.logger!==false)throw Error('false logger value lost');
term.input('filtered',false);
term.options.logLevel='trace';let caught;
try{term.input('x',false);}catch(error){caught=error;}
if(!(caught instanceof TypeError))throw Error('invalid logger must throw native TypeError');
term.options.logger=undefined;if(term.options.logger!==undefined)throw Error('undefined logger value lost');
"#,
    );
}
