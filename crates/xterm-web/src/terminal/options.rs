//! JS descriptor/prototype mechanics backed by Rust validation and event order.
#[path = "options_convert.rs"]
mod convert;
use convert::ConversionCache;
pub use convert::{error_to_js, from_js, into_js};
use js_sys::{Function, Object};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::{
    headless::{PublicOptionError, PublicOptions},
    options::{OptionError, OptionValue, OptionsService, RawOptionsStore},
};

#[wasm_bindgen(inline_js = "
export function initializeOptions(input, defaults, apply) {
  for (const key in input) {
    if (key in defaults) {
      try { defaults[key] = apply(key, input[key]); }
      catch (error) { console.error(error); }
    }
  }
  return defaults;
}
export function optionLayout(raw) {
  const entries = [];
  for (const key in raw) entries.push([key, Object.prototype.hasOwnProperty.call(raw, key)]);
  return entries;
}
export function defineOption(target, key, get, set) {
  Object.defineProperty(target, key, {get, set, enumerable:true, configurable:true});
}
export function assignOptions(target, input) {
  for (const key in input) target[key] = input[key];
}
")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn initializeOptions(
        input: &JsValue,
        defaults: &Object,
        apply: &Function,
    ) -> Result<Object, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionLayout(raw: &Object) -> Result<js_sys::Array, JsValue>;
    #[wasm_bindgen(catch)]
    fn defineOption(
        target: &Object,
        key: &str,
        get: &JsValue,
        set: &JsValue,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn assignOptions(target: &Object, input: &JsValue) -> Result<(), JsValue>;
}

pub fn constructor_options(input: &JsValue, is_mac: bool) -> Result<Rc<OptionsService>, JsValue> {
    let service = Rc::new(OptionsService::new(
        is_mac,
        std::iter::empty::<(&str, OptionValue)>(),
    ));
    let defaults = Object::new();
    let cache = ConversionCache::default();
    for key in service.keys() {
        let value = cache.convert(&service.get(key).map_err(|e| error_to_js(&e))?)?;
        js_sys::Reflect::set(&defaults, &key.into(), &value)?;
    }
    let core = service.clone();
    let conversion = Rc::new(cache);
    let apply = Closure::<dyn FnMut(String, JsValue) -> Result<JsValue, JsValue>>::new(
        move |key: String, value: JsValue| {
            let value = core
                .sanitize_value(&key, from_js(&value)?)
                .map_err(|e| error_to_js(&e))?;
            conversion.convert(&value)
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
            cache: ConversionCache::default(),
        }),
        layout,
    )))
}

fn from_js_key(key: &str, value: &JsValue) -> Result<OptionValue, JsValue> {
    if key == "logger" {
        crate::logger::from_js(value)
    } else {
        from_js(value)
    }
}

struct JsOptionStore {
    raw: Object,
    cache: ConversionCache,
}
impl RawOptionsStore for JsOptionStore {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        js_sys::Reflect::get(&self.raw, &key.into())
            .and_then(|value| from_js_key(key, &value))
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
        let object = Object::new();
        let state = Rc::new(BindingState {
            options,
            cache: ConversionCache::default(),
        });
        for (key, enumerable) in state.options.service().property_layout() {
            if !enumerable {
                continue;
            }
            // Public Terminal first spreads core.options. Every enumerable
            // getter runs before forwarding descriptors are installed.
            state
                .cache
                .convert(&state.options.get(&key).map_err(|e| error_to_js(&e))?)?;
            let get_key = key.clone();
            let get_state = state.clone();
            let get = Closure::<dyn FnMut() -> Result<JsValue, JsValue>>::new(move || {
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
                Closure::<dyn FnMut(JsValue) -> Result<(), JsValue>>::new(move |value: JsValue| {
                    set_state
                        .options
                        .set(&set_key, from_js_key(&set_key, &value)?)
                        .map_err(|error| match error {
                            PublicOptionError::Option(error) => error_to_js(&error),
                            PublicOptionError::Api(error) => {
                                js_sys::Error::new(&error.to_string()).into()
                            }
                        })
                })
                .into_js_value();
            // into_js_value transfers closure lifetime to JS GC, unlike forget.
            // Captured state does not retain the options object itself.
            defineOption(&object, &key, &get, &set)?;
        }
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
