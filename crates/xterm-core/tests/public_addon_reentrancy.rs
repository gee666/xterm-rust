use std::{cell::RefCell, rc::Rc};
use xterm_core::public_api::{AddonError, AddonManager, TerminalAddon};

struct Addon(Box<dyn Fn() -> Result<(), &'static str>>);
impl TerminalAddon<()> for Addon {
    type Error = &'static str;
    fn activate(&self, _: &()) -> Result<(), Self::Error> {
        Ok(())
    }
    fn dispose(&self) -> Result<(), Self::Error> {
        (self.0)()
    }
}

#[test]
fn nested_manager_disposal_preserves_upstream_invalidated_index_error() {
    let manager = AddonManager::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    for label in ["a", "b"] {
        let log = log.clone();
        manager
            .load_addon(
                &(),
                Rc::new(Addon(Box::new(move || {
                    log.borrow_mut().push(label);
                    Ok(())
                }))),
            )
            .unwrap();
    }
    let weak = manager.downgrade();
    let captured = log.clone();
    manager
        .load_addon(
            &(),
            Rc::new(Addon(Box::new(move || {
                captured.borrow_mut().push("c");
                weak.upgrade().unwrap().dispose().unwrap();
                Ok(())
            }))),
        )
        .unwrap();
    assert_eq!(manager.dispose(), Err(AddonError::TraversalInvalidated));
    assert_eq!(*log.borrow(), ["c", "b", "a"]);
    assert_eq!(manager.len(), 1);
    manager.dispose().unwrap();
    assert_eq!(*log.borrow(), ["c", "b", "a"]);
}

#[test]
fn removed_lower_index_leaves_disposed_registration_at_captured_splice_index() {
    let manager = AddonManager::new();
    let a = manager
        .load_addon(&(), Rc::new(Addon(Box::new(|| Ok(())))))
        .unwrap();
    let slot = Rc::new(a);
    let weak = Rc::downgrade(&slot);
    manager
        .load_addon(
            &(),
            Rc::new(Addon(Box::new(move || {
                weak.upgrade().unwrap().dispose().unwrap();
                Ok(())
            }))),
        )
        .unwrap();
    manager.dispose().unwrap();
    assert_eq!(manager.len(), 1);
    manager.dispose().unwrap();
    assert_eq!(manager.len(), 1);
}

#[test]
fn weak_callback_route_does_not_keep_manager_alive() {
    let manager = AddonManager::<(), &'static str>::new();
    let weak = manager.downgrade();
    drop(manager);
    assert!(weak.upgrade().is_none());
}
