// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{event_error, event_function, set, subscription_event, SubscriptionEvent};
use crate::terminal::Terminal;
use js_sys::{Array, Object};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_addons::{
    native_status::HeadlessStatusTerminal,
    program_status::{ProgramStatusAddon as NativeAddon, StatusKind, StatusRecord, StatusState},
};
use xterm_core::lifecycle::Disposable;

type Snapshot = Rc<[StatusRecord]>;
#[derive(Default)]
struct SnapshotCache {
    native: Option<Snapshot>,
    js: JsValue,
    records: Vec<(Rc<()>, JsValue)>,
}
impl SnapshotCache {
    fn get(&mut self, snapshot: &Snapshot) -> Result<JsValue, JsValue> {
        if self
            .native
            .as_ref()
            .is_some_and(|old| Rc::ptr_eq(old, snapshot))
        {
            return Ok(self.js.clone());
        }
        let array = Array::new();
        let mut identities = Vec::with_capacity(snapshot.len());
        for record in snapshot.iter() {
            if let Some((identity, object)) = self
                .records
                .iter()
                .find(|(identity, _)| Rc::ptr_eq(identity, record.identity()))
            {
                array.push(object);
                identities.push((identity.clone(), object.clone()));
                continue;
            }
            let object = Object::new();
            set(&object, "id", &record.id.clone().into())?;
            set(
                &object,
                "state",
                &match record.state {
                    StatusState::Idle => "idle",
                    StatusState::Working => "working",
                    StatusState::Done => "done",
                    StatusState::Blocked => "blocked",
                    StatusState::Error => "error",
                }
                .into(),
            )?;
            if let Some(kind) = record.kind {
                set(
                    &object,
                    "kind",
                    &match kind {
                        StatusKind::Permission => "permission",
                        StatusKind::Question => "question",
                        StatusKind::Auth => "auth",
                    }
                    .into(),
                )?;
            }
            if let Some(progress) = record.progress {
                set(&object, "progress", &JsValue::from(progress))?;
            }
            for (key, value) in [
                ("app", &record.app),
                ("effectiveApp", &record.effective_app),
                ("title", &record.title),
                ("message", &record.message),
            ] {
                if let Some(value) = value {
                    set(&object, key, &value.clone().into())?;
                }
            }
            let object: JsValue = Object::freeze(&object).into();
            array.push(&object);
            identities.push((record.identity().clone(), object));
        }
        self.records = identities;
        self.js = Object::freeze(&Object::from(array)).into();
        self.native = Some(snapshot.clone());
        Ok(self.js.clone())
    }
}

#[wasm_bindgen]
pub struct ProgramStatusAddon {
    addon: NativeAddon,
    adapter: RefCell<Option<Rc<HeadlessStatusTerminal>>>,
    snapshot: Rc<RefCell<SnapshotCache>>,
    event: SubscriptionEvent,
}
#[wasm_bindgen]
impl ProgramStatusAddon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let addon = NativeAddon::new();
        let snapshot = Rc::new(RefCell::new(SnapshotCache::default()));
        let cache = snapshot.clone();
        let event = subscription_event(addon.on_change(), move |snapshot| {
            cache.borrow_mut().get(snapshot)
        });
        Self {
            addon,
            adapter: RefCell::new(None),
            snapshot,
            event,
        }
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let adapter = Rc::new(
            HeadlessStatusTerminal::from_terminal(terminal.shared_native())
                .map_err(|e| js_sys::Error::new(&e.to_string()))?,
        );
        self.addon
            .activate(adapter.clone())
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        *self.adapter.borrow_mut() = Some(adapter);
        Ok(())
    }
    #[wasm_bindgen(getter)]
    pub fn records(&self) -> Result<JsValue, JsValue> {
        self.snapshot.borrow_mut().get(&self.addon.records())
    }
    #[wasm_bindgen(getter,js_name=onChange)]
    pub fn on_change(&self) -> JsValue {
        event_function(&self.event)
    }
    pub fn clear(&self, id: Option<String>) -> Result<(), JsValue> {
        self.addon
            .clear(id.as_deref().unwrap_or(""))
            .map_err(event_error)
    }
    #[wasm_bindgen(js_name=processExited)]
    pub fn process_exited(&self) -> Result<(), JsValue> {
        self.addon.process_exited().map_err(event_error)
    }
    #[wasm_bindgen(js_name=promptStarted)]
    pub fn prompt_started(&self) -> Result<(), JsValue> {
        self.addon.prompt_started().map_err(event_error)
    }
    pub fn dispose(&self) {
        self.addon.dispose();
        let current = self.addon.records();
        let mut cache = self.snapshot.borrow_mut();
        if !cache
            .native
            .as_ref()
            .is_some_and(|old| Rc::ptr_eq(old, &current))
        {
            // A saved onChange function can outlive this wrapper. Do not let
            // its conversion cache retain records that disposal removed.
            cache.native = None;
            cache.js = JsValue::UNDEFINED;
            cache.records.clear();
        }
    }
}
impl Drop for ProgramStatusAddon {
    fn drop(&mut self) {
        self.dispose();
    }
}
impl Default for ProgramStatusAddon {
    fn default() -> Self {
        Self::new()
    }
}
