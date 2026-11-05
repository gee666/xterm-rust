// Copyright (c) 2020 The xterm.js authors. MIT license.
use crate::support::{eval, get, public_api};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

async fn constructors() -> JsValue {
    let module = JsFuture::from(js_sys::Promise::resolve(&eval("import('/suite.js')")))
        .await
        .unwrap();
    js_sys::Array::of3(
        &eval("SourceImageAddon"),
        &get(&module, "ImageAddon"),
        &get(&public_api().await, "ImageAddon"),
    )
    .into()
}
fn compare(constructors: &JsValue, program: &str) {
    let run = js_sys::Function::new_with_args("constructors", program);
    let observations = run.call1(&JsValue::UNDEFINED, constructors).unwrap();
    let observations = js_sys::Array::from(&observations);
    let expected = js_sys::JSON::stringify(&observations.get(0)).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&observations.get(1)).unwrap(),
        expected,
        "direct WASM binding"
    );
    assert_eq!(
        js_sys::JSON::stringify(&observations.get(2)).unwrap(),
        expected,
        "shipped browser bootstrap"
    );
}

#[wasm_bindgen_test(async)]
async fn inactive_cell_methods_skip_all_raw_coordinate_conversions() {
    compare(
        &constructors().await,
        r#"
      return Array.from(constructors, Constructor => {
        const trace=[], sentinel={sentinel:true};
        const poison = {
          get [Symbol.toPrimitive]() { trace.push('toPrimitive'); throw sentinel; },
          valueOf() { trace.push('valueOf'); throw sentinel; },
          toString() { trace.push('toString'); throw sentinel; }
        };
        const revoked=Proxy.revocable({},{}); revoked.revoke();
        const values=[undefined,null,false,true,0,-0,NaN,Infinity,-Infinity,1,-1,
          '', '0', '00', 'not-a-number',0n,1n,Symbol('coordinate'),{},[],[0],
          new Number(0),new String('0'),()=>0,document.all,revoked.proxy,poison];
        const addon=new Constructor(), rows=[];
        try {
          for(const method of ['getImageAtBufferCell','extractTileAtBufferCell']) {
            for(let index=0; index<values.length; index++) {
              for(const args of [[values[index],poison],[poison,values[index]]]) {
                try { rows.push([method,index,addon[method](...args)===undefined]); }
                catch(error) { rows.push([method,index,false,error===sentinel,error?.name]); }
              }
            }
            rows.push([method,'missing',addon[method]()===undefined]);
          }
          return {rows,trace,usage:addon.storageUsage};
        } finally { addon.dispose(); }
      });
    "#,
    );
}

#[wasm_bindgen_test(async)]
async fn inactive_reset_and_disposal_never_run_coordinate_side_effects() {
    compare(
        &constructors().await,
        r#"
      return Array.from(constructors, Constructor => {
        const addon=new Constructor(), trace=[], rows=[];
        const coordinate={
          [Symbol.toPrimitive](hint) {
            trace.push(['convert',hint]); addon.showPlaceholder=false;
            addon.reset(); addon.dispose(); throw undefined;
          }
        };
        try {
          for(const phase of ['constructed','reset','disposed','disposedAgain']) {
            if(phase==='reset') addon.reset();
            if(phase==='disposed' || phase==='disposedAgain') addon.dispose();
            for(const method of ['getImageAtBufferCell','extractTileAtBufferCell']) {
              for(const args of [[coordinate,0],[0,coordinate],[coordinate,coordinate]]) {
                try { rows.push([phase,method,addon[method](...args)===undefined]); }
                catch(error) { rows.push([phase,method,false,error===undefined]); }
              }
            }
            rows.push([phase,addon.showPlaceholder,addon.storageUsage]);
          }
          return {rows,trace};
        } finally { addon.dispose(); }
      });
    "#,
    );
}
