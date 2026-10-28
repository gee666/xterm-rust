//! JS object mechanics backed by Rust validation and event order.
#[path = "options_convert.rs"]
mod convert;
#[path = "options_store.rs"]
mod store;
use convert::ConversionCache;
pub use convert::{error_to_js, from_js, into_js, operator_semantics, property_access};
use js_sys::{Array, Function, Object};
use std::{
    cell::{OnceCell, RefCell},
    rc::{Rc, Weak},
};
use store::JsOptionStore;
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::{
    headless::PublicOptions,
    options::{OptionValue, OptionsService},
};
thread_local! { static JS_DEFAULTS:OnceCell<Object>=const{OnceCell::new()}; }

#[wasm_bindgen(inline_js = "
const optionOwners=new WeakMap();
export function copyOptionsDefaults(template) { return {...template}; }
export function initializeOptions(input,defaults,apply) {
  for(const key in input) if(key in defaults) {
    try { defaults[key]=apply(key,input[key]); } catch(error) { console.error(error); }
  }
  return defaults;
}
export function rawKeys(raw) { const keys=[];for(const key in raw)keys.push(key);return keys; }
export function optionLayout(view) { return Object.getOwnPropertyNames(view).map(key=>[key,Object.getOwnPropertyDescriptor(view,key).enumerable]); }
export function optionMembership(template,key) { return key in template; }
export function optionStrictEqual(left,right) { return left===right; }
export function defineOption(target,key,get,set) { Object.defineProperty(target,key,{get,set}); }
export function defineNativeOption(target,key,enumerable,get,set) { Object.defineProperty(target,key,{get,set,enumerable,configurable:enumerable}); }
export function publicOptions(core,checkReadonly,owner) {
  const view={...core};
  const getter=key=>core[key];
  const setter=(key,value)=>{checkReadonly(key);core[key]=value;};
  for(const key in core)Object.defineProperty(view,key,{get:getter.bind(null,key),set:setter.bind(null,key)});
  optionOwners.set(view,owner);
  return view;
}
export function assignOptions(target,input) { for(const key in input)target[key]=input[key]; }
")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn copyOptionsDefaults(template: &Object) -> Result<Object, JsValue>;
    #[wasm_bindgen(catch)]
    fn initializeOptions(
        input: &JsValue,
        defaults: &Object,
        apply: &Function,
    ) -> Result<Object, JsValue>;
    #[wasm_bindgen(catch)]
    fn rawKeys(raw: &Object) -> Result<Array, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionLayout(view: &Object) -> Result<Array, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionMembership(template: &Object, key: &str) -> Result<bool, JsValue>;
    fn optionStrictEqual(left: &JsValue, right: &JsValue) -> bool;
    #[wasm_bindgen(catch)]
    fn defineOption(
        target: &Object,
        key: &str,
        get: &JsValue,
        set: &JsValue,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn defineNativeOption(
        target: &Object,
        key: &str,
        enumerable: bool,
        get: &JsValue,
        set: &JsValue,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn publicOptions(core: &Object, check: &Function, owner: &JsValue) -> Result<Object, JsValue>;
    #[wasm_bindgen(catch)]
    fn assignOptions(target: &Object, input: &JsValue) -> Result<(), JsValue>;
}
type ServiceSlot = Rc<RefCell<Weak<OptionsService>>>;
fn upgrade(slot: &ServiceSlot) -> Result<Rc<OptionsService>, JsValue> {
    slot.borrow()
        .upgrade()
        .ok_or_else(|| js_sys::Error::new("options service released").into())
}
fn install_host_accessors(
    view: &Object,
    keys: &[String],
    slot: &ServiceSlot,
    cache: &Rc<ConversionCache>,
) -> Result<(), JsValue> {
    for key in keys {
        let get_slot = slot.clone();
        let get_key = key.clone();
        let conversion = cache.clone();
        let get = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
            conversion.convert(
                &upgrade(&get_slot)?
                    .get(&get_key)
                    .map_err(|e| error_to_js(&e))?,
            )
        })
        .into_js_value();
        let set_slot = slot.clone();
        let set_key = key.clone();
        let set = Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(move |value: JsValue| {
            upgrade(&set_slot)?
                .set(&set_key, from_js_key(&set_key, &value)?)
                .map_err(|e| error_to_js(&e))
        })
        .into_js_value();
        defineOption(view, key, &get, &set)?;
    }
    Ok(())
}
fn layout(view: &Object) -> Result<Vec<(String, bool)>, JsValue> {
    optionLayout(view)?
        .iter()
        .map(|entry| {
            let pair = Array::from(&entry);
            Ok((
                pair.get(0)
                    .as_string()
                    .ok_or_else(|| js_sys::TypeError::new("invalid option key"))?,
                pair.get(1)
                    .as_bool()
                    .ok_or_else(|| js_sys::TypeError::new("invalid option enumerability"))?,
            ))
        })
        .collect()
}

pub fn constructor_options(input: &JsValue, is_mac: bool) -> Result<Rc<OptionsService>, JsValue> {
    let service = Rc::new(OptionsService::new(
        is_mac,
        std::iter::empty::<(&str, OptionValue)>(),
    ));
    let cache = ConversionCache::default();
    let template = JS_DEFAULTS.with(|cell| -> Result<Object, JsValue> {
        if let Some(template) = cell.get() {
            return Ok(template.clone());
        }
        let template = Object::new();
        for key in service.keys() {
            convert::set_data(
                &template,
                key,
                &cache.convert(&service.get(key).map_err(|e| error_to_js(&e))?)?,
            )?;
        }
        cell.set(template.clone())
            .map_err(|_| js_sys::Error::new("reentrant defaults initialization"))?;
        Ok(template)
    })?;
    for key in service.keys() {
        cache.remember(
            &service.get(key).map_err(|e| error_to_js(&e))?,
            &js_sys::Reflect::get(&template, &key.into())?,
        );
    }
    let defaults = copyOptionsDefaults(&template)?;
    convert::set_data(&defaults, "rightClickSelectsWord", &is_mac.into())?;
    let core = service.clone();
    let cache = Rc::new(cache);
    let conversion = cache.clone();
    let apply = Closure::<dyn Fn(String, JsValue) -> Result<JsValue, JsValue>>::new(
        move |key: String, value: JsValue| {
            conversion.convert(
                &core
                    .sanitize_value(&key, from_js(&value)?)
                    .map_err(|e| error_to_js(&e))?,
            )
        },
    );
    let raw = initializeOptions(input, &defaults, apply.as_ref().unchecked_ref())?;
    drop(apply);
    // The source copies raw options before enumerating/installing accessors.
    // Own getters can throw, delete fields or add fields during this spread.
    let view = copyOptionsDefaults(&raw)?;
    let keys = rawKeys(&raw)?
        .iter()
        .map(|key| {
            key.as_string()
                .ok_or_else(|| js_sys::TypeError::new("invalid raw option key").into())
        })
        .collect::<Result<Vec<String>, JsValue>>()?;
    let slot = Rc::new(RefCell::new(Weak::new()));
    install_host_accessors(&view, &keys, &slot, &cache)?;
    let final_layout = layout(&view)?;
    let service = Rc::try_unwrap(service)
        .map_err(|_| js_sys::Error::new("constructor callback retained service"))?;
    let service = Rc::new(service.with_raw_store(
        Rc::new(JsOptionStore {
            raw,
            view,
            accessors: keys.into_iter().collect(),
            cache,
        }),
        final_layout,
    ));
    *slot.borrow_mut() = Rc::downgrade(&service);
    Ok(service)
}
fn from_js_key(key: &str, value: &JsValue) -> Result<OptionValue, JsValue> {
    match key {
        "logger" => crate::logger::from_js(value),
        "logLevel" => crate::logger::log_level_from_js(value),
        _ => from_js(value),
    }
}
struct BindingState {
    options: PublicOptions,
    cache: ConversionCache,
}
pub struct OptionsBinding {
    object: Object,
    state: Rc<BindingState>,
}
impl OptionsBinding {
    pub fn new(options: PublicOptions) -> Result<Self, JsValue> {
        let state = Rc::new(BindingState {
            options,
            cache: ConversionCache::default(),
        });
        let existing = state
            .options
            .service()
            .host_options_view()
            .map_err(|e| error_to_js(&e))?;
        let core_view = if let Some(view) = existing {
            into_js(&view)?
                .dyn_into::<Object>()
                .map_err(|_| js_sys::TypeError::new("invalid host options view"))?
        } else {
            let view = Object::new();
            for (key, enumerable) in state.options.service().property_layout() {
                let get_state = state.clone();
                let get_key = key.clone();
                let get = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
                    get_state.cache.convert(
                        &get_state
                            .options
                            .get(&get_key)
                            .map_err(|e| error_to_js(&e))?,
                    )
                })
                .into_js_value();
                let set_state = state.clone();
                let set_key = key.clone();
                let set = Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(
                    move |value: JsValue| {
                        set_state
                            .options
                            .service()
                            .set(&set_key, from_js_key(&set_key, &value)?)
                            .map_err(|e| error_to_js(&e))
                    },
                )
                .into_js_value();
                defineNativeOption(&view, &key, enumerable, &get, &set)?;
            }
            view
        };
        let check = Closure::<dyn Fn(String) -> Result<(), JsValue>>::new(|key: String| {
            crate::api::verify_mutable_option(&key)
                .map_err(|e| js_sys::Error::new(&e.to_string()).into())
        });
        // The JS-owned public view keeps the service alive without adding an
        // own property or a service -> view -> service strong-reference loop.
        let owner_state = state.clone();
        let owner = Closure::<dyn Fn()>::new(move || {
            let _ = &owner_state;
        })
        .into_js_value();
        let object = publicOptions(&core_view, check.as_ref().unchecked_ref(), &owner)?;
        let _check = check.into_js_value();
        Ok(Self { object, state })
    }
    pub fn object(&self) -> Object {
        self.object.clone()
    }
    pub fn assign(&self, input: &JsValue) -> Result<(), JsValue> {
        assignOptions(&self.object, input)
    }
    pub fn service(&self) -> Rc<OptionsService> {
        self.state.options.service()
    }
    pub fn take_dispose_error(&self) -> Option<JsValue> {
        self.service().take_dispose_error().map(|e| error_to_js(&e))
    }
}
