#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlIFrameElement, Window};
use xterm_core::{buffer::BufferService, headless::Terminal};
use xterm_wasm_support::js_error::to_js_error;
use xterm_web::buffer_cleanup;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function observeCleanup(window,pending) {
  let resolve, reject;
  const promise=new Promise((yes,no)=>{resolve=yes;reject=no;});
  const records={requested:0,fired:0,cancelled:0};
  const saved=new Map();
  for(const name of ['setTimeout','requestIdleCallback','clearTimeout','cancelIdleCallback']) {
    const original=window[name];
    if(typeof original!=='function') continue;
    saved.set(name,Object.getOwnPropertyDescriptor(window,name));
    const cancel=name==='clearTimeout'||name==='cancelIdleCallback';
    Object.defineProperty(window,name,{configurable:true,writable:true,value:function(callback,...args) {
      if(cancel) {records.cancelled++;return Reflect.apply(original,this,[callback,...args]);}
      records.requested++;
      return Reflect.apply(original,this,[function(...values) {
        records.fired++;
        try {const result=Reflect.apply(callback,this,values);if(!pending()) resolve();return result;}
        catch(error) {reject(error);}
      },...args]);
    }});
  }
  return {promise,records,restore() {
    for(const [name,descriptor] of saved) {
      if(descriptor) Object.defineProperty(window,name,descriptor);else delete window[name];
    }
  }};
}
")]
extern "C" {
    #[wasm_bindgen(js_name = observeCleanup)]
    fn observe_cleanup(window: &Window, pending: &Function) -> JsValue;
}
fn frame() -> (HtmlIFrameElement, Window) {
    let document = web_sys::window().unwrap().document().unwrap();
    let frame: HtmlIFrameElement = document
        .create_element("iframe")
        .unwrap()
        .dyn_into()
        .unwrap();
    document.body().unwrap().append_child(&frame).unwrap();
    let window = frame.content_window().unwrap();
    (frame, window)
}
fn buffers(terminal: &Terminal) -> Rc<BufferService> {
    terminal
        .with_state(|state| Rc::clone(&state.buffer_service))
        .unwrap()
}
fn pending(buffers: &BufferService) -> bool {
    buffers.buffers.normal().borrow().has_pending_cleanup()
        || buffers.buffers.alt().borrow().has_pending_cleanup()
}
struct Observation {
    value: JsValue,
    _pending: Closure<dyn Fn() -> Result<bool, JsValue>>,
}
impl Observation {
    fn new(window: &Window, terminal: &Rc<Terminal>) -> Self {
        let weak = Rc::downgrade(terminal);
        let check = Closure::wrap(Box::new(move || -> Result<bool, JsValue> {
            let Some(terminal) = weak.upgrade() else {
                return Ok(false);
            };
            let buffers = terminal
                .with_state(|state| Rc::clone(&state.buffer_service))
                .map_err(|e| to_js_error(&e))?;
            Ok(pending(&buffers))
        }) as Box<dyn Fn() -> Result<bool, JsValue>>);
        let value = observe_cleanup(window, check.as_ref().unchecked_ref());
        Self {
            value,
            _pending: check,
        }
    }
    async fn completed(&self) {
        let promise: Promise = Reflect::get(&self.value, &"promise".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        JsFuture::from(promise).await.unwrap();
    }
    fn count(&self, name: &str) -> f64 {
        let records = Reflect::get(&self.value, &"records".into()).unwrap();
        Reflect::get(&records, &name.into())
            .unwrap()
            .as_f64()
            .unwrap()
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        Reflect::get(&self.value, &"restore".into())
            .unwrap()
            .unchecked_into::<Function>()
            .call0(&self.value)
            .unwrap();
    }
}
#[wasm_bindgen_test(async)]
async fn real_idle_or_priority_callback_cleans_parser_populated_buffers() {
    let (frame, window) = frame();
    let terminal = Rc::new(Terminal::new(12, 4, 10).unwrap());
    let observer = Observation::new(&window, &terminal);
    let owner = buffer_cleanup::install_in_window(&terminal, &window).unwrap();
    terminal.write("hello\r\nworld\r\nRust").unwrap();
    terminal.resize(3, 4).unwrap();
    let buffers = buffers(&terminal);
    assert!(pending(&buffers));
    observer.completed().await;
    assert!(!pending(&buffers));
    assert!(observer.count("requested") > 0.0);
    assert!(observer.count("fired") > 0.0);
    owner.dispose();
    drop(observer);
    frame.remove();
}
#[wasm_bindgen_test(async)]
async fn inactive_normal_and_active_alternate_are_both_scheduled() {
    let (frame, window) = frame();
    let terminal = Rc::new(Terminal::new(12, 4, 10).unwrap());
    let observer = Observation::new(&window, &terminal);
    let owner = buffer_cleanup::install_in_window(&terminal, &window).unwrap();
    terminal.write("normal\x1b[?1049halt").unwrap();
    terminal.resize(3, 4).unwrap();
    let buffers = buffers(&terminal);
    assert!(buffers.buffers.normal().borrow().has_pending_cleanup());
    assert!(buffers.buffers.alt().borrow().has_pending_cleanup());
    observer.completed().await;
    assert!(!pending(&buffers));
    assert!(observer.count("requested") >= 2.0);
    owner.dispose();
    drop(observer);
    frame.remove();
}
#[wasm_bindgen_test]
fn reset_and_host_detach_cancel_real_pending_tokens() {
    let (frame, window) = frame();
    let terminal = Rc::new(Terminal::new(12, 4, 10).unwrap());
    let observer = Observation::new(&window, &terminal);
    let owner = buffer_cleanup::install_in_window(&terminal, &window).unwrap();
    terminal.write("old").unwrap();
    terminal.resize(3, 4).unwrap();
    terminal.reset().unwrap();
    assert!(observer.count("cancelled") > 0.0);
    terminal.resize(12, 4).unwrap();
    terminal.write("new").unwrap();
    terminal.resize(3, 4).unwrap();
    let before = observer.count("cancelled");
    owner.dispose();
    assert!(observer.count("cancelled") > before);
    drop(observer);
    frame.remove();
}
#[wasm_bindgen_test]
fn scheduler_throw_preserves_the_original_js_object() {
    let (frame, window) = frame();
    let terminal = Rc::new(Terminal::new(12, 4, 10).unwrap());
    let sentinel = Object::new();
    let old = Reflect::get(&window, &"requestIdleCallback".into()).unwrap();
    let existed = Reflect::has(&window, &"requestIdleCallback".into()).unwrap();
    let request = Function::new_with_args("error", "return function(){throw error;}")
        .call1(&JsValue::NULL, &sentinel)
        .unwrap();
    Reflect::set(&window, &"requestIdleCallback".into(), &request).unwrap();
    let owner = buffer_cleanup::install_in_window(&terminal, &window).unwrap();
    terminal.write("test").unwrap();
    let error = terminal.resize(3, 4).unwrap_err();
    assert!(Object::is(&to_js_error(&error), &sentinel));
    owner.dispose();
    if existed {
        Reflect::set(&window, &"requestIdleCallback".into(), &old).unwrap();
    } else {
        Reflect::delete_property(&window, &"requestIdleCallback".into()).unwrap();
    }
    frame.remove();
}
