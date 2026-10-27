// Copyright (c) 2019 The xterm.js authors. MIT license.
// Inspired by Microsoft Corporation's MIT-licensed dependency injection system.
//! Shared service values and constructor argument resolution without unsafe code.
use crate::service_registry::{
    ConstructorMetadata, RegistryError, ServiceId, ServiceKey, ServiceRegistry,
};
use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

/// Null and undefined remain distinct values, but both reject service injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyValue {
    Null,
    Undefined,
}

#[derive(Clone)]
pub struct ServiceValue(Rc<dyn Any>);
impl ServiceValue {
    pub fn new<T: Any>(value: T) -> Self {
        Self(Rc::new(value))
    }
    pub fn shared<T: Any>(value: Rc<T>) -> Self {
        Self(value)
    }
    pub fn downcast<T: Any>(&self) -> Option<Rc<T>> {
        self.0.clone().downcast().ok()
    }
    fn truthy(&self) -> bool {
        let any = self.0.as_ref();
        if any.is::<EmptyValue>() {
            return false;
        }
        if let Some(value) = any.downcast_ref::<bool>() {
            return *value;
        }
        if let Some(value) = any.downcast_ref::<String>() {
            return !value.is_empty();
        }
        if let Some(value) = any.downcast_ref::<&str>() {
            return !value.is_empty();
        }
        if let Some(value) = any.downcast_ref::<f64>() {
            return *value != 0.0 && !value.is_nan();
        }
        if let Some(value) = any.downcast_ref::<f32>() {
            return *value != 0.0 && !value.is_nan();
        }
        macro_rules! nonzero {
            ($($kind:ty),*) => { $(if let Some(value) = any.downcast_ref::<$kind>() { return *value != 0; })* };
        }
        nonzero!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
        true
    }
}

enum StoredValue {
    Strong(ServiceValue),
    Weak(Weak<dyn Any>),
}
impl StoredValue {
    fn resolve(&self) -> Option<ServiceValue> {
        match self {
            Self::Strong(value) => Some(value.clone()),
            Self::Weak(value) => value.upgrade().map(ServiceValue),
        }
    }
}

/// Entries retain insertion order and callbacks may add or replace services.
/// Each callback receives a shared value, with no collection borrow held across it.
#[derive(Default)]
pub struct ServiceCollection {
    entries: RefCell<Vec<(ServiceKey, StoredValue)>>,
}
impl ServiceCollection {
    pub fn new(
        entries: impl IntoIterator<Item = (ServiceKey, ServiceValue)>,
    ) -> Result<Self, RegistryError> {
        let collection = Self::default();
        for (key, value) in entries {
            collection.set_value(key, value)?;
        }
        Ok(collection)
    }
    fn store(&self, key: ServiceKey, value: StoredValue) -> Option<ServiceValue> {
        let mut entries = self.entries.borrow_mut();
        if let Some(entry) = entries.iter_mut().find(|entry| entry.0 == key) {
            let old = entry.1.resolve();
            entry.1 = value;
            old
        } else {
            entries.push((key, value));
            None
        }
    }
    pub fn set_value(
        &self,
        key: ServiceKey,
        value: ServiceValue,
    ) -> Result<Option<ServiceValue>, RegistryError> {
        if value.0.as_ref().type_id() != key.kind() {
            return Err(RegistryError::TypeMismatch(key.name().to_owned()));
        }
        Ok(self.store(key, StoredValue::Strong(value)))
    }
    pub fn set<T: Any>(&self, id: &ServiceId<T>, value: Rc<T>) -> Option<Rc<T>> {
        self.store(id.key(), StoredValue::Strong(ServiceValue::shared(value)))
            .and_then(|old| old.downcast())
    }
    pub fn get_value(&self, key: &ServiceKey) -> Option<ServiceValue> {
        self.entries
            .borrow()
            .iter()
            .find(|entry| entry.0 == *key)
            .and_then(|entry| entry.1.resolve())
    }
    pub fn get<T: Any>(&self, id: &ServiceId<T>) -> Option<Rc<T>> {
        self.get_value(&id.key()).and_then(|value| value.downcast())
    }
    pub fn has(&self, key: &ServiceKey) -> bool {
        self.entries.borrow().iter().any(|entry| entry.0 == *key)
    }
    pub fn for_each(&self, mut callback: impl FnMut(ServiceKey, ServiceValue)) {
        match self.try_for_each::<std::convert::Infallible>(|key, value| {
            callback(key, value);
            Ok(())
        }) {
            Ok(()) => (),
            Err(never) => match never {},
        }
    }
    pub fn try_for_each<E>(
        &self,
        mut callback: impl FnMut(ServiceKey, ServiceValue) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut index = 0;
        loop {
            let next = self
                .entries
                .borrow()
                .get(index)
                .and_then(|(key, value)| value.resolve().map(|value| (key.clone(), value)));
            match next {
                Some((key, value)) => callback(key, value)?,
                None => return Ok(()),
            }
            index += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstantiationError {
    UnknownService {
        constructor: String,
        service: String,
    },
    ArgumentCount {
        constructor: String,
        first_service_position: usize,
        static_count: usize,
    },
}
impl fmt::Display for InstantiationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownService { constructor, service } => write!(f, "[createInstance] {constructor} depends on UNKNOWN service {service}."),
            Self::ArgumentCount { constructor, first_service_position, static_count } => write!(f, "[createInstance] First service dependency of {constructor} at position {first_service_position} conflicts with {static_count} static arguments"),
        }
    }
}
impl std::error::Error for InstantiationError {}

pub struct InstantiationService {
    services: ServiceCollection,
}
impl InstantiationService {
    /// Self-registration uses a weak pointer to prevent a self-owned reference cycle.
    pub fn new(registry: &mut ServiceRegistry) -> Result<Rc<Self>, RegistryError> {
        let id = registry.create_decorator::<Self>("InstantiationService")?;
        Ok(Rc::new_cyclic(|weak: &Weak<Self>| {
            let services = ServiceCollection::default();
            let erased: Weak<dyn Any> = weak.clone();
            services.store(id.key(), StoredValue::Weak(erased));
            Self { services }
        }))
    }
    pub fn set_service<T: Any>(&self, id: &ServiceId<T>, value: Rc<T>) {
        let instance: &dyn Any = value.as_ref();
        if instance
            .downcast_ref::<Self>()
            .is_some_and(|service| std::ptr::eq(service, self))
        {
            // Explicitly re-registering this service must not introduce a self-cycle.
            let weak: Weak<T> = Rc::downgrade(&value);
            self.services.store(id.key(), StoredValue::Weak(weak));
        } else {
            self.services.set(id, value);
        }
    }
    pub fn get_service<T: Any>(&self, id: &ServiceId<T>) -> Option<Rc<T>> {
        self.services.get(id)
    }
    /// A callback may return its own Result. Such failures pass through unchanged.
    pub fn create_instance<T>(
        &self,
        metadata: &ConstructorMetadata,
        mut args: Vec<ServiceValue>,
        constructor: impl FnOnce(Vec<ServiceValue>) -> T,
    ) -> Result<T, InstantiationError> {
        let dependencies = metadata.sorted_dependencies();
        let mut service_args = Vec::with_capacity(dependencies.len());
        for dependency in &dependencies {
            let value = self
                .services
                .get_value(&dependency.id)
                .filter(ServiceValue::truthy)
                .ok_or_else(|| InstantiationError::UnknownService {
                    constructor: metadata.name(),
                    service: dependency.id.name().to_owned(),
                })?;
            service_args.push(value);
        }
        let first = dependencies
            .first()
            .map_or(args.len(), |dependency| dependency.index);
        if first != args.len() {
            return Err(InstantiationError::ArgumentCount {
                constructor: metadata.name(),
                first_service_position: first + 1,
                static_count: args.len(),
            });
        }
        args.extend(service_args);
        Ok(constructor(args))
    }
}
