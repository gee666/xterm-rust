//! A mutable diagnostic field read after the logger returns. This is needed for
//! parsing-state.abort: a custom logger can install a getter or change its value.
use super::LogError;
use crate::options::OptionValue;
use std::{cell::RefCell, fmt, rc::Rc};

/// One live property of the host object delivered to a logger. Accessors remain
/// observable and fallible; the core never converts them into cached booleans.
pub trait LogPropertyHost {
    fn get(&self) -> Result<OptionValue, LogError>;
    fn set(&self, value: &OptionValue) -> Result<(), LogError>;
}

/// A field belongs to one diagnostic object. Native sinks can update it through
/// set; a browser sink attaches its actual property rather than a copied value.
pub struct LogProperty {
    value: RefCell<OptionValue>,
    host: RefCell<Option<Rc<dyn LogPropertyHost>>>,
}
impl LogProperty {
    pub fn new(value: OptionValue) -> Self {
        Self {
            value: RefCell::new(value),
            host: RefCell::new(None),
        }
    }
    pub fn get(&self) -> Result<OptionValue, LogError> {
        let host = self.host.borrow().clone();
        match host {
            Some(host) => host.get(),
            None => Ok(self.value.borrow().clone()),
        }
    }
    pub fn set(&self, value: OptionValue) -> Result<(), LogError> {
        let host = self.host.borrow().clone();
        if let Some(host) = host {
            host.set(&value)
        } else {
            let old = self.value.replace(value);
            drop(old);
            Ok(())
        }
    }
    pub fn install_host(&self, host: Rc<dyn LogPropertyHost>) {
        let old_host = self.host.replace(Some(host));
        // The actual JS property now owns the delivered value. Do not retain a
        // stale native payload, and do not run user destructors under a borrow.
        let old_value = self.value.replace(OptionValue::Undefined);
        drop(old_host);
        drop(old_value);
    }
}
impl fmt::Debug for LogProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogProperty").finish_non_exhaustive()
    }
}
impl PartialEq for LogProperty {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Weak};
    struct Host(Rc<RefCell<OptionValue>>);
    impl LogPropertyHost for Host {
        fn get(&self) -> Result<OptionValue, LogError> {
            Ok(self.0.borrow().clone())
        }
        fn set(&self, value: &OptionValue) -> Result<(), LogError> {
            *self.0.borrow_mut() = value.clone();
            Ok(())
        }
    }
    #[test]
    fn native_and_host_updates_remain_live() {
        let property = LogProperty::new(false.into());
        property.set(true.into()).unwrap();
        assert_eq!(property.get().unwrap(), true.into());
        let value = Rc::new(RefCell::new(false.into()));
        property.install_host(Rc::new(Host(Rc::clone(&value))));
        assert_eq!(property.get().unwrap(), false.into());
        *value.borrow_mut() = "truthy".into();
        assert!(property.get().unwrap().is_truthy());
        property.set(OptionValue::Undefined).unwrap();
        assert_eq!(*value.borrow(), OptionValue::Undefined);
    }
    struct DropSignal {
        property: Weak<LogProperty>,
        calls: Rc<Cell<usize>>,
    }
    impl Drop for DropSignal {
        fn drop(&mut self) {
            let property = self.property.upgrade().unwrap();
            assert_eq!(property.get().unwrap(), true.into());
            property.set(false.into()).unwrap();
            self.calls.set(self.calls.get() + 1);
        }
    }
    #[test]
    fn old_native_payload_drop_can_reenter_after_commit() {
        let property = Rc::new(LogProperty::new(false.into()));
        let calls = Rc::new(Cell::new(0));
        property
            .set(OptionValue::object(DropSignal {
                property: Rc::downgrade(&property),
                calls: Rc::clone(&calls),
            }))
            .unwrap();
        property.set(true.into()).unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(property.get().unwrap(), false.into());
    }
}
