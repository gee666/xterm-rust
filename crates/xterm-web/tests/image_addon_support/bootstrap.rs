// Copyright (c) 2020 The xterm.js authors. MIT license.
use super::{eval, promise};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

pub async fn public_bootstrap() {
    let module = JsFuture::from(js_sys::Promise::resolve(&eval("import('/suite.js')")))
        .await
        .unwrap();
    // Test the shipped bootstrap against this same WASM instance. Only module
    // directives change; class bodies execute byte-for-byte from production.
    let source = include_str!("../../../../browser/xterm.mjs")
        .replace(
            "import initialize, * as wasm from './pkg/xterm_web.js';",
            "",
        )
        .replace("export * from './pkg/xterm_web.js';", "")
        .replace("export { initialize };", "")
        .replace("export class ", "class ");
    let exports =
        js_sys::Function::new_with_args("wasm", &(source + "\nreturn {Terminal,ImageAddon};"))
            .call1(&JsValue::UNDEFINED, &module)
            .unwrap();
    let run = js_sys::Function::new_with_args(
        "api",
        r#"
      return (async()=>{
        const t=new api.Terminal({cols:20,rows:10,allowProposedApi:true});
        const p=document.body.appendChild(document.createElement('div'));
        t.open(p);
        const addon=new api.ImageAddon({showPlaceholder:false});
        let added=0; addon.onImageAdded(()=>added++);
        t.loadAddon(addon);
        const reference = new SourceImageAddon({sixelSupport:false,iipSupport:false,kittySupport:false});
        const referenceTerminal = new SourceTerminal(); reference.activate(referenceTerminal);
        let expected; try { reference.storageLimit=0; } catch(error) { expected=error; }
        let actual; try { addon.storageLimit=0; } catch(error) { actual=error; }
        if(!actual || actual.name!==expected.name || actual.message!==expected.message)throw Error('storageLimit error mismatch');
        reference.dispose(); referenceTerminal.dispose();
        await new Promise(resolve=>t.write('\x1b]1337;File=inline=1;width=2px;height=2px;preserveAspectRatio=0:cW9pZgAAAAIAAAACBAD/AQID//8BAgP//wECA///AQID/wAAAAAAAAAB\x07',resolve));
        if(added!==1 || addon.storageUsage!==0.000016)throw Error('public ImageAddon storage/event not connected');
        const image=addon.getImageAtBufferCell(0,0),tile=addon.extractTileAtBufferCell(0,0);
        if(image.width!==2 || image.height!==2 || !tile)throw Error('public image retrieval not connected');
        const rgba=[...image.getContext('2d').getImageData(0,0,1,1).data];
        if(JSON.stringify(rgba)!=='[1,2,3,255]')throw Error('public image pixels differ');
        t.dispose();p.remove();
        if(addon.storageUsage!==0)throw Error('terminal did not dispose managed addon');
      })();
    "#,
    );
    promise(run.call1(&JsValue::UNDEFINED, &exports).unwrap()).await;
}
