#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{search_addon::SearchAddon, terminal::Terminal};
wasm_bindgen_test_configure!(run_in_browser);

async fn run(body: &str, opened: bool, initial: &str) {
    let terminal: JsValue = Terminal::new(js_sys::JSON::parse(initial).unwrap())
        .unwrap()
        .into();
    let addon: JsValue = SearchAddon::new(JsValue::UNDEFINED).unwrap().into();
    let setup = r#"
term.loadAddon=function(addon){return this._loadAddon(this,addon);};
const parent=document.createElement('div');parent.style.cssText='width:640px;height:240px;';document.body.appendChild(parent);
const style=document.createElement('style');style.textContent=css;document.head.appendChild(style);
const write=data=>new Promise(resolve=>term.write(data,resolve));
const frame=()=>new Promise(resolve=>requestAnimationFrame(resolve));
const equal=(a,b)=>{if(JSON.stringify(a)!==JSON.stringify(b))throw Error('actual '+JSON.stringify(a)+' expected '+JSON.stringify(b));};
const until=async predicate=>{for(let i=0;i<60;i++){if(predicate())return;await frame();}throw Error('Rust browser condition did not become true within 60 frames');};
term.loadAddon(addon);
return (async()=>{try{if(opened)term.open(parent);
"#;
    let script =
        format!("{setup}{body}\n}}finally{{term.dispose();parent.remove();style.remove();}}}})();");
    let promise = Function::new_with_args("term,addon,css,opened", &script)
        .call4(
            &JsValue::UNDEFINED,
            &terminal,
            &addon,
            &include_str!("../../../browser/xterm.css").into(),
            &opened.into(),
        )
        .unwrap()
        .dyn_into::<Promise>()
        .unwrap();
    JsFuture::from(promise).await.unwrap();
}
const DEFAULTS: &str = r#"{"cols":20,"rows":5,"scrollback":1000}"#;

#[wasm_bindgen_test]
async fn actual_shared_selection_and_search_events_through_public_binding() {
    run(r#"
let events=[];addon.onBeforeSearch(()=>events.push('before'));addon.onAfterSearch(()=>events.push('after'));
addon.onDidChangeResults(event=>events.push([event.resultIndex,event.resultCount]));
await write('hello hello');
const decorations={matchOverviewRuler:'#ff0000',activeMatchColorOverviewRuler:'#00ff00'};
equal(addon.findNext('hello',{decorations}),true);equal(term.getSelection(),'hello');
equal(term.getSelectionPosition(),{start:{x:0,y:0},end:{x:5,y:0}});equal(events,['before',[0,2],'after']);
events=[];equal(addon.findNext('hello',{decorations}),true);equal(term.getSelectionPosition(),{start:{x:6,y:0},end:{x:11,y:0}});equal(events,['before',[1,2],'after']);
equal(addon.findPrevious('hello',{decorations}),true);equal(term.getSelectionPosition(),{start:{x:0,y:0},end:{x:5,y:0}});
addon.clearDecorations();equal(term.getSelection(),'hello');equal(addon.findNext(''),false);equal(term.hasSelection(),false);
"#, true, DEFAULTS).await;
}

#[wasm_bindgen_test]
async fn actual_dom_decorations_receive_live_border_colors_and_source_classes() {
    run(r#"
await write('hello');
const decorations={matchBorder:'#ff0000',activeMatchBorder:'#ff0000',matchOverviewRuler:'#ff0000',activeMatchColorOverviewRuler:'#00ff00'};
addon.findNext('hello',{decorations});decorations.matchBorder='#0000ff';decorations.activeMatchBorder='#0000ff';
await until(()=>parent.querySelectorAll('.xterm-find-result-decoration').length===2);
for(const element of parent.querySelectorAll('.xterm-find-result-decoration')){
 if(element.classList.contains('xterm-find-active-result-decoration'))throw Error('source never passes active=true to style helper');
 equal(element.style.outlineColor,'rgb(0, 0, 255)');equal(element.style.outlineStyle,'solid');equal(element.style.outlineWidth,'1px');
}
addon.clearActiveDecoration();await until(()=>parent.querySelectorAll('.xterm-find-result-decoration').length===1);
addon.clearDecorations();await until(()=>parent.querySelectorAll('.xterm-find-result-decoration').length===0);
"#, true, DEFAULTS).await;
}

#[wasm_bindgen_test]
async fn raw_limit_values_use_source_comparison_and_slice_coercion_points() {
    run(r#"
await write('a a a');
const decorations={matchOverviewRuler:'#ff0000',activeMatchColorOverviewRuler:'#00ff00'};
for(const [limit,count,index] of [[0,0,-1],[-1,0,-1],[1.5,1,0],[NaN,0,-1],[Infinity,3,0]]){
 const child=new addon.constructor({highlightLimit:limit});term.loadAddon(child);let result;
 child.onDidChangeResults(value=>result=value);equal(child.findNext('a',{decorations}),true);equal(result,{resultIndex:index,resultCount:count});child.dispose();term.clearSelection();
}
let calls=0;const limit={valueOf(){calls++;return 1;}};const child=new addon.constructor({highlightLimit:limit});equal(calls,0);term.loadAddon(child);child.findNext('a',{decorations});equal(calls,3);child.dispose();
const symbolChild=new addon.constructor({highlightLimit:Symbol('limit')});term.loadAddon(symbolChild);let caught=false;
try{symbolChild.findNext('a',{decorations});}catch(error){if(!(error instanceof TypeError))throw error;caught=true;}if(!caught)throw Error('Symbol numeric exception lost');symbolChild.dispose();
"#, true, DEFAULTS).await;
}

#[wasm_bindgen_test]
async fn javascript_callback_exception_identity_and_option_mutation_are_preserved() {
    run(r#"
await write('foo FOO');const options={caseSensitive:false};
const edit=addon.onBeforeSearch(()=>options.caseSensitive=true);equal(addon.findNext('FOO',options),true);equal(term.getSelectionPosition().start.x,4);edit.dispose();
const original={searchCallback:true};const failure=addon.onBeforeSearch(()=>{throw original;});let caught=false;
try{addon.findNext('foo');}catch(error){if(error!==original)throw Error('before callback identity changed');caught=true;}if(!caught)throw Error('callback swallowed');failure.dispose();
const decorations={matchOverviewRuler:'#ff0000',activeMatchColorOverviewRuler:'#00ff00',get matchBackground(){throw original;}};
caught=false;try{addon.findNext('different',{decorations});}catch(error){if(error!==original)throw Error('color getter identity changed');caught=true;}
// No match means the background getter is correctly never read.
if(caught)throw Error('color getter read without a match');
try{addon.findNext('foo',{decorations});}catch(error){if(error!==original)throw Error('color getter identity changed');caught=true;}if(!caught)throw Error('color getter swallowed');
"#, true, DEFAULTS).await;
}

#[wasm_bindgen_test]
async fn before_open_matches_source_optional_selection_and_select_type_error() {
    run(r#"
equal(term.hasSelection(),false);equal(term.getSelection(),'');equal(term.getSelectionPosition(),undefined);
term.clearSelection();term.selectAll();term.selectLines(0,1);let caught=false;
try{term.select(0,0,1);}catch(error){if(!(error instanceof TypeError))throw error;caught=true;}if(!caught)throw Error('before-open select succeeded');
await write('hello');caught=false;try{addon.findNext('hello');}catch(error){if(!(error instanceof TypeError))throw error;caught=true;}if(!caught)throw Error('before-open addon selection succeeded');
term.open(parent);equal(addon.findNext('hello'),true);equal(term.getSelection(),'hello');
"#, false, DEFAULTS).await;
}

#[wasm_bindgen_test]
async fn observed_alternate_ring_recursion_is_a_javascript_range_error() {
    run(r#"
for(const text of ['\x1b[2J','\r\nnext','中','中','\x1b[2J','e\u0301','e\u0301','\r\nnext','\r\nline','\x1b[?1049hALT'])await write(text);
term.resize(9,1);await write('abc');let caught=false;
try{addon.findNext('abc');}catch(error){if(!(error instanceof RangeError)||error.message!=='Maximum call stack size exceeded')throw error;caught=true;}if(!caught)throw Error('source ring recursion did not return RangeError');
"#, true, r#"{"cols":9,"rows":1,"scrollback":5}"#).await;
}
