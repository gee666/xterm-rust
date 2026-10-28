//! Mutable native data bags and host property access. Native bags contain data
//! properties; prototypes/accessors belong to explicit host implementations.
use super::{CoercionError, OptionError, OptionObject, OptionValue};
use std::{any::Any, cell::RefCell, rc::Rc};

type Getter = dyn Fn(&dyn Any, &str) -> Result<OptionValue, OptionError>;

pub trait PropertyAccess {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError>;
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError>;
    fn delete(&self, key: &str) -> Result<bool, OptionError>;
    fn keys(&self) -> Result<Vec<String>, OptionError>;
}

pub(super) enum PropertyStorage {
    ReadOnly(Box<Getter>),
    Mutable(Rc<dyn PropertyAccess>),
}
impl PropertyStorage {
    pub(super) fn get(&self, payload: &dyn Any, key: &str) -> Result<OptionValue, OptionError> {
        match self {
            Self::ReadOnly(get) => get(payload, key),
            Self::Mutable(access) => access.get(key),
        }
    }
}
impl OptionObject {
    pub fn set_property(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        match &self.properties {
            PropertyStorage::Mutable(access) => access.set(key, value),
            PropertyStorage::ReadOnly(_) => Err(CoercionError::new(
                "native adapter does not provide property assignment",
            )
            .into()),
        }
    }
    pub fn delete_property(&self, key: &str) -> Result<bool, OptionError> {
        match &self.properties {
            PropertyStorage::Mutable(access) => access.delete(key),
            PropertyStorage::ReadOnly(_) => {
                Err(CoercionError::new("native adapter does not provide property deletion").into())
            }
        }
    }
    pub fn property_keys(&self) -> Result<Vec<String>, OptionError> {
        match &self.properties {
            PropertyStorage::Mutable(access) => access.keys(),
            PropertyStorage::ReadOnly(_) => Err(CoercionError::new(
                "native adapter does not provide property enumeration",
            )
            .into()),
        }
    }
    pub fn attach_property_access(
        object: &mut Rc<Self>,
        access: Rc<dyn PropertyAccess>,
    ) -> Result<(), OptionError> {
        let object = Rc::get_mut(object).ok_or_else(|| {
            CoercionError::new("property access must be attached before sharing the object")
        })?;
        object.properties = PropertyStorage::Mutable(access);
        Ok(())
    }
}

#[derive(Default)]
pub struct NativeProperties {
    entries: RefCell<Vec<(String, OptionValue)>>,
}
impl NativeProperties {
    pub fn new<K: Into<String>>(entries: impl IntoIterator<Item = (K, OptionValue)>) -> Rc<Self> {
        let mut values: Vec<(String, OptionValue)> = Vec::new();
        for (key, value) in entries {
            let key = key.into();
            if let Some((_, current)) = values.iter_mut().find(|(name, _)| name == &key) {
                *current = value;
            } else {
                values.push((key, value));
            }
        }
        Rc::new(Self {
            entries: RefCell::new(values),
        })
    }
    pub fn snapshot(&self) -> Vec<(String, OptionValue)> {
        self.entries.borrow().clone()
    }

    fn contains_self(&self, value: &OptionValue, seen: &mut Vec<*const Self>) -> bool {
        let OptionValue::Object(object) = value else {
            return false;
        };
        let Some(properties) = object.payload::<Rc<Self>>() else {
            return false;
        };
        if std::ptr::eq(self, properties.as_ref()) {
            return true;
        }
        let pointer = Rc::as_ptr(properties);
        if seen.contains(&pointer) {
            return false;
        }
        seen.push(pointer);
        properties
            .snapshot()
            .iter()
            .any(|(_, value)| self.contains_self(value, seen))
    }
}
impl PropertyAccess for NativeProperties {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        Ok(self
            .entries
            .borrow()
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
            .unwrap_or(OptionValue::Undefined))
    }
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        // These native bags are acyclic data, not a garbage-collected JS heap.
        // Reject a cycle rather than leak Rc state. Host objects retain their
        // own graph semantics through PropertyAccess.
        if self.contains_self(&value, &mut Vec::new()) {
            return Err(
                CoercionError::new("cyclic native data properties require a host object").into(),
            );
        }
        let previous = {
            let mut entries = self.entries.borrow_mut();
            if let Some((_, current)) = entries.iter_mut().find(|(name, _)| name == key) {
                Some(std::mem::replace(current, value))
            } else {
                entries
                    .try_reserve(1)
                    .map_err(|_| CoercionError::new("unable to allocate native option property"))?;
                entries.push((key.into(), value));
                None
            }
        };
        drop(previous);
        Ok(())
    }
    fn delete(&self, key: &str) -> Result<bool, OptionError> {
        let previous = {
            let mut entries = self.entries.borrow_mut();
            entries
                .iter()
                .position(|(name, _)| name == key)
                .map(|index| entries.remove(index))
        };
        drop(previous);
        Ok(true)
    }
    fn keys(&self) -> Result<Vec<String>, OptionError> {
        let entries = self.entries.borrow();
        let mut indexed = Vec::new();
        let mut named = Vec::new();
        for (key, _) in entries.iter() {
            match key
                .parse::<u32>()
                .ok()
                .filter(|index| *index < u32::MAX && index.to_string() == *key)
            {
                Some(index) => indexed.push((index, key.clone())),
                None => named.push(key.clone()),
            }
        }
        indexed.sort_by_key(|(index, _)| *index);
        Ok(indexed
            .into_iter()
            .map(|(_, key)| key)
            .chain(named)
            .collect())
    }
}
