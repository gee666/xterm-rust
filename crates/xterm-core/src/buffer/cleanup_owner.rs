// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Adapter lifetime is independent of an outer Buffer borrow at cancellation.
use std::{any::Any, cell::RefCell, rc::Rc};

struct Owner {
    token: Rc<()>,
    resource: Box<dyn Any>,
}

#[derive(Default)]
pub(crate) struct CleanupOwnerSlot {
    owner: RefCell<Option<Owner>>,
}
impl CleanupOwnerSlot {
    pub(crate) fn is_owned_by(&self, token: &Rc<()>) -> bool {
        self.owner
            .borrow()
            .as_ref()
            .is_some_and(|owner| Rc::ptr_eq(&owner.token, token))
    }
    /// Publish ownership before releasing an old adapter. Host cancellation may
    /// reenter and install another owner, which must not be overwritten afterward.
    pub(crate) fn install(&self, token: &Rc<()>, resource: Box<dyn Any>) {
        let previous = self.owner.replace(Some(Owner {
            token: Rc::clone(token),
            resource,
        }));
        drop(previous);
    }
    /// Only the installing registration can detach this adapter. Moving the box
    /// out lets the caller release it after Set and slot borrows have ended.
    pub(crate) fn take_if_owned(&self, token: &Rc<()>) -> Option<Box<dyn Any>> {
        let previous = {
            let mut owner = self.owner.borrow_mut();
            if owner
                .as_ref()
                .is_some_and(|owner| Rc::ptr_eq(&owner.token, token))
            {
                owner.take()
            } else {
                None
            }
        };
        previous.map(|owner| owner.resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct OnDrop(Option<Box<dyn FnOnce()>>);
    impl Drop for OnDrop {
        fn drop(&mut self) {
            self.0.take().unwrap()();
        }
    }
    #[test]
    fn foreign_registration_cannot_release_an_owner() {
        let slot = CleanupOwnerSlot::default();
        let a = Rc::new(());
        let b = Rc::new(());
        slot.install(&a, Box::new(9u32));
        assert!(slot.take_if_owned(&b).is_none());
        assert!(slot.is_owned_by(&a));
        assert_eq!(
            *slot.take_if_owned(&a).unwrap().downcast::<u32>().unwrap(),
            9
        );
        assert!(!slot.is_owned_by(&a));
    }
    #[test]
    fn released_destructor_can_reenter_without_overwriting_its_replacement() {
        let slot = Rc::new(CleanupOwnerSlot::default());
        let first = Rc::new(());
        let second = Rc::new(());
        let nested = Rc::new(());
        let weak = Rc::downgrade(&slot);
        let token = Rc::clone(&nested);
        slot.install(
            &first,
            Box::new(OnDrop(Some(Box::new(move || {
                weak.upgrade().unwrap().install(&token, Box::new(3u32));
            })))),
        );
        slot.install(&second, Box::new(2u32));
        assert!(slot.is_owned_by(&nested));
        assert!(!slot.is_owned_by(&second));
        assert_eq!(
            *slot
                .take_if_owned(&nested)
                .unwrap()
                .downcast::<u32>()
                .unwrap(),
            3
        );
    }
}
