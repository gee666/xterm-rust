// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use xterm_core::{
    event::EmitError,
    unicode::{UnicodeCharProperties as Props, UnicodeCharWidth as Width, UnicodeVersionProvider},
    unicode_callbacks::UnicodeProviderCallbacks,
    unicode_graphemes::UnicodeGraphemeProvider,
    unicode_service::UnicodeService,
    unicode11::UnicodeV11,
    unicode_v6::UnicodeV6,
};
#[test]
fn signed_atlas_width_matches_all_original_providers() {
    let data: Value =
        serde_json::from_str(include_str!("fixtures/webgl-unicode-number.json")).unwrap();
    assert_eq!(data["cases"].as_array().unwrap().len(), 112);
    for c in data["cases"].as_array().unwrap() {
        let provider: Rc<dyn UnicodeVersionProvider> = match c["version"].as_str().unwrap() {
            "6" => Rc::new(UnicodeV6::new()),
            "11" => Rc::new(UnicodeV11::new()),
            version => {
                let p = UnicodeGraphemeProvider::new(version == "15-graphemes");
                p.ambiguous_chars_are_wide
                    .set(c["ambiguous"].as_bool().unwrap());
                Rc::new(p)
            }
        };
        let service = UnicodeService::new();
        service.register(provider).unwrap();
        assert_eq!(
            u64::from(
                service
                    .wcwidth_i32(i32::try_from(c["code"].as_i64().unwrap()).unwrap())
                    .unwrap()
                    .columns()
            ),
            c["width"].as_u64().unwrap(),
            "{c}"
        );
    }
}
struct Reentrant {
    service: Weak<UnicodeService>,
    calls: Rc<RefCell<Vec<i32>>>,
}
impl UnicodeProviderCallbacks for Reentrant {
    fn version(&self) -> Result<String, EmitError> {
        Ok("signed".into())
    }
    fn wcwidth(&self, c: u32) -> Result<Width, EmitError> {
        self.calls.borrow_mut().push(i32::try_from(c).unwrap());
        Ok(Width::Narrow)
    }
    fn wcwidth_i32(&self, c: i32) -> Result<Width, EmitError> {
        self.calls.borrow_mut().push(c);
        self.service
            .upgrade()
            .unwrap()
            .set_active_version("6")
            .map_err(EmitError::callback)?;
        Ok(Width::Wide)
    }
    fn char_properties(&self, _: u32, _: Props) -> Result<Props, EmitError> {
        Ok(Props::new(0, 1, false))
    }
}
#[test]
fn signed_host_callback_reenters_the_actual_registry() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    service
        .register_callbacks(Rc::new(Reentrant {
            service: Rc::downgrade(&service),
            calls: calls.clone(),
        }))
        .unwrap();
    service.set_active_version("signed").unwrap();
    assert_eq!(service.wcwidth(65).unwrap(), Width::Narrow);
    assert_eq!(service.wcwidth_i32(-2147483583).unwrap(), Width::Wide);
    assert_eq!(service.active_version(), "6");
    assert_eq!(service.wcwidth_i32(-2147483583).unwrap(), Width::Zero);
    assert_eq!(*calls.borrow(), [65, -2147483583]);
}
