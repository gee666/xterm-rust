use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::public_api::{AddonError, AddonHandle, AddonManager, TerminalAddon};

type Callback = dyn Fn() -> Result<(), &'static str>;
type Activation = dyn Fn(&str) -> Result<(), &'static str>;
struct Addon {
    activate: Box<Activation>,
    dispose: Box<Callback>,
}
impl TerminalAddon<String> for Addon {
    type Error = &'static str;
    fn activate(&self, terminal: &String) -> Result<(), Self::Error> {
        (self.activate)(terminal)
    }
    fn dispose(&self) -> Result<(), Self::Error> {
        (self.dispose)()
    }
}
fn addon(dispose: impl Fn() -> Result<(), &'static str> + 'static) -> Rc<Addon> {
    Rc::new(Addon {
        activate: Box::new(|_| Ok(())),
        dispose: Box::new(dispose),
    })
}

#[test]
fn upstream_activation_receives_terminal() {
    let manager = AddonManager::new();
    let called = Rc::new(Cell::new(false));
    let flag = called.clone();
    manager
        .load_addon(
            &"foo".to_owned(),
            Rc::new(Addon {
                activate: Box::new(move |terminal| {
                    assert_eq!(terminal, "foo");
                    flag.set(true);
                    Ok(())
                }),
                dispose: Box::new(|| Ok(())),
            }),
        )
        .unwrap();
    assert!(called.get());
}

#[test]
fn upstream_dispose_all_loaded_addons() {
    let manager = AddonManager::new();
    let calls = Rc::new(Cell::new(0));
    for _ in 0..3 {
        let calls = calls.clone();
        manager
            .load_addon(
                &String::new(),
                addon(move || {
                    calls.set(calls.get() + 1);
                    Ok(())
                }),
            )
            .unwrap();
    }
    assert_eq!(manager.len(), 3);
    manager.dispose().unwrap();
    assert_eq!(calls.get(), 3);
    assert!(manager.is_empty());
}

#[test]
fn reverse_order_and_direct_idempotent_disposal() {
    let manager = AddonManager::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut handles = Vec::new();
    for label in ["a", "b", "c"] {
        let log = log.clone();
        handles.push(
            manager
                .load_addon(
                    &String::new(),
                    addon(move || {
                        log.borrow_mut().push(label);
                        Ok(())
                    }),
                )
                .unwrap(),
        );
    }
    handles[1].dispose().unwrap();
    handles[1].dispose().unwrap();
    manager.dispose().unwrap();
    manager.dispose().unwrap();
    assert_eq!(*log.borrow(), ["b", "c", "a"]);
}

#[test]
fn activation_failure_keeps_registration_for_cleanup() {
    let manager = AddonManager::new();
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let result = manager.load_addon(
        &String::new(),
        Rc::new(Addon {
            activate: Box::new(|_| Err("activate")),
            dispose: Box::new(move || {
                count.set(count.get() + 1);
                Ok(())
            }),
        }),
    );
    assert!(matches!(result, Err(AddonError::Callback("activate"))));
    assert_eq!(manager.len(), 1);
    manager.dispose().unwrap();
    assert_eq!(calls.get(), 1);
}

#[test]
fn cleanup_failure_marks_before_call_and_retains_failed_record() {
    let manager = AddonManager::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    for (label, fail) in [("a", false), ("b", true), ("c", false)] {
        let log = log.clone();
        manager
            .load_addon(
                &String::new(),
                addon(move || {
                    log.borrow_mut().push(label);
                    if fail {
                        Err("cleanup")
                    } else {
                        Ok(())
                    }
                }),
            )
            .unwrap();
    }
    assert_eq!(manager.dispose(), Err(AddonError::Callback("cleanup")));
    assert_eq!(manager.len(), 2);
    assert_eq!(*log.borrow(), ["c", "b"]);
    manager.dispose().unwrap();
    assert_eq!(*log.borrow(), ["c", "b", "a"]);
    assert_eq!(manager.len(), 1);
    manager.dispose().unwrap();
    assert_eq!(*log.borrow(), ["c", "b", "a"]);
}

#[test]
fn recursive_self_disposal_runs_once_without_borrow_panic() {
    let manager = AddonManager::new();
    let slot: Rc<RefCell<Option<AddonHandle<String, &'static str>>>> = Rc::default();
    let captured = slot.clone();
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let handle = manager
        .load_addon(
            &String::new(),
            addon(move || {
                calls.set(calls.get() + 1);
                captured.borrow().as_ref().unwrap().dispose().unwrap();
                Ok(())
            }),
        )
        .unwrap();
    *slot.borrow_mut() = Some(handle.clone());
    handle.dispose().unwrap();
    assert_eq!(count.get(), 1);
    assert!(manager.is_empty());
    slot.borrow_mut().take();
}

#[test]
fn callbacks_can_load_addons_without_borrowing_registry() {
    let manager = AddonManager::new();
    let captured = manager.clone();
    manager
        .load_addon(
            &String::new(),
            addon(move || {
                captured
                    .load_addon(&String::new(), addon(|| Ok(())))
                    .unwrap();
                Ok(())
            }),
        )
        .unwrap();
    manager.dispose().unwrap();
    assert_eq!(manager.len(), 1);
    manager.dispose().unwrap();
    assert!(manager.is_empty());
}
