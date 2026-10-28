//! The actual raw object and its source-shaped options view are distinct.
use super::{
    convert::ConversionCache, from_js, from_js_key, optionMembership, optionStrictEqual,
    JS_DEFAULTS,
};
use js_sys::Object;
use std::{collections::HashSet, rc::Rc};
use wasm_bindgen::JsValue;
use xterm_core::options::{OptionError, OptionValue, RawOptionsStore};

pub(super) struct JsOptionStore {
    pub raw: Object,
    pub view: Object,
    pub accessors: HashSet<String>,
    pub cache: Rc<ConversionCache>,
}
fn host(error: JsValue) -> OptionError {
    OptionError::Dispatch(crate::js_error::callback_error(error))
}
impl JsOptionStore {
    fn read(&self, target: &Object, key: &str) -> Result<OptionValue, OptionError> {
        js_sys::Reflect::get(target, &key.into())
            .and_then(|value| {
                if key == "logger" {
                    crate::logger::from_js_with_options(&value, &self.raw)
                } else {
                    from_js_key(key, &value)
                }
            })
            .map_err(host)
    }
    fn write(&self, target: &Object, key: &str, value: OptionValue) -> Result<(), OptionError> {
        let value = self.cache.convert(&value).map_err(host)?;
        if js_sys::Reflect::set(target, &key.into(), &value).map_err(host)? {
            Ok(())
        } else {
            Err(host(
                js_sys::TypeError::new("Cannot assign to read only option").into(),
            ))
        }
    }
}
impl RawOptionsStore for JsOptionStore {
    fn strict_equal(
        &self,
        left: &OptionValue,
        right: &OptionValue,
    ) -> Result<Option<bool>, OptionError> {
        let left = self.cache.convert(left).map_err(host)?;
        let right = self.cache.convert(right).map_err(host)?;
        Ok(Some(optionStrictEqual(&left, &right)))
    }
    fn validate_key(&self, key: &str) -> Result<Option<bool>, OptionError> {
        if !self.accessors.contains(key) {
            return js_sys::Reflect::has(&self.view, &key.into())
                .map(Some)
                .map_err(host);
        }
        JS_DEFAULTS.with(|defaults| {
            let template = defaults
                .get()
                .expect("host store installed after default initialization");
            optionMembership(template, key).map(Some).map_err(host)
        })
    }
    fn uses_accessor(&self, key: &str) -> Option<bool> {
        Some(self.accessors.contains(key))
    }
    fn options_view(&self) -> Result<Option<OptionValue>, OptionError> {
        from_js(self.view.as_ref()).map(Some).map_err(host)
    }
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        self.read(
            if self.accessors.contains(key) {
                &self.raw
            } else {
                &self.view
            },
            key,
        )
    }
    fn get_raw(&self, key: &str) -> Result<OptionValue, OptionError> {
        self.read(&self.raw, key)
    }
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        self.write(
            if self.accessors.contains(key) {
                &self.raw
            } else {
                &self.view
            },
            key,
            value,
        )
    }
    fn set_raw(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        self.write(&self.raw, key, value)
    }
}
