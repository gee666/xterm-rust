// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::handler::registry::HandlerRegistry;

#[test]
fn duplicate_tokens_remove_first_identity_per_disposal_not_equal_values() {
    let registry = HandlerRegistry::<i32, usize>::new();
    let shared = Rc::new(RefCell::new(7));
    let other = Rc::new(RefCell::new(7));
    let token = registry.register(1, Rc::clone(&shared));
    registry.register(1, Rc::clone(&other));
    registry.register(1, Rc::clone(&shared));
    token.dispose();
    let active = registry.get(&1);
    assert_eq!(active.borrow().len(), 2);
    assert!(Rc::ptr_eq(&active.borrow()[0], &other));
    assert!(Rc::ptr_eq(&active.borrow()[1], &shared));
    token.dispose();
    token.dispose();
    assert_eq!(active.borrow().len(), 1);
    assert!(Rc::ptr_eq(&active.borrow()[0], &other));
}

#[test]
fn clearing_lookup_detaches_only_lookup_and_weak_token_keeps_no_callback_alive() {
    let registry = HandlerRegistry::<i32, usize>::new();
    let handler = Rc::new(RefCell::new(7));
    let weak = Rc::downgrade(&handler);
    let token = registry.register(1, handler);
    let active = registry.get(&1);
    registry.clear(&1);
    assert!(registry.get(&1).borrow().is_empty());
    assert_eq!(active.borrow().len(), 1);
    assert!(weak.upgrade().is_some());
    drop(active);
    assert!(weak.upgrade().is_none());
    token.dispose();
}

#[test]
fn weak_registrar_allows_live_registration_but_never_extends_parser_lifetime() {
    let registry = HandlerRegistry::<i32, usize>::new();
    let registrar = registry.registrar();
    let cloned = registrar.clone();
    assert_eq!(registrar.clear(&123), Some(()));
    registrar.register(1, Rc::new(RefCell::new(7))).unwrap();
    let active = registry.get(&1);
    cloned.register(1, Rc::new(RefCell::new(8))).unwrap();
    assert_eq!(active.borrow().len(), 2);
    registry.clear_all();
    registrar.register(2, Rc::new(RefCell::new(9))).unwrap();
    assert_eq!(*registry.get(&2).borrow()[0].borrow(), 9);
    drop(registry);
    assert!(registrar.register(3, Rc::new(RefCell::new(10))).is_none());
    assert_eq!(registrar.clear(&2), None);
}

struct SelfToken {
    token: Option<DisposableHandle>,
    drops: Rc<Cell<usize>>,
}
impl Drop for SelfToken {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn callback_can_own_its_token_without_a_strong_reference_cycle() {
    let registry = HandlerRegistry::<u8, SelfToken>::new();
    let drops = Rc::new(Cell::new(0));
    let handler = Rc::new(RefCell::new(SelfToken {
        token: None,
        drops: Rc::clone(&drops),
    }));
    let token = registry.register(1, Rc::clone(&handler));
    handler.borrow_mut().token = Some(token);
    drop(handler);
    assert_eq!(drops.get(), 0);
    registry.clear_all();
    assert_eq!(drops.get(), 1);
}
