// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Controlled timer turns for component tests only. Terminal E2E uses real jobs.
use js_sys::{Function, Reflect};
use std::cell::Cell;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Window;

#[wasm_bindgen(inline_js = r#"
export function installAccessibilityClock(win, start) {
  const keys = ['setTimeout','clearTimeout','requestAnimationFrame','cancelAnimationFrame'];
  const saved = new Map(keys.map(key => [key,Object.getOwnPropertyDescriptor(win,key)]));
  const originalClear = win.clearTimeout.bind(win);
  const originalCancel = win.cancelAnimationFrame.bind(win);
  const nowDescriptor = Object.getOwnPropertyDescriptor(win.performance,'now');
  const jobs = new Map();
  const FIRST_TEST_ID = 0x40000000;
  const MAX_TEST_CALLBACKS = 10000;
  let next = FIRST_TEST_ID, now = start, restored = false;
  Object.defineProperty(win.performance,'now',{configurable:true,value:()=>now});
  win.setTimeout = (callback, delay=0, ...args) => {
    if (typeof callback !== 'function') throw new TypeError('test clock accepts function callbacks only');
    const id=next++; const numeric=Number(delay); const wait=Number.isFinite(numeric)?Math.max(0,numeric>>0):0;
    jobs.set(id,{when:now+wait,callback,args,frame:false}); return id;
  };
  win.clearTimeout = id => { if (jobs.get(id)?.frame === false) jobs.delete(id); else originalClear(id); };
  win.requestAnimationFrame = callback => {
    const id=next++; jobs.set(id,{when:now+16,callback,args:[],frame:true}); return id;
  };
  win.cancelAnimationFrame = id => { if (jobs.get(id)?.frame === true) jobs.delete(id); else originalCancel(id); };
  return {
    advance(milliseconds) {
      const target=now+milliseconds;
      if (!Number.isFinite(target) || milliseconds<0) throw new RangeError('invalid test clock advance');
      let calls=0;
      while (true) {
        const due=[...jobs].filter(([,job])=>job.when<=target).sort((a,b)=>a[1].when-b[1].when||a[0]-b[0])[0];
        if (!due) break;
        if (++calls>MAX_TEST_CALLBACKS) throw new Error('test callback budget exceeded');
        const [id,job]=due; jobs.delete(id); now=job.when;
        job.callback.apply(win,job.frame?[now]:job.args);
      }
      now=target;
    },
    pending() { return jobs.size; },
    restore() {
      if (restored) return;
      restored=true; jobs.clear();
      for (const key of keys) {
        const descriptor=saved.get(key);
        if (descriptor) Object.defineProperty(win,key,descriptor); else delete win[key];
      }
      if (nowDescriptor) Object.defineProperty(win.performance,'now',nowDescriptor); else delete win.performance.now;
    }
  };
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = installAccessibilityClock)]
    fn install(window: &Window, start: f64) -> Result<JsValue, JsValue>;
}

pub struct Clock {
    value: JsValue,
    restored: Cell<bool>,
}
impl Clock {
    pub fn new(window: &Window) -> Self {
        Self {
            value: install(window, 2000.0).unwrap(),
            restored: Cell::new(false),
        }
    }
    pub fn advance(&self, milliseconds: f64) -> Result<(), JsValue> {
        let function: Function = Reflect::get(&self.value, &"advance".into())?.dyn_into()?;
        function
            .call1(&self.value, &milliseconds.into())
            .map(|_| ())
    }
    pub fn pending(&self) -> usize {
        let function: Function = Reflect::get(&self.value, &"pending".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        function.call0(&self.value).unwrap().as_f64().unwrap() as usize
    }
    fn restore(&self) {
        if !self.restored.replace(true) {
            let function: Function = Reflect::get(&self.value, &"restore".into())
                .unwrap()
                .dyn_into()
                .unwrap();
            function
                .call0(&self.value)
                .expect("test clock restores its saved descriptors");
        }
    }
}
impl Drop for Clock {
    fn drop(&mut self) {
        self.restore();
    }
}
