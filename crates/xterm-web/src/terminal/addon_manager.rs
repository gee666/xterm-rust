//! Browser addon method replacement and lifecycle over private JS records.
//!
//! JavaScript owns the closure references installed on addon instances. Rust
//! owns the manager but no Rust closure captures that manager or a JS instance,
//! avoiding a Rust/JavaScript reference cycle. The tiny bridge does only method
//! installation; ownership checks, order, errors and callbacks stay here.
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};

#[wasm_bindgen(module = "/src/terminal/addon_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = installAddonDispose)]
    fn install_dispose(
        addon: &JsValue,
        entry: &JsValue,
        entries: &Array,
        route: &JsValue,
    ) -> Result<(), JsValue>;
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
    field(entry, "dispose")?
        .dyn_into::<Function>()?
        .call0(&instance)?;
    Ok(())
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
    // Calling the two-argument JS splice preserves captured-index behavior.
    // js-sys's typed overload inserts a third undefined item and is not equivalent.
    field(&entries, "splice")?
        .dyn_into::<Function>()?
        .call2(&entries, &index.into(), &1.into())?;
    Ok(())
}

pub struct BrowserAddonManager {
    entries: Array,
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
            entries: Array::new(),
            route: route.into_js_value(),
        }
    }
    pub fn load_addon(&self, terminal: &JsValue, instance: &JsValue) -> Result<(), JsValue> {
        let original = field(instance, "dispose")?;
        let entry: JsValue = Object::new().into();
        set(&entry, "instance", instance)?;
        set(&entry, "dispose", &original)?;
        set(&entry, "isDisposed", &JsValue::FALSE)?;
        self.entries.push(&entry);
        install_dispose(instance, &entry, &self.entries, &self.route)?;
        field(instance, "activate")?
            .dyn_into::<Function>()?
            .call1(instance, terminal)?;
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        for index in (0..self.entries.length()).rev() {
            let instance = field(&self.entries.get(index), "instance")?;
            field(&instance, "dispose")?
                .dyn_into::<Function>()?
                .call0(&instance)?;
        }
        Ok(())
    }
}
