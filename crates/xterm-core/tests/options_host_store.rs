use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    lifecycle::Disposable,
    options::{default_options, OptionError, OptionValue as V, OptionsService, RawOptionsStore},
};

struct Store {
    values: RefCell<BTreeMap<&'static str, V>>,
    hidden: Cell<f64>,
    calls: RefCell<Vec<String>>,
    fail_document: Cell<bool>,
    error: Rc<dyn std::error::Error>,
}
impl Default for Store {
    fn default() -> Self {
        Self {
            values: RefCell::new(default_options(false).into_iter().collect()),
            hidden: Cell::new(7.0),
            calls: RefCell::new(Vec::new()),
            fail_document: Cell::new(false),
            error: Rc::new(std::io::Error::other("host failure")),
        }
    }
}
impl RawOptionsStore for Store {
    fn get(&self, key: &str) -> Result<V, OptionError> {
        self.calls.borrow_mut().push(format!("get:{key}"));
        if key == "toString" {
            let V::Number(rows) = self.values.borrow()["rows"] else {
                panic!()
            };
            return Ok(V::Number(rows + self.hidden.get()));
        }
        Ok(self
            .values
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or(V::Undefined))
    }
    fn set(&self, key: &str, value: V) -> Result<(), OptionError> {
        self.calls.borrow_mut().push(format!("set:{key}"));
        if key == "documentOverride" && self.fail_document.get() {
            return Err(OptionError::Dispatch(EmitError::Callback(
                self.error.clone(),
            )));
        }
        if key == "toString" {
            let V::Number(number) = value else { panic!() };
            self.hidden.set(number);
        } else {
            let known = default_options(false)
                .into_iter()
                .find(|(name, _)| *name == key)
                .map(|(name, _)| name)
                .unwrap();
            self.values.borrow_mut().insert(known, value);
        }
        Ok(())
    }
}
fn service(store: Rc<Store>) -> OptionsService {
    let mut layout: Vec<_> = default_options(false)
        .into_iter()
        .map(|(key, _)| (key.into(), true))
        .collect();
    layout.extend([("toString".into(), false), ("unknown".into(), false)]);
    OptionsService::default().with_raw_store(store, layout)
}

#[test]
fn host_store_retains_receiver_post_assignment_value_and_live_specific_read() {
    let store = Rc::new(Store::default());
    let service = service(store.clone());
    let seen = Rc::new(RefCell::new(Vec::new()));
    let output = seen.clone();
    service
        .on_specific_option_change("toString", move |value| output.borrow_mut().push(value))
        .unwrap();
    assert_eq!(service.get("toString").unwrap(), V::Number(31.0));
    store.calls.borrow_mut().clear();
    service.set("toString", V::Number(3.0)).unwrap();
    assert_eq!(*seen.borrow(), [V::Number(27.0)]);
    assert_eq!(
        *store.calls.borrow(),
        ["get:toString", "set:toString", "get:toString"]
    );
    service.set("rows", V::Number(30.0)).unwrap();
    assert_eq!(service.get("toString").unwrap(), V::Number(33.0));
    assert_eq!(service.keys().len(), 48);
    assert_eq!(service.property_layout().len(), 50);
}

#[test]
fn invalid_inherited_accessor_rejects_before_host_getter() {
    let store = Rc::new(Store::default());
    let service = service(store.clone());
    store.calls.borrow_mut().clear();
    assert_eq!(
        service.get("unknown").unwrap_err().to_string(),
        "No option with key \"unknown\""
    );
    assert!(service.set("unknown", V::Null).is_err());
    assert!(store.calls.borrow().is_empty());
}

#[test]
fn host_raw_writes_bypass_validation_and_events() {
    let store = Rc::new(Store::default());
    let service = service(store);
    service
        .on_option_change()
        .subscribe(|_| panic!("raw write emitted"));
    service.set_raw("rows", V::Number(-5.0)).unwrap();
    assert_eq!(service.get("rows").unwrap(), V::Number(-5.0));
    assert!(service.set("cursorWidth", V::Number(0.0)).is_err());
}

#[test]
fn cleanup_is_link_then_document_once_and_retains_original_host_error() {
    let store = Rc::new(Store::default());
    let service = service(store.clone());
    service.set_raw("documentOverride", V::object(())).unwrap();
    service.set_raw("linkHandler", V::object(())).unwrap();
    store.calls.borrow_mut().clear();
    store.fail_document.set(true);
    service.dispose();
    let OptionError::Dispatch(EmitError::Callback(actual)) = service.take_dispose_error().unwrap()
    else {
        panic!()
    };
    assert!(Rc::ptr_eq(&actual, &store.error));
    assert_eq!(
        *store.calls.borrow(),
        ["set:linkHandler", "set:documentOverride"]
    );
    service.dispose();
    service.try_dispose().unwrap();
    assert_eq!(
        *store.calls.borrow(),
        ["set:linkHandler", "set:documentOverride"]
    );
    assert_eq!(store.values.borrow()["linkHandler"], V::Null);
    assert!(matches!(
        store.values.borrow()["documentOverride"],
        V::Object(_)
    ));
}

#[test]
fn replaced_payload_destructor_can_reenter_options_without_borrow_panics() {
    struct Probe {
        service: std::rc::Weak<OptionsService>,
        seen: Rc<Cell<usize>>,
    }
    impl Drop for Probe {
        fn drop(&mut self) {
            if let Some(service) = self.service.upgrade() {
                assert_eq!(service.get("logger").unwrap(), V::Null);
                service.set("fontSize", V::Number(16.0)).unwrap();
                self.seen.set(self.seen.get() + 1);
            }
        }
    }
    for raw in [false, true] {
        let service = Rc::new(OptionsService::default());
        let seen = Rc::new(Cell::new(0));
        service
            .set(
                "logger",
                V::object(Probe {
                    service: Rc::downgrade(&service),
                    seen: seen.clone(),
                }),
            )
            .unwrap();
        if raw {
            service.set_raw("logger", V::Null).unwrap();
        } else {
            service.set("logger", V::Null).unwrap();
        }
        assert_eq!(seen.get(), 1);
        assert_eq!(service.get("fontSize").unwrap(), V::Number(16.0));
    }
}

#[test]
fn native_constructor_preserves_object_prototype_data_names() {
    let service = OptionsService::new(
        false,
        [
            ("toString", V::Number(7.0)),
            ("constructor", V::Bool(false)),
        ],
    );
    assert_eq!(service.get("toString").unwrap(), V::Number(7.0));
    assert_eq!(service.get("constructor").unwrap(), V::Bool(false));
    assert_eq!(service.keys().len(), 50);
    service.set("toString", V::Number(8.0)).unwrap();
    assert_eq!(service.get("toString").unwrap(), V::Number(8.0));
}
