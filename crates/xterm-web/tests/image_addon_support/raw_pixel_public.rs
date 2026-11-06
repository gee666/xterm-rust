// Copyright (c) 2023, 2026 The xterm.js authors. MIT license.
use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn raw_pixel_direct_and_shipped_terminal_protocols_match_original_pixels_and_replies() {
    let direct = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&eval(
        "import('/suite.js')",
    )))
    .await
    .unwrap();
    let shipped = public_api().await;
    let probe = js_sys::Function::new_with_args(
        "apis,iip,kitty",
        r#"
      return (async()=>{
        async function trial(api,expression,text){
          const value=eval(expression),addon=new api.ImageAddon({sixelSupport:false,showPlaceholder:false,pixelLimit:value});
          let t,parent,runtime;
          try{
            if(!Object.is(addon._opts.pixelLimit,value))throw Error('raw public identity');
            t=new api.Terminal({cols:20,rows:10,fontSize:14,allowProposedApi:true,logLevel:'off'});
            if(api.BrowserRuntime){runtime=new api.BrowserRuntime();t._setRuntime(runtime);}
            parent=document.body.appendChild(document.createElement('div'));
            parent.style.cssText='position:fixed;top:0;left:0';
            t.open(parent);addon.activate(t);
            let count=0;const replies=[];
            addon.onImageAdded(()=>count++);t.onData(s=>replies.push(s));
            await new Promise(resolve=>t.write(text,resolve));
            const image=addon.getImageAtBufferCell(0,0);
            const pixels=image?[image.width,image.height,[...image.getContext('2d').getImageData(0,0,image.width,image.height).data]]:null;
            return JSON.stringify([count,replies,addon.storageUsage,pixels,value?.log]);
          }finally{
            addon.dispose();t?.dispose();parent?.remove();addon.free?.();t?.free?.();runtime?.free?.();
          }
        }
        for(const expression of ["1n","4n","5n","NaN","'1000'","({log:[],[Symbol.toPrimitive](h){this.log.push(h);return 1000}})"]){
          for(const text of [iip,kitty,'\x1b[?2;1S']){
            const expected=await trial(apis[0],expression,text);
            for(const api of apis.slice(1)){
              const actual=await trial(api,expression,text);
              if(actual!==expected)throw Error('raw public pixels/replies '+expression+' '+text+' '+actual+' != '+expected);
            }
          }
        }
      })();
    "#,
    );
    let result = probe
        .call3(
            &JsValue::UNDEFINED,
            &js_sys::Array::of3(
                &eval("({Terminal:SourceTerminal,ImageAddon:SourceImageAddon})"),
                &direct,
                &shipped,
            ),
            &iip().into(),
            &kitty().into(),
        )
        .unwrap();
    promise(result).await;
}
