// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! The mouse normalizer's same-origin window-chain cache. JS WeakMap/WeakRef
//! retain the source's GC semantics without Rust references pinning child frames.
use super::interop::{add, get, method0, method1, method2, nullish};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Window;

#[wasm_bindgen(inline_js = "
export function scrollWeakWindow(window) { return new WeakRef(window); }
export function scrollChainEntry(window, iframeElement) { return { window, iframeElement }; }
")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn scrollWeakWindow(window: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn scrollChainEntry(window: &JsValue, iframe: &JsValue) -> Result<JsValue, JsValue>;
}

pub struct IframeCache {
    cache: js_sys::WeakMap,
}
impl Default for IframeCache {
    fn default() -> Self {
        Self {
            cache: js_sys::WeakMap::new(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct IframePosition {
    pub top: JsValue,
    pub left: JsValue,
}
impl IframeCache {
    pub fn new() -> Self {
        Self::default()
    }
    fn chain(&self, target: &JsValue) -> Result<js_sys::Array, JsValue> {
        let mut chain = method1(self.cache.as_ref(), "get", target)?;
        if !chain.is_truthy() {
            chain = js_sys::Array::new().into();
            // Source publishes the array before walking. If a later getter
            // fails, subsequent calls observe that same partially built cache.
            method2(self.cache.as_ref(), "set", target, &chain)?;
            let mut window = target.clone();
            // Browser parent windows form a finite tree, with the root pointing
            // to itself. same_origin_parent stops at that root or an origin edge.
            loop {
                let parent = same_origin_parent(&window)?;
                let weak = scrollWeakWindow(&window)?;
                let iframe = if parent.is_some() {
                    let value = get(&window, "frameElement")?;
                    if nullish(&value) {
                        JsValue::NULL
                    } else {
                        value
                    }
                } else {
                    JsValue::NULL
                };
                let entry = scrollChainEntry(&weak, &iframe)?;
                method1(&chain, "push", &entry)?;
                let Some(next) = parent else {
                    break;
                };
                window = next;
            }
        }
        let copy = method1(&chain, "slice", &JsValue::from_f64(0.0))?;
        copy.dyn_into()
            .map_err(|_| js_sys::TypeError::new("Iframe chain must be an array").into())
    }
    pub fn position_relative_to(
        &self,
        child: &Window,
        ancestor: &JsValue,
    ) -> Result<IframePosition, JsValue> {
        let mut position = IframePosition {
            top: 0.0.into(),
            left: 0.0.into(),
        };
        if !ancestor.is_truthy() || AsRef::<JsValue>::as_ref(child) == ancestor {
            return Ok(position);
        }
        for entry in self.chain(child.as_ref())?.iter() {
            let window = method0(&get(&entry, "window")?, "deref")?;
            let y = if nullish(&window) {
                JsValue::UNDEFINED
            } else {
                get(&window, "scrollY")?
            };
            position.top = add(&position.top, &if nullish(&y) { 0.0.into() } else { y })?;
            let x = if nullish(&window) {
                JsValue::UNDEFINED
            } else {
                get(&window, "scrollX")?
            };
            position.left = add(&position.left, &if nullish(&x) { 0.0.into() } else { x })?;
            if window == *ancestor {
                break;
            }
            if !get(&entry, "iframeElement")?.is_truthy() {
                break;
            }
            let bounds = method0(&get(&entry, "iframeElement")?, "getBoundingClientRect")?;
            position.top = add(&position.top, &get(&bounds, "top")?)?;
            position.left = add(&position.left, &get(&bounds, "left")?)?;
        }
        Ok(position)
    }
}
fn same_origin_parent(window: &JsValue) -> Result<Option<JsValue>, JsValue> {
    if !get(window, "parent")?.is_truthy() || get(window, "parent")? == *window {
        return Ok(None);
    }
    let origins = (|| {
        let location = get(window, "location")?;
        let parent_location = get(&get(window, "parent")?, "location")?;
        let null = JsValue::from_str("null");
        if get(&location, "origin")? != null
            && get(&parent_location, "origin")? != null
            && get(&location, "origin")? != get(&parent_location, "origin")?
        {
            return Ok(false);
        }
        Ok::<_, JsValue>(true)
    })();
    // This is the source's explicit security-boundary catch, not a fallback for
    // other DOM failures. Frame/cache/geometry getter failures still propagate.
    if !matches!(origins, Ok(true)) {
        return Ok(None);
    }
    let parent = get(window, "parent")?;
    Ok(parent.is_truthy().then_some(parent))
}
