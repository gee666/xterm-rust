//! JS descriptor/prototype mechanics backed by Rust validation and event order.
#[path = "options_convert.rs"]
mod convert;
use convert::ConversionCache;
pub use convert::{error_to_js, from_js, into_js, operator_semantics};
use js_sys::{Function, Object};
use std::{cell::OnceCell, rc::Rc};
// Source DEFAULT_OPTIONS shares object values across every shallow copy.
thread_local! { static JS_DEFAULTS: OnceCell<Object> = const { OnceCell::new() }; }
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::{
    headless::PublicOptions,
    options::{OptionError, OptionValue, OptionsService, RawOptionsStore},
};

#[wasm_bindgen(inline_js = "
export function copyOptionsDefaults(template) { return {...template}; }
export function initializeOptions(input, defaults, apply) {
  for (const key in input) {
    if (key in defaults) {
      try { defaults[key] = apply(key, input[key]); }
      catch (error) { console.error(error); }
    }
  }
  return defaults;
}
export function optionMembership(template, key) { return key in template; }
export function optionStrictEqual(left,right) { return left === right; }
export function optionLayout(raw) {
  const entries = [];
  for (const key in raw) entries.push([key, Object.prototype.hasOwnProperty.call(raw, key)]);
  return entries;
}
export function defineOption(target, key, enumerable, get, set) {
  Object.defineProperty(target, key, {get, set, enumerable, configurable:enumerable});
}
export function publicOptions(core, checkReadonly) {
  const view = {...core};
  const getter = key => core[key];
  const setter = (key,value) => { checkReadonly(key); core[key]=value; };
  for (const key in core) Object.defineProperty(view,key,{get:getter.bind(null,key),set:setter.bind(null,key)});
  return view;
}
export function assignOptions(target, input) {
  for (const key in input) target[key] = input[key];
}
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
    fn optionLayout(raw: &Object) -> Result<js_sys::Array, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionMembership(template: &Object, key: &str) -> Result<bool, JsValue>;
    fn optionStrictEqual(left: &JsValue, right: &JsValue) -> bool;
    #[wasm_bindgen(catch)]
    fn defineOption(
        target: &Object,
        key: &str,
        enumerable: bool,
        get: &JsValue,
        set: &JsValue,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn assignOptions(target: &Object, input: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn publicOptions(core: &Object, check_readonly: &Function) -> Result<Object, JsValue>;
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
            let value = cache.convert(&service.get(key).map_err(|e| error_to_js(&e))?)?;
            convert::set_data(&template, key, &value)?;
        }
        cell.set(template.clone())
            .map_err(|_| js_sys::Error::new("reentrant default options initialization"))?;
        Ok(template)
    })?;
    for key in service.keys() {
        let native = service.get(key).map_err(|e| error_to_js(&e))?;
        let host = js_sys::Reflect::get(&template, &key.into())?;
        cache.remember(&native, &host);
    }
    let defaults = copyOptionsDefaults(&template)?;
    convert::set_data(
        &defaults,
        "rightClickSelectsWord",
        &JsValue::from_bool(is_mac),
    )?;
    let core = service.clone();
    let conversion = Rc::new(cache);
    let callback_conversion = conversion.clone();
    let apply = Closure::<dyn Fn(String, JsValue) -> Result<JsValue, JsValue>>::new(
        move |key: String, value: JsValue| {
            let value = core
                .sanitize_value(&key, from_js(&value)?)
                .map_err(|e| error_to_js(&e))?;
            callback_conversion.convert(&value)
        },
    );
    let raw = initializeOptions(input, &defaults, apply.as_ref().unchecked_ref())?;
    drop(apply);
    let layout = optionLayout(&raw)?
        .iter()
        .map(|entry| {
            let pair = js_sys::Array::from(&entry);
            Ok((
                pair.get(0)
                    .as_string()
                    .ok_or_else(|| js_sys::TypeError::new("invalid option layout key"))?,
                pair.get(1)
                    .as_bool()
                    .ok_or_else(|| js_sys::TypeError::new("invalid option layout flag"))?,
            ))
        })
        .collect::<Result<Vec<_>, JsValue>>()?;
    let service = Rc::try_unwrap(service)
        .map_err(|_| js_sys::Error::new("constructor callback retained options service"))?;
    Ok(Rc::new(service.with_raw_store(
        Rc::new(JsOptionStore {
            raw,
            cache: conversion,
        }),
        layout,
    )))
}

fn from_js_key(key: &str, value: &JsValue) -> Result<OptionValue, JsValue> {
    if key == "logger" {
        crate::logger::from_js(value)
    } else if key == "logLevel" {
        crate::logger::log_level_from_js(value)
    } else {
        from_js(value)
    }
}

struct JsOptionStore {
    raw: Object,
    cache: Rc<ConversionCache>,
}
impl RawOptionsStore for JsOptionStore {
    fn strict_equal(
        &self,
        left: &OptionValue,
        right: &OptionValue,
    ) -> Result<Option<bool>, OptionError> {
        let left = self
            .cache
            .convert(left)
            .map_err(|e| OptionError::Dispatch(crate::js_error::callback_error(e)))?;
        let right = self
            .cache
            .convert(right)
            .map_err(|e| OptionError::Dispatch(crate::js_error::callback_error(e)))?;
        Ok(Some(optionStrictEqual(&left, &right)))
    }
    fn validate_key(&self, key: &str) -> Result<Option<bool>, OptionError> {
        JS_DEFAULTS.with(|defaults| {
            let template = defaults
                .get()
                .expect("host store installed after default initialization");
            optionMembership(template, key)
                .map(Some)
                .map_err(|error| OptionError::Dispatch(crate::js_error::callback_error(error)))
        })
    }
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        js_sys::Reflect::get(&self.raw, &key.into())
            .and_then(|value| {
                if key == "logger" {
                    crate::logger::from_js_with_options(&value, &self.raw)
                } else if key == "logLevel" {
                    crate::logger::log_level_from_js(&value)
                } else {
                    from_js(&value)
                }
            })
            .map_err(|error| OptionError::Dispatch(crate::js_error::callback_error(error)))
    }
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        let value = self
            .cache
            .convert(&value)
            .map_err(|error| OptionError::Dispatch(crate::js_error::callback_error(error)))?;
        js_sys::Reflect::set(&self.raw, &key.into(), &value)
            .map_err(|error| OptionError::Dispatch(crate::js_error::callback_error(error)))?
            .then_some(())
            .ok_or_else(|| {
                OptionError::Dispatch(crate::js_error::callback_error(
                    js_sys::TypeError::new("Cannot assign to read only option").into(),
                ))
            })
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
        let core_view = Object::new();
        let state = Rc::new(BindingState {
            options,
            cache: ConversionCache::default(),
        });
        for (key, enumerable) in state.options.service().property_layout() {
            let get_key = key.clone();
            let get_state = state.clone();
            let get = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
                get_state.cache.convert(
                    &get_state
                        .options
                        .get(&get_key)
                        .map_err(|e| error_to_js(&e))?,
                )
            })
            .into_js_value();
            let set_key = key.clone();
            let set_state = state.clone();
            let set =
                Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(move |value: JsValue| {
                    set_state
                        .options
                        .service()
                        .set(&set_key, from_js_key(&set_key, &value)?)
                        .map_err(|error| error_to_js(&error))
                })
                .into_js_value();
            // into_js_value transfers closure lifetime to JS GC, unlike forget.
            // Captured state does not retain the options object itself.
            defineOption(&core_view, &key, enumerable, &get, &set)?;
        }
        let check = Closure::<dyn Fn(String) -> Result<(), JsValue>>::new(|key: String| {
            crate::api::verify_mutable_option(&key)
                .map_err(|error| js_sys::Error::new(&error.to_string()).into())
        });
        // JS owns public forwarding descriptors, including prototype changes
        // caused by getters during the initial spread. Rust owns validation.
        let object = publicOptions(&core_view, check.as_ref().unchecked_ref())?;
        // The public setter retains this callback, so transfer it to JS GC.
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
