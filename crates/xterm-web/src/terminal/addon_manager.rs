//! Browser addon method replacement and lifecycle over private JS records.
//!
//! JavaScript owns closure references installed on addon instances. The source
//! facade also owns the addon array via a JS WeakMap. Rust keeps only a WeakRef
//! after that transfer, avoiding manager -> addon._terminal -> JS root cycles.
//! Raw standalone Rust managers own their array until `release_js_owner` is used.
use js_sys::{Array, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{closure::Closure, prelude::*};

#[wasm_bindgen(module = "/src/terminal/addon_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = installAddonDispose)]
    fn install_dispose(
        addon: &JsValue,
        entry: &JsValue,
        entries: &Array,
        route: &JsValue,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(js_name = emptyAddonEntries)]
    fn empty_entries() -> Array;
    #[wasm_bindgen(catch, js_name = weakAddonOwner)]
    fn weak_owner(entries: &Array) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = readAddonOwner)]
    fn read_owner(owner: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = pushAddonEntry)]
    fn push_entry(entries: &Array, entry: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = addonDisposeProperty)]
    fn dispose_property(instance: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = addonRecord)]
    fn addon_record(instance: &JsValue, dispose: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = activateAddon)]
    fn activate_addon(instance: &JsValue, terminal: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = disposeAddon)]
    fn dispose_addon(instance: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = applyAddonDispose)]
    fn apply_dispose(dispose: &JsValue, instance: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = removeAddon)]
    fn remove_addon(entries: &Array, index: u32) -> Result<(), JsValue>;
}

fn field(object: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    Reflect::get(object, &key.into())
}
fn set(object: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if Reflect::set(object, &key.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new("Cannot assign addon lifecycle property").into())
    }
}
fn call_dispose(entry: &JsValue) -> Result<(), JsValue> {
    let instance = field(entry, "instance")?;
    apply_dispose(&field(entry, "dispose")?, &instance)
}
fn dispose_entry(entry: JsValue, entries: Array) -> Result<(), JsValue> {
    if field(&entry, "isDisposed")?.as_bool() == Some(true) {
        return Ok(());
    }
    let index = (0..entries.length())
        .find(|&index| entries.get(index) == entry)
        .ok_or_else(|| js_sys::Error::new("Could not dispose an addon that has not been loaded"))?;
    set(&entry, "isDisposed", &JsValue::TRUE)?;
    call_dispose(&entry)?;
    // js-sys's typed splice overload inserts a third undefined item; use the
    // actual two-argument operation at the Rust-captured index instead.
    remove_addon(&entries, index)
}

#[derive(Clone)]
enum EntriesOwner {
    Rust(Array),
    JavaScript(JsValue),
}

pub struct BrowserAddonManager {
    entries: RefCell<EntriesOwner>,
    route: JsValue,
}
impl Default for BrowserAddonManager {
    fn default() -> Self {
        Self::new()
    }
}
impl BrowserAddonManager {
    pub fn new() -> Self {
        let route = Closure::wrap(
            Box::new(dispose_entry) as Box<dyn FnMut(JsValue, Array) -> Result<(), JsValue>>
        );
        Self {
            entries: RefCell::new(EntriesOwner::Rust(empty_entries())),
            route: route.into_js_value(),
        }
    }
    fn entries(&self) -> Result<Array, JsValue> {
        let owner = self.entries.borrow().clone();
        match owner {
            EntriesOwner::Rust(entries) => Ok(entries),
            EntriesOwner::JavaScript(owner) => {
                read_owner(&owner)?.dyn_into::<Array>().map_err(|_| {
                    js_sys::Error::new("Addon JavaScript owner is no longer available").into()
                })
            }
        }
    }
    /// The compatible JS facade must retain the returned array in a WeakMap
    /// keyed by its public Terminal. A Rust-held strong array would retain that
    /// same Terminal through ordinary addons' `_terminal` fields.
    pub fn release_js_owner(&self) -> Result<Array, JsValue> {
        let entries = self.entries()?;
        let weak = weak_owner(&entries)?;
        *self.entries.borrow_mut() = EntriesOwner::JavaScript(weak);
        Ok(entries)
    }
    pub fn load_addon(&self, terminal: &JsValue, instance: &JsValue) -> Result<(), JsValue> {
        let entries = self.entries()?;
        let original = dispose_property(instance)?;
        // Source uses an object literal, so inherited setters must not intercept
        // record creation. A primitive addon still reaches the later assignment.
        let entry = addon_record(instance, &original)?;
        push_entry(&entries, &entry)?;
        install_dispose(instance, &entry, &entries, &self.route)?;
        activate_addon(instance, terminal)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        let entries = self.entries()?;
        for index in (0..entries.length()).rev() {
            let instance = field(&entries.get(index), "instance")?;
            dispose_addon(&instance)?;
        }
        Ok(())
    }
}
