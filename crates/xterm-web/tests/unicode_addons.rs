#![cfg(target_arch = "wasm32")]
// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved. MIT license.

use std::rc::Rc;
use wasm_bindgen_test::*;
use xterm_core::public_api::AddonManager;
use xterm_core::unicode_addons::{Unicode11Addon, UnicodeGraphemesAddon};
use xterm_core::unicode_service::UnicodeService;
use xterm_core::unicode_v6::UnicodeV6;

wasm_bindgen_test_configure!(run_in_browser);

// These are Rust/WASM component assertions in real engines, not a substitute
// for original Playwright suites against the complete Rust terminal binding.
#[wasm_bindgen_test]
fn original_unicode_addon_versions_and_widths_in_wasm() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let addon = Unicode11Addon::new();
    addon.activate(&service).unwrap();
    assert!(service.versions().contains(&"11".to_owned()));
    service.set_active_version("11").unwrap();
    assert_eq!(service.active_version(), "11");
    assert_eq!(
        service
            .get_string_cell_width("🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣")
            .unwrap(),
        20
    );
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let addon = UnicodeGraphemesAddon::new();
    addon.activate(Rc::clone(&service)).unwrap();
    assert_eq!(service.versions(), ["6", "15", "15-graphemes"]);
    assert_eq!(service.active_version(), "15-graphemes");
    for line in include_str!("../../xterm-core/tests/fixtures/unicode-grapheme-strings.tsv").lines()
    {
        let (units, expected) = line.split_once('\t').unwrap();
        let units: Vec<u16> = units
            .split(',')
            .map(|v| u16::from_str_radix(v, 16).unwrap())
            .collect();
        assert_eq!(
            service.get_string_cell_width_utf16(&units).unwrap(),
            expected.parse::<i64>().unwrap()
        );
    }
    addon.dispose().unwrap();
    assert_eq!(service.active_version(), "6");
}

#[wasm_bindgen_test]
fn native_addon_manager_lifecycle_in_wasm() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let manager = AddonManager::new();
    let addon = manager
        .load_addon(&service, Rc::new(UnicodeGraphemesAddon::new()))
        .unwrap();
    assert_eq!(service.active_version(), "15-graphemes");
    addon.dispose().unwrap();
    assert_eq!(service.active_version(), "6");
    service.set_active_version("15").unwrap();
    addon.dispose().unwrap();
    assert_eq!(service.active_version(), "15");
    assert!(manager.is_empty());
}
