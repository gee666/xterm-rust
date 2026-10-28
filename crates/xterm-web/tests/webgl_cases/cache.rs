// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. CharAtlasCache ownership and partial-state assertions.
use crate::webgl_support::config;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_web::webgl::cache::{AtlasCache, AtlasResource, CacheError};

type DisposeCallback = Rc<dyn Fn()>;

#[derive(Default)]
struct Atlas {
    calls: Cell<usize>,
    fail: Cell<bool>,
    on_dispose: RefCell<Option<DisposeCallback>>,
}
impl AtlasResource for Atlas {
    type Error = &'static str;
    fn dispose(&self) -> Result<(), Self::Error> {
        self.calls.set(self.calls.get() + 1);
        let callback = self.on_dispose.borrow().clone();
        if let Some(callback) = callback {
            callback();
        }
        if self.fail.replace(false) {
            Err("dispose failure")
        } else {
            Ok(())
        }
    }
}
fn create(_: Rc<xterm_web::webgl::config::AtlasConfig>) -> Result<Rc<Atlas>, &'static str> {
    Ok(Rc::new(Atlas::default()))
}

#[test]
fn same_owner_and_config_reuses_without_running_the_factory() {
    let cache = AtlasCache::<(), Atlas>::new();
    let owner = Rc::new(());
    let first = cache.acquire(&owner, config(), create).unwrap();
    let second = cache
        .acquire(&owner, config(), |_| {
            panic!("existing owner must reuse atlas")
        })
        .unwrap();
    assert!(Rc::ptr_eq(&first, &second));
    assert_eq!(first.calls.get(), 0);
}

#[test]
fn equal_configs_share_and_only_final_owner_disposes_in_either_order() {
    for reverse in [false, true] {
        let cache = AtlasCache::<(), Atlas>::new();
        let a = Rc::new(());
        let b = Rc::new(());
        let first = cache.acquire(&a, config(), create).unwrap();
        let shared = cache
            .acquire(&b, config(), |_| panic!("equal config must share"))
            .unwrap();
        assert!(Rc::ptr_eq(&first, &shared));
        let owners = if reverse { [&b, &a] } else { [&a, &b] };
        cache.remove_terminal(&Rc::downgrade(owners[0])).unwrap();
        assert_eq!(first.calls.get(), 0);
        cache.remove_terminal(&Rc::downgrade(owners[1])).unwrap();
        assert_eq!(first.calls.get(), 1);
        cache.remove_terminal(&Rc::downgrade(owners[1])).unwrap();
        assert_eq!(first.calls.get(), 1);
        let reacquired = cache.acquire(&a, config(), create).unwrap();
        assert!(!Rc::ptr_eq(&first, &reacquired));
    }
}

#[test]
fn config_change_releases_one_owner_without_freeing_anothers_atlas() {
    let cache = AtlasCache::<(), Atlas>::new();
    let a = Rc::new(());
    let b = Rc::new(());
    let first = cache.acquire(&a, config(), create).unwrap();
    cache.acquire(&b, config(), create).unwrap();
    let mut changed = config();
    changed.dimensions.device_pixel_ratio = 2.0;
    let second = cache.acquire(&a, changed, create).unwrap();
    assert!(!Rc::ptr_eq(&first, &second));
    assert_eq!(first.calls.get(), 0);
    cache.remove_terminal(&Rc::downgrade(&b)).unwrap();
    assert_eq!(first.calls.get(), 1);
    assert_eq!(second.calls.get(), 0);
    cache.remove_terminal(&Rc::downgrade(&a)).unwrap();
    assert_eq!(second.calls.get(), 1);
}

#[test]
fn single_owner_change_disposes_before_creation_and_weak_owner_does_not_leak_terminal() {
    let cache = AtlasCache::<(), Atlas>::new();
    let owner = Rc::new(());
    let weak = Rc::downgrade(&owner);
    let first = cache.acquire(&owner, config(), create).unwrap();
    let mut changed = config();
    changed.options.font_size = 16.0;
    let second = cache
        .acquire(&owner, changed, |_| {
            assert_eq!(first.calls.get(), 1);
            create(Rc::new(config()))
        })
        .unwrap();
    drop(owner);
    assert!(weak.upgrade().is_none());
    cache.remove_terminal(&weak).unwrap();
    assert_eq!(second.calls.get(), 1);
}

#[test]
fn failed_factory_and_disposal_preserve_source_partial_state() {
    let cache = AtlasCache::<(), Atlas>::new();
    let owner = Rc::new(());
    assert!(matches!(
        cache.acquire(&owner, config(), |_| Err("create failure")),
        Err(CacheError::Create("create failure"))
    ));
    let first = cache.acquire(&owner, config(), create).unwrap();
    first.fail.set(true);
    let mut changed = config();
    changed.options.font_size = 16.0;
    assert!(matches!(
        cache.acquire(&owner, changed.clone(), |_| panic!(
            "failed disposal must stop creation"
        )),
        Err(CacheError::Dispose("dispose failure"))
    ));
    assert_eq!(first.calls.get(), 1);
    let unchanged = cache
        .acquire(&owner, config(), |_| {
            panic!("failed release retained original owner")
        })
        .unwrap();
    assert!(Rc::ptr_eq(&first, &unchanged));
    let replacement = cache.acquire(&owner, changed, create).unwrap();
    assert_eq!(first.calls.get(), 2);
    assert!(!Rc::ptr_eq(&first, &replacement));
}

#[test]
fn invalid_config_comparison_does_not_release_existing_owner() {
    let cache = AtlasCache::<(), Atlas>::new();
    let owner = Rc::new(());
    let first = cache.acquire(&owner, config(), create).unwrap();
    let mut short = config();
    short.colors.ansi.pop();
    assert!(matches!(
        cache.acquire(&owner, short, |_| panic!(
            "comparison must fail before creation"
        )),
        Err(CacheError::Config(_))
    ));
    assert_eq!(first.calls.get(), 0);
    let retained = cache.acquire(&owner, config(), create).unwrap();
    assert!(Rc::ptr_eq(&first, &retained));
}

#[test]
fn reentrant_disposal_uses_the_captured_live_splice_index_without_borrow_panics() {
    let cache = Rc::new(AtlasCache::<(), Atlas>::new());
    let a = Rc::new(());
    let b = Rc::new(());
    let first = cache.acquire(&a, config(), create).unwrap();
    let mut different = config();
    different.options.font_size = 18.0;
    let other = cache.acquire(&b, different.clone(), create).unwrap();
    let weak_cache = Rc::downgrade(&cache);
    let weak_a = Rc::downgrade(&a);
    let entered = Cell::new(false);
    *first.on_dispose.borrow_mut() = Some(Rc::new(move || {
        if !entered.replace(true) {
            weak_cache
                .upgrade()
                .unwrap()
                .remove_terminal(&weak_a)
                .unwrap();
        }
    }));
    cache.remove_terminal(&Rc::downgrade(&a)).unwrap();
    assert_eq!(first.calls.get(), 2);
    // Source outer splice removes the entry now at index0 after the inner
    // release already removed A. It does not invent a disposal event for B.
    assert_eq!(other.calls.get(), 0);
    let new_b = cache.acquire(&b, different, create).unwrap();
    assert!(!Rc::ptr_eq(&other, &new_b));
}
