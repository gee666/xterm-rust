// Copyright (c) 2023 The xterm.js authors. MIT license.
mod patch;
pub use patch::{thrower, try_call, Patch};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::image::storage::{
    ImageStorage, ImageStorageOptions, ImageTileInfo, TerminalStorageHost,
};
use xterm_core::{headless::HeadlessError, lifecycle::DisposableHandle, terminal::TerminalError};
use xterm_web::{
    image::{BrowserImageRenderer, BrowserImageSurface, ImageRendererLifecycle},
    Terminal,
};

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
    let data = call(
        &context,
        "getImageData",
        &[
            0.into(),
            0.into(),
            get(canvas, "width"),
            get(canvas, "height"),
        ],
    );
    js_sys::Uint8Array::new(&get(&get(&data, "data"), "buffer")).to_vec()
}
pub struct Live<P> {
    pub terminal: Terminal,
    pub renderer: Rc<BrowserImageRenderer>,
    pub storage: ImageStorage,
    pub protocol: P,
    pub registration: DisposableHandle,
    pub added: Rc<Cell<usize>>,
    pub replies: Rc<RefCell<Vec<Vec<u16>>>>,
    owner: ImageRendererLifecycle,
    parent: web_sys::Element,
}
impl<P> Live<P> {
    pub fn with_protocol(
        open: bool,
        build: impl FnOnce(&Terminal, Rc<BrowserImageRenderer>, ImageStorage) -> (P, DisposableHandle),
    ) -> Self {
        let terminal = Terminal::new(eval(
            "({cols:20,rows:10,fontSize:14,allowProposedApi:true})",
        ))
        .unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        let parent = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        if open {
            terminal.open(parent.clone().into()).unwrap();
        }
        let owner = ImageRendererLifecycle::new(&terminal).unwrap();
        let renderer = owner.renderer();
        let native = terminal.shared_native();
        let host = TerminalStorageHost::from_terminal(&native, Rc::new(|_, _| Ok(()))).unwrap();
        let storage = ImageStorage::new(
            Rc::new(host),
            renderer.clone(),
            ImageStorageOptions {
                show_placeholder: Rc::new(Cell::new(false)),
                ..Default::default()
            },
        )
        .unwrap();
        let added = Rc::new(Cell::new(0));
        let count = added.clone();
        storage.set_on_image_added(Some(Rc::new(move || {
            count.set(count.get() + 1);
            Ok(())
        })));
        let replies = Rc::new(RefCell::new(Vec::new()));
        let capture = replies.clone();
        native
            .on_data()
            .subscribe(move |text| capture.borrow_mut().push(text.clone()));
        let (protocol, registration) = build(&terminal, renderer.clone(), storage.clone());
        Self {
            terminal,
            renderer,
            storage,
            protocol,
            registration,
            added,
            replies,
            owner,
            parent,
        }
    }
    pub fn begin(&self, text: &str) -> bool {
        match self.terminal.shared_native().write(text) {
            Ok(()) => false,
            Err(HeadlessError::Engine(TerminalError::Pending)) => true,
            Err(error) => panic!("unexpected parse error: {error}"),
        }
    }
    pub async fn finish(&self, mut pending: bool) {
        let native = self.terminal.shared_native();
        while pending {
            let result = native
                .take_pending()
                .unwrap()
                .expect("pending handler")
                .resolve()
                .await
                .unwrap();
            pending = match native.resume(result) {
                Ok(()) => false,
                Err(HeadlessError::Engine(TerminalError::Pending)) => true,
                Err(error) => panic!("unexpected continuation error: {error}"),
            };
        }
    }
    pub async fn write(&self, text: &str) {
        self.finish(self.begin(text)).await;
    }
    pub fn image(&self, x: usize, y: usize) -> JsValue {
        self.storage
            .get_image_at_buffer_cell(x, y)
            .unwrap()
            .unwrap()
            .surface::<BrowserImageSurface>()
            .unwrap()
            .value()
    }
    pub fn snapshot(&self) -> JsValue {
        self.terminal
            .shared_native()
            .with_state(|state| {
                let buffer = state.active_buffer();
                let buffer = buffer.borrow();
                let rows = js_sys::Array::new();
                for y in 0..buffer.lines.len() {
                    let line = buffer.lines.get(y).unwrap();
                    let line = line.borrow();
                    let cells = js_sys::Array::new();
                    for x in 0..20 {
                        let image = line
                            .get_extended(x)
                            .and_then(|e| e.payload())
                            .and_then(|p| p.downcast::<ImageTileInfo>().ok());
                        cells.push(&js_sys::Array::of5(
                            &js_sys::JsString::from_char_code(&line.get_string(x).unwrap()).into(),
                            &line.get_width(x).unwrap().into(),
                            &line.get_fg(x).unwrap().into(),
                            &line.get_bg(x).unwrap().into(),
                            &image.map_or(JsValue::NULL, |p| {
                                js_sys::Array::of2(
                                    &(p.image_id.get() as f64).into(),
                                    &p.tile_id.get().into(),
                                )
                                .into()
                            }),
                        ));
                    }
                    rows.push(&js_sys::Array::of2(&line.is_wrapped.into(), &cells));
                }
                js_sys::Array::of5(
                    &(buffer.x as f64).into(),
                    &(buffer.y as f64).into(),
                    &(buffer.ybase as f64).into(),
                    &(buffer.ydisp as f64).into(),
                    &rows,
                )
                .into()
            })
            .unwrap()
    }
}
impl<P> Drop for Live<P> {
    fn drop(&mut self) {
        self.registration.dispose();
        self.storage.dispose().unwrap();
        self.owner.dispose().unwrap();
        self.terminal.dispose().unwrap();
        self.parent.remove();
    }
}
pub struct Source(pub JsValue);
impl Source {
    pub fn with_protocol(open: bool, options: &str, setup: &str) -> Self {
        Self(js_sys::Function::new_with_args("open,options,setup", r#"
        const t=new SourceTerminal({cols:20,rows:10,fontSize:14,allowProposedApi:true});
        const parent=document.createElement('div'); document.body.appendChild(parent);
        if(open)t.open(parent);
        const opts=Object.assign({storageLimit:128,showPlaceholder:false,pixelLimit:16777216},eval(options));
        const renderer=new SourceImageRenderer(t), store=new SourceImageStorage(t,renderer,opts);
        const {h,registration,ready}=new Function('t','renderer','store','opts',setup)(t,renderer,store,opts);
        let added=0, pending; const replies=[];
        store.onImageAdded=()=>++added; t.onData(data=>replies.push(data));
        return {t,renderer,store,h,opts,ready,replies, get added(){return added},
          registration,
          begin(text){pending=t._core._inputHandler.parse(text);return !!pending},
          async finish(){while(pending){pending=t._core._inputHandler.parse('',await pending)}},
          reset(){h.reset()},
          snapshot(){const b=t._core._bufferService.buffer;return [b.x,b.y,b.ybase,b.ydisp,
            Array.from({length:b.lines.length},(_,y)=>{const l=b.lines.get(y);return [l.isWrapped,
              Array.from({length:20},(_,x)=>{const e=l.getExtended(x)?.payload;return [l.getString(x),l.getWidth(x),l.getFg(x),l.getBg(x),
                e?.imageId===undefined?null:[e.imageId,e.tileId]]})]})]},
          image(x,y){return store.getImageAtBufferCell(x,y)},
          dispose(){registration.dispose();store.dispose();renderer.dispose();t.dispose();parent.remove()}
        };"#).call3(&JsValue::UNDEFINED, &open.into(), &options.into(), &setup.into()).unwrap())
    }
    pub fn begin(&self, text: &str) -> bool {
        call(&self.0, "begin", &[text.into()]).as_bool().unwrap()
    }
    pub async fn finish(&self) {
        promise(call(&self.0, "finish", &[])).await;
    }
    pub async fn write(&self, text: &str) {
        self.begin(text);
        self.finish().await;
    }
    pub fn image(&self, x: u32, y: u32) -> JsValue {
        call(&self.0, "image", &[x.into(), y.into()])
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        call(&self.0, "dispose", &[]);
    }
}
pub fn same_state<P>(live: &Live<P>, source: &Source) {
    let replies = js_sys::Array::from(&get(&source.0, "replies"));
    assert_eq!(live.replies.borrow().len(), replies.length() as usize);
    for (index, text) in live.replies.borrow().iter().enumerate() {
        assert_eq!(
            String::from_utf16(text).unwrap(),
            replies.get(index as u32).as_string().unwrap()
        );
    }
    let json = |v: &JsValue| js_sys::JSON::stringify(v).unwrap().as_string().unwrap();
    let rust = js_sys::Array::from(&live.snapshot());
    let original = js_sys::Array::from(&call(&source.0, "snapshot", &[]));
    for index in 0..4 {
        assert_eq!(
            rust.get(index),
            original.get(index),
            "cursor/scroll field {index}"
        );
    }
    let rows = js_sys::Array::from(&rust.get(4));
    let source_rows = js_sys::Array::from(&original.get(4));
    assert_eq!(rows.length(), source_rows.length());
    for row in 0..rows.length() {
        let r = js_sys::Array::from(&rows.get(row));
        let s = js_sys::Array::from(&source_rows.get(row));
        assert_eq!(r.get(0), s.get(0), "wrapped row{row}");
        let cells = js_sys::Array::from(&r.get(1));
        let source_cells = js_sys::Array::from(&s.get(1));
        assert_eq!(cells.length(), source_cells.length());
        for col in 0..cells.length() {
            assert_eq!(
                json(&cells.get(col)),
                json(&source_cells.get(col)),
                "cell {col},{row}"
            );
        }
    }
    assert_eq!(
        live.added.get() as f64,
        get(&source.0, "added").as_f64().unwrap()
    );
    assert_eq!(
        live.storage.get_usage().unwrap(),
        call(&get(&source.0, "store"), "getUsage", &[])
            .as_f64()
            .unwrap()
    );
}
