// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved. MIT license.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::public_api::AddonManager;
use xterm_core::unicode_addons::{Unicode11Addon, UnicodeGraphemesAddon};
use xterm_core::unicode_service::UnicodeService;
use xterm_core::unicode_v6::UnicodeV6;

#[test]
fn shared_manager_owns_registration_and_idempotent_disposal() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let manager = AddonManager::new();
    let eleven = manager
        .load_addon(&service, Rc::new(Unicode11Addon::new()))
        .unwrap();
    let graphemes = manager
        .load_addon(&service, Rc::new(UnicodeGraphemesAddon::new()))
        .unwrap();
    assert_eq!(manager.len(), 2);
    assert_eq!(service.active_version(), "15-graphemes");
    graphemes.dispose().unwrap();
    assert_eq!(service.active_version(), "6");
    assert_eq!(manager.len(), 1);
    // Managed disposal differs from repeated direct addon.dispose, just as
    // upstream's AddonManager replaces dispose with an idempotent wrapper.
    service.set_active_version("11").unwrap();
    graphemes.dispose().unwrap();
    assert_eq!(service.active_version(), "11");
    manager.dispose().unwrap();
    assert!(eleven.is_disposed());
    assert!(manager.is_empty());
    assert_eq!(service.versions(), ["6", "11", "15", "15-graphemes"]);
}

#[test]
fn unchanged_original_addon_lifecycle_traces() {
    let mut count = 0;
    for (case, line) in include_str!("fixtures/unicode-addon-lifecycle.tsv")
        .lines()
        .enumerate()
    {
        let services = [
            Rc::new(UnicodeService::new()),
            Rc::new(UnicodeService::new()),
        ];
        let changes = Rc::new(RefCell::new(Vec::new()));
        for (index, service) in services.iter().enumerate() {
            service.register(Rc::new(UnicodeV6::new())).unwrap();
            let changes = Rc::clone(&changes);
            service
                .on_change()
                .subscribe(move |version| changes.borrow_mut().push(format!("{index}:{version}")));
        }
        let addon = UnicodeGraphemesAddon::new();
        let (actions, expected) = line.split_once('\t').unwrap();
        let actions: Vec<_> = actions.split(';').collect();
        let expected: Vec<_> = expected.split(';').collect();
        assert_eq!(actions.len(), expected.len());
        for (step, (action, expected)) in actions.into_iter().zip(expected).enumerate() {
            let result = match action.as_bytes()[0] {
                b'd' => addon.dispose(),
                b'a' => addon.activate(Rc::clone(&services[action[1..].parse::<usize>().unwrap()])),
                b'e' => {
                    Unicode11Addon::new().activate(&services[action[1..].parse::<usize>().unwrap()])
                }
                b'v' => {
                    let (index, version) = action[1..].split_once(':').unwrap();
                    services[index.parse::<usize>().unwrap()].set_active_version(version)
                }
                _ => panic!("invalid persisted action"),
            };
            let error = result.err().map_or_else(String::new, |e| format!("!{e}:"));
            let actual = format!(
                "{error}{},{}|{}>{}|{}",
                services[0].active_version(),
                services[1].active_version(),
                services[0].versions().join(","),
                services[1].versions().join(","),
                changes.borrow().join(",")
            );
            assert_eq!(
                actual, expected,
                "lifecycle case {case} step {step} action {action}"
            );
            count += 1;
        }
    }
    assert_eq!(count, 4096);
}

#[test]
fn addon_reentry_releases_borrows_and_owns_no_callback_cycles() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let addon = Rc::new(UnicodeGraphemesAddon::new());
    let weak = Rc::downgrade(&addon);
    let once = Cell::new(false);
    service.on_change().subscribe(move |version| {
        if version == "15-graphemes" && !once.replace(true) {
            weak.upgrade().unwrap().dispose().unwrap();
        }
    });
    addon.activate(Rc::clone(&service)).unwrap();
    assert_eq!(service.active_version(), "6");
    assert_eq!(Rc::strong_count(&service), 2);
    let weak = Rc::downgrade(&addon);
    drop(addon);
    assert!(weak.upgrade().is_none());
    assert_eq!(Rc::strong_count(&service), 1);
}
