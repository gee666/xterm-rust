// Copyright (c) 2020 The xterm.js authors. MIT license.
mod bootstrap;
mod pending;
pub use pending::{failed_disposal, late_completion};
#[path = "../image_terminal_support/snapshot.rs"]
mod snapshot;
pub use bootstrap::{public_api, public_bootstrap};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_core::{headless::HeadlessError, terminal::TerminalError};
use xterm_web::{image::ImageAddon, Terminal};
pub fn eval(code: &str) -> JsValue {
    js_sys::eval(code).unwrap()
}
pub fn get(value: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(value, &key.into()).unwrap()
}
pub fn call(value: &JsValue, key: &str, args: &[JsValue]) -> JsValue {
    get(value, key)
        .unchecked_into::<js_sys::Function>()
        .apply(value, &args.iter().cloned().collect::<js_sys::Array>())
        .unwrap()
}
pub async fn promise(value: JsValue) {
    JsFuture::from(value.unchecked_into::<js_sys::Promise>())
        .await
        .unwrap();
}
pub fn pixels(canvas: &JsValue) -> Vec<u8> {
    let context = call(canvas, "getContext", &["2d".into()]);
    let image = call(
        &context,
        "getImageData",
        &[
            0.into(),
            0.into(),
            get(canvas, "width"),
            get(canvas, "height"),
        ],
    );
    js_sys::Uint8Array::new(&get(&get(&image, "data"), "buffer")).to_vec()
}
pub struct Pair {
    pub terminal: Terminal,
    pub addon: ImageAddon,
    pub source: JsValue,
    pub events: JsValue,
    pub replies: Rc<RefCell<Vec<Vec<u16>>>>,
    parent: web_sys::Element,
}
impl Pair {
    pub fn new(open: bool, options: &str) -> Self {
        let terminal = Terminal::new(eval(
            "({cols:20,rows:10,fontSize:14,allowProposedApi:true,logLevel:'off'})",
        ))
        .unwrap();
        let doc = web_sys::window().unwrap().document().unwrap();
        let parent = doc.create_element("div").unwrap();
        // Harness output must not push just one terminal below the viewport.
        // Both renderers need the same visible state for image-cache assertions.
        parent
            .set_attribute("style", "position:fixed;top:0;left:0")
            .unwrap();
        doc.body().unwrap().append_child(&parent).unwrap();
        if open {
            terminal.open(parent.clone().into()).unwrap();
        }
        let addon = ImageAddon::new(eval(options)).unwrap();
        let events = eval("({count:0})");
        addon
            .on_image_added()
            .call1(
                &JsValue::UNDEFINED,
                &js_sys::Function::new_no_args("this.count++").bind0(&events),
            )
            .unwrap();
        let replies = Rc::new(RefCell::new(Vec::new()));
        let capture = replies.clone();
        terminal
            .shared_native()
            .on_data()
            .subscribe(move |text| capture.borrow_mut().push(text.clone()));
        addon.activate(&terminal).unwrap();
        let source = js_sys::Function::new_with_args("open,options", r#"
          const terminal = new SourceTerminal({cols:20,rows:10,fontSize:14,allowProposedApi:true,logLevel:'off'});
          const parent = document.body.appendChild(document.createElement('div'));
          parent.style.cssText='position:fixed;top:0;left:50%';
          if(open) terminal.open(parent);
          const addon = new SourceImageAddon(options), replies=[];
          let count=0;
          addon.onImageAdded(()=>count++);
          terminal.onData(data=>replies.push(data));
          addon.activate(terminal);
          return { terminal, addon, parent, replies, get count(){return count},
            write(text){return new Promise(resolve=>terminal.write(text,resolve));},
            async ready(){
              const handler = addon._handlers.get('sixel');
              for(let frame=0;handler && !handler._dec;frame++){
                if(frame===120)throw Error('SIXEL initialization did not finish');
                await new Promise(requestAnimationFrame);
              }
            },
            snapshot(){
              const b=terminal._core._bufferService.buffer, rows=[];
              for(let y=0;y<b.lines.length;y++){
                const line=b.lines.get(y),cells=[];
                for(let x=0;x<terminal.cols;x++){
                  const attrs=line.getExtended(x)?.payload;
                  cells.push([line.getString(x),line.getWidth(x),line.getFg(x),line.getBg(x),attrs?.imageId===undefined ? null : [attrs.imageId,attrs.tileId]]);
                }
                rows.push([line.isWrapped,cells]);
              }
              return [b.x,b.y,b.ybase,b.ydisp,rows];
            },
            dispose(){addon.dispose();terminal.dispose();parent.remove();}
          };
        "#).call2(&JsValue::UNDEFINED, &open.into(), &eval(options)).unwrap();
        Self {
            terminal,
            addon,
            source,
            events,
            replies,
            parent,
        }
    }
    pub async fn ready(&self) {
        promise(call(&self.source, "ready", &[])).await;
    }
    pub async fn write(&self, text: &str) {
        let native = self.terminal.shared_native();
        let mut result = native.write(text);
        while let Err(HeadlessError::Engine(TerminalError::Pending)) = result {
            let value = native
                .take_pending()
                .unwrap()
                .unwrap()
                .resolve()
                .await
                .unwrap();
            result = native.resume(value);
        }
        result.unwrap();
        promise(call(&self.source, "write", &[text.into()])).await;
        self.frame().await;
    }
    pub async fn frame(&self) {
        promise(eval("new Promise(requestAnimationFrame)")).await;
    }
    pub fn compare(&self) {
        let json = |v: &JsValue| js_sys::JSON::stringify(v).unwrap().as_string().unwrap();
        assert_eq!(
            json(&snapshot::snapshot(&self.terminal)),
            json(&call(&self.source, "snapshot", &[]))
        );
        let replies: js_sys::Array = self
            .replies
            .borrow()
            .iter()
            .map(|text| JsValue::from(js_sys::JsString::from_char_code(text)))
            .collect();
        assert_eq!(json(&replies), json(&get(&self.source, "replies")));
        assert_eq!(get(&self.events, "count"), get(&self.source, "count"));
        let source = get(&self.source, "addon");
        assert_eq!(
            self.addon.storage_usage().unwrap(),
            get(&source, "storageUsage").as_f64().unwrap()
        );
        assert_eq!(
            self.addon.storage_limit(),
            get(&source, "storageLimit").as_f64().unwrap()
        );
        assert_eq!(
            self.addon.show_placeholder(),
            get(&source, "showPlaceholder")
        );
    }
    pub fn compare_image(&self, x: f64, y: f64) {
        let rust = self.addon.get_image_at_buffer_cell(x, y).unwrap();
        let source = call(
            &get(&self.source, "addon"),
            "getImageAtBufferCell",
            &[x.into(), y.into()],
        );
        assert!(!rust.is_undefined());
        assert_eq!(pixels(&rust), pixels(&source));
        let rust = self.addon.extract_tile_at_buffer_cell(x, y).unwrap();
        let source = call(
            &get(&self.source, "addon"),
            "extractTileAtBufferCell",
            &[x.into(), y.into()],
        );
        assert_eq!(pixels(&rust), pixels(&source));
    }
}
impl Drop for Pair {
    fn drop(&mut self) {
        self.addon.dispose().unwrap();
        self.terminal.dispose().unwrap();
        self.parent.remove();
        call(&self.source, "dispose", &[]);
    }
}
pub fn sixel() -> &'static str {
    "\x1bP0;1q\"1;1;14;12#1;2;100;0;0!14~-#2;2;0;100;0!14~\x1b\\"
}
pub fn kitty() -> &'static str {
    "\x1b_Ga=T,f=32,s=1,v=1,i=42;AQID/w==\x1b\\"
}
pub fn iip() -> &'static str {
    "\x1b]1337;File=inline=1;size=42;width=2px;height=2px;preserveAspectRatio=0:cW9pZgAAAAIAAAACBAD/AQID//8BAgP//wECA///AQID/wAAAAAAAAAB\x07"
}
