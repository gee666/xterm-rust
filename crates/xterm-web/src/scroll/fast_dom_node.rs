// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Port of browser/scrollable/fastDomNode.ts.

use crate::terminal::options::{from_js, into_js};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use xterm_core::options::OptionValue as V;

// Only primitive DOM assignments cross this boundary. Rust owns all caching and
// number policy. Strict JS assignment preserves actual host exceptions and reads
// the current cache AFTER an overridden style getter has had a chance to reenter.
#[wasm_bindgen(inline_js = "
export function fastCacheNew() { return {width:'',height:'',top:'',left:'',bottom:'',right:'',position:'',contain:'none',className:''}; }
export function fastCacheSet(cache,key,value) { 'use strict'; cache[key]=value; }
export function fastCachedStyle(node,key,cache) { 'use strict'; node.style[key]=cache[key]; }
export function fastCachedClass(node,cache) { 'use strict'; node.className=cache.className; }
export function fastLayerHint(node,enabled) { 'use strict'; node.style.transform=enabled?'translate3d(0px, 0px, 0px)':''; }
export function fastToggle(node,name,force) { 'use strict'; node.classList.toggle(name,force); return node.className; }
export function fastAttribute(node,name,value) { 'use strict'; node.setAttribute(name,value); }
export function fastBeginStyle(node) { return node.style; }
export function fastAssignStyle(style,key,value) { 'use strict'; style[key]=value; }
")]
extern "C" {
    fn fastCacheNew() -> JsValue;
    #[wasm_bindgen(catch)]
    fn fastCacheSet(cache: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn fastCachedStyle(node: &HtmlElement, key: &str, cache: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn fastCachedClass(node: &HtmlElement, cache: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn fastLayerHint(node: &HtmlElement, enabled: bool) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn fastToggle(node: &HtmlElement, name: &JsValue, force: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn fastAttribute(node: &HtmlElement, name: &JsValue, value: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn fastBeginStyle(node: &HtmlElement) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn fastAssignStyle(style: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue>;
}

pub(crate) fn begin_style(node: &HtmlElement) -> Result<JsValue, JsValue> {
    fastBeginStyle(node)
}
pub(crate) fn assign_style(style: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue> {
    fastAssignStyle(style, key, value)
}
pub(crate) fn set_style(node: &HtmlElement, key: &str, value: &JsValue) -> Result<(), JsValue> {
    assign_style(&begin_style(node)?, key, value)
}
pub(crate) fn set_style_with(
    node: &HtmlElement,
    key: &str,
    create: impl FnOnce() -> Result<JsValue, JsValue>,
) -> Result<(), JsValue> {
    let style = begin_style(node)?;
    let value = create()?;
    assign_style(&style, key, &value)
}

/// Both portable option values and live browser values are consumed without
/// premature string/number coercion. This keeps geometry and DOM callers shared.
pub trait DomValue {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue>;
}
impl DomValue for V {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue> {
        Ok(Cow::Borrowed(self))
    }
}
impl DomValue for JsValue {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue> {
        from_js(self).map(Cow::Owned)
    }
}
impl DomValue for f64 {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue> {
        Ok(Cow::Owned(V::Number(*self)))
    }
}
impl DomValue for str {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue> {
        Ok(Cow::Owned(V::from(self)))
    }
}
impl DomValue for String {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue> {
        self.as_str().option_value()
    }
}
impl DomValue for js_sys::JsString {
    fn option_value(&self) -> Result<Cow<'_, V>, JsValue> {
        from_js(self.as_ref()).map(Cow::Owned)
    }
}

#[derive(Clone, Copy)]
#[repr(usize)]
enum Property {
    Width,
    Height,
    Top,
    Left,
    Bottom,
    Right,
    Position,
    Contain,
}
const PROPERTY_NAMES: [&str; 8] = [
    "width", "height", "top", "left", "bottom", "right", "position", "contain",
];
struct Cache {
    styles: [Rc<V>; PROPERTY_NAMES.len()],
    class_name: Rc<V>,
    layer_hint: bool,
}

/// Clones represent the same FastDomNode/cache identity, not independent wrappers
/// around one element. New wrappers intentionally start with empty source caches.
#[derive(Clone)]
pub struct FastDomNode {
    pub dom_node: HtmlElement,
    cache: Rc<RefCell<Cache>>,
    js_cache: JsValue,
}
impl FastDomNode {
    pub fn new(dom_node: HtmlElement) -> Self {
        let mut styles = std::array::from_fn(|_| Rc::new(V::from("")));
        styles[Property::Contain as usize] = Rc::new(V::from("none"));
        Self {
            dom_node,
            cache: Rc::new(RefCell::new(Cache {
                styles,
                class_name: Rc::new(V::from("")),
                layer_hint: false,
            })),
            js_cache: fastCacheNew(),
        }
    }

    pub fn set_width<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.dimension(Property::Width, value.as_ref())
    }
    pub fn set_height<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.dimension(Property::Height, value.as_ref())
    }
    pub fn set_top<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.dimension(Property::Top, value.as_ref())
    }
    pub fn set_left<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.dimension(Property::Left, value.as_ref())
    }
    pub fn set_bottom<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.dimension(Property::Bottom, value.as_ref())
    }
    pub fn set_right<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.dimension(Property::Right, value.as_ref())
    }
    pub fn set_position<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        self.style(Property::Position, value.option_value()?)
    }
    pub fn set_contain<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        self.style(Property::Contain, value.option_value()?)
    }

    pub fn set_class_name<T: DomValue + ?Sized>(&self, value: &T) -> Result<(), JsValue> {
        let value = value.option_value()?;
        let previous = Rc::clone(&self.cache.borrow().class_name);
        if previous.as_ref() == value.as_ref() {
            return Ok(());
        }
        let next = Rc::new(value.into_owned());
        let replaced = std::mem::replace(&mut self.cache.borrow_mut().class_name, Rc::clone(&next));
        drop(previous);
        drop(replaced);
        fastCacheSet(&self.js_cache, "className", &into_js(&next)?)?;
        fastCachedClass(&self.dom_node, &self.js_cache)
    }

    pub fn toggle_class_name<T: DomValue + ?Sized>(
        &self,
        name: &T,
        force: Option<bool>,
    ) -> Result<(), JsValue> {
        let name = name.option_value()?;
        self.toggle_class_name_raw(name.as_ref(), &force.map(V::Bool).unwrap_or(V::Undefined))
    }
    pub fn toggle_class_name_raw(&self, name: &V, force: &V) -> Result<(), JsValue> {
        let class_name = fastToggle(&self.dom_node, &into_js(name)?, &into_js(force)?)?;
        let value = Rc::new(from_js(&class_name)?);
        let previous = std::mem::replace(&mut self.cache.borrow_mut().class_name, value);
        drop(previous);
        fastCacheSet(&self.js_cache, "className", &class_name)
    }

    pub fn set_layer_hinting(&self, enabled: bool) -> Result<(), JsValue> {
        if self.cache.borrow().layer_hint == enabled {
            return Ok(());
        }
        self.cache.borrow_mut().layer_hint = enabled;
        fastLayerHint(&self.dom_node, enabled)
    }
    pub fn set_attribute<T: DomValue + ?Sized>(
        &self,
        name: &str,
        value: &T,
    ) -> Result<(), JsValue> {
        let value = value.option_value()?;
        self.set_attribute_raw(&V::from(name), value.as_ref())
    }
    pub fn set_attribute_raw(&self, name: &V, value: &V) -> Result<(), JsValue> {
        fastAttribute(&self.dom_node, &into_js(name)?, &into_js(value)?)
    }

    fn dimension(&self, property: Property, value: &V) -> Result<(), JsValue> {
        let value = if let V::Number(number) = value {
            let text = format!("{}px", ryu_js::Buffer::new().format(*number));
            Cow::Owned(V::from(text.as_str()))
        } else {
            Cow::Borrowed(value)
        };
        self.style(property, value)
    }
    fn style(&self, property: Property, value: Cow<'_, V>) -> Result<(), JsValue> {
        let index = property as usize;
        let previous = Rc::clone(&self.cache.borrow().styles[index]);
        if previous.as_ref() == value.as_ref() {
            return Ok(());
        }
        let next = Rc::new(value.into_owned());
        let replaced =
            std::mem::replace(&mut self.cache.borrow_mut().styles[index], Rc::clone(&next));
        drop(previous);
        drop(replaced);
        let key = PROPERTY_NAMES[index];
        fastCacheSet(&self.js_cache, key, &into_js(&next)?)?;
        fastCachedStyle(&self.dom_node, key, &self.js_cache)
    }
}
