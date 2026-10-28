use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlElement};
use xterm_web::{dom_listener::DomListener, input::touch::GestureContext};

pub type Log = Rc<RefCell<Vec<(String, Event)>>>;
pub struct Setup {
    pub parent: HtmlElement,
    pub child: HtmlElement,
    pub leaf: HtmlElement,
    pub context: Rc<GestureContext>,
    pub log: Log,
    listeners: Vec<DomListener>,
    clock: Clock,
}
impl Setup {
    pub fn new() -> Self {
        let clock = js_sys::Function::new_no_args(r#"
const w=window, own=Object.getOwnPropertyDescriptor(w,'ontouchstart');
const OriginalDate=Date, raf=w.requestAnimationFrame, cancel=w.cancelAnimationFrame;
let time=1000, id=1; const frames=new Map();
Object.defineProperty(w,'ontouchstart',{configurable:true,writable:true,value:null});
function FakeDate(...args){return args.length ? new OriginalDate(...args) : new OriginalDate(time);}
Object.setPrototypeOf(FakeDate,OriginalDate);FakeDate.prototype=OriginalDate.prototype;
FakeDate.now=()=>time;w.Date=FakeDate;
w.requestAnimationFrame=cb=>{const n=id++;frames.set(n,cb);return n;};
w.cancelAnimationFrame=n=>frames.delete(n);
return {time(n){time=n;},flush(){const work=[...frames.values()];frames.clear();for(const cb of work)cb(0);},
pending(){return frames.size;},restore(){w.Date=OriginalDate;w.requestAnimationFrame=raf;w.cancelAnimationFrame=cancel;
if(own)Object.defineProperty(w,'ontouchstart',own);else delete w.ontouchstart;}};
"#).call0(&JsValue::UNDEFINED).unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        let element = || {
            document
                .create_element("div")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap()
        };
        let parent = element();
        let child = element();
        let leaf = element();
        parent.append_child(&child).unwrap();
        child.append_child(&leaf).unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let context = GestureContext::new(document).unwrap();
        Self {
            parent,
            child,
            leaf,
            context,
            log: Rc::new(RefCell::new(Vec::new())),
            listeners: Vec::new(),
            clock: Clock(clock),
        }
    }
    pub fn track(&mut self, child: bool) {
        let (target, label) = if child {
            (&self.child, "child")
        } else {
            (&self.parent, "parent")
        };
        for kind in [
            "-xterm-gesturestart",
            "-xterm-gesturechange",
            "-xterm-gesturetap",
            "-xterm-gesturesend",
            "-xterm-gesturecontextmenu",
        ] {
            let log = Rc::clone(&self.log);
            self.listeners.push(
                DomListener::new(target.as_ref(), kind, false, move |event| {
                    log.borrow_mut().push((label.into(), event));
                })
                .unwrap(),
            );
        }
    }
    pub fn time(&self, value: f64) {
        call(&self.clock.0, "time", Some(value.into()));
    }
    pub fn flush(&self) {
        call(&self.clock.0, "flush", None);
    }
    pub fn pending(&self) -> f64 {
        call(&self.clock.0, "pending", None).as_f64().unwrap()
    }
    pub fn send(&self, kind: &str, id: i32, x: f64, y: f64) -> Event {
        send(&self.leaf, kind, id, x, y)
    }
    pub fn clear(&self) {
        self.log.borrow_mut().clear();
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        self.context.dispose().unwrap();
        self.listeners.clear();
        self.parent.remove();
    }
}
struct Clock(JsValue);
impl Drop for Clock {
    fn drop(&mut self) {
        call(&self.0, "restore", None);
    }
}
fn call(target: &JsValue, name: &str, arg: Option<JsValue>) -> JsValue {
    let function: js_sys::Function = js_sys::Reflect::get(target, &name.into())
        .unwrap()
        .dyn_into()
        .unwrap();
    match arg {
        Some(arg) => function.call1(target, &arg),
        None => function.call0(target),
    }
    .unwrap()
}
pub fn get(event: &Event, name: &str) -> JsValue {
    js_sys::Reflect::get(event, &name.into()).unwrap()
}
pub fn number(event: &Event, name: &str) -> f64 {
    get(event, name).as_f64().unwrap()
}
pub fn has(event: &Event, name: &str) -> bool {
    js_sys::Reflect::has(event, &name.into()).unwrap()
}
pub fn send(target: &HtmlElement, kind: &str, id: i32, x: f64, y: f64) -> Event {
    send_batch(target, kind, &[(id, x, y)])
}
pub fn send_batch(target: &HtmlElement, kind: &str, points: &[(i32, f64, f64)]) -> Event {
    let list = js_sys::Array::new();
    for &(id, x, y) in points {
        let touch = js_sys::Object::new();
        for (name, value) in [
            ("identifier", id.into()),
            ("pageX", x.into()),
            ("pageY", y.into()),
            ("clientX", (x + 1.0).into()),
            ("clientY", (y + 2.0).into()),
            ("target", target.clone().into()),
        ] {
            js_sys::Reflect::set(&touch, &name.into(), &value).unwrap();
        }
        list.push(&touch);
    }
    let item = js_sys::Function::new_with_args("i", "return this[i];");
    js_sys::Reflect::set(&list, &"item".into(), &item).unwrap();
    let event = target
        .owner_document()
        .unwrap()
        .create_event("Event")
        .unwrap();
    event.init_event_with_bubbles_and_cancelable(kind, true, true);
    for name in ["targetTouches", "changedTouches"] {
        js_sys::Reflect::set(&event, &name.into(), &list).unwrap();
    }
    target.dispatch_event(&event).unwrap();
    event
}
