// Copyright (c) 2019, 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Includes every UnicodeService.test.ts case.

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use xterm_core::event::EmitError;
use xterm_core::lifecycle::Disposable;
use xterm_core::unicode::{
    UnicodeCharProperties as Properties, UnicodeCharWidth as Width, UnicodeVersionProvider,
};
use xterm_core::unicode_service::{
    UnicodeApi, UnicodeService, UnicodeServiceError as ServiceError,
};
use xterm_core::unicode_v6::UnicodeV6;

struct ConstantProvider {
    version: String,
    width: Width,
}

impl UnicodeVersionProvider for ConstantProvider {
    fn version(&self) -> &str {
        &self.version
    }
    fn wcwidth(&self, _: u32) -> Width {
        self.width
    }
    fn char_properties(&self, code: u32, _: Properties) -> Properties {
        Properties::new(0, u32::from(self.wcwidth(code).columns()), false)
    }
}

fn provider(version: &str, width: Width) -> Rc<dyn UnicodeVersionProvider> {
    Rc::new(ConstantProvider {
        version: version.to_owned(),
        width,
    })
}

fn v6_service() -> UnicodeService {
    let service = UnicodeService::new();
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    service
}

#[test]
fn default_to_v6() {
    let service = v6_service();
    assert_eq!(service.active_version(), "6");
    assert_eq!(service.versions(), ["6"]);
    service.set_active_version("6").unwrap();
    assert_eq!(service.get_string_cell_width("hello").unwrap(), 5);
}

#[test]
fn activate_should_throw_for_unknown_version() {
    let service = v6_service();
    let error = service.set_active_version("55").unwrap_err();
    assert_eq!(error.to_string(), "unknown Unicode version \"55\"");
}

#[test]
fn should_notify_about_version_change() {
    let service = v6_service();
    let notes = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&notes);
    let _subscription = service
        .on_change()
        .subscribe(move |version| output.borrow_mut().push(version.clone()));
    service.register(provider("123", Width::Wide)).unwrap();
    service.set_active_version("123").unwrap();
    assert_eq!(*notes.borrow(), ["123"]);
}

#[test]
fn correctly_changes_provider_implementation() {
    let service = v6_service();
    assert_eq!(service.get_string_cell_width("hello").unwrap(), 5);
    service.register(provider("123", Width::Wide)).unwrap();
    service.set_active_version("123").unwrap();
    assert_eq!(service.get_string_cell_width("hello").unwrap(), 2 * 5);
}

#[test]
fn wcwidth_v6_emoji_test() {
    assert_eq!(
        v6_service()
            .get_string_cell_width("🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣")
            .unwrap(),
        10
    );
}

#[test]
fn empty_service() {
    let service = UnicodeService::new();
    assert!(service.versions().is_empty());
    assert_eq!(service.active_version(), "");
    assert_eq!(service.get_string_cell_width("").unwrap(), 0);
    assert_eq!(service.get_string_cell_width_utf16(&[]).unwrap(), 0);
    assert_eq!(service.wcwidth(65), Err(ServiceError::NoActiveProvider));
    assert_eq!(
        service.char_properties(65, Properties::default()),
        Err(ServiceError::NoActiveProvider)
    );
    assert_eq!(
        service.get_string_cell_width("a"),
        Err(ServiceError::NoActiveProvider)
    );
    assert!(ServiceError::NoActiveProvider.source().is_none());
}

#[test]
fn unknown_version_is_transactional() {
    let service = v6_service();
    let notifications = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&notifications);
    let _handle = service
        .on_change()
        .subscribe(move |value| output.borrow_mut().push(value.clone()));
    assert_eq!(
        service.set_active_version("unknown"),
        Err(ServiceError::UnknownVersion("unknown".into()))
    );
    assert_eq!(service.active_version(), "6");
    assert_eq!(service.get_string_cell_width("hello").unwrap(), 5);
    assert!(notifications.borrow().is_empty());
}

#[test]
fn registry_replacement_retains_active_snapshot() {
    let service = UnicodeService::new();
    service.register(provider("first", Width::Narrow)).unwrap();
    service.register(provider("first", Width::Wide)).unwrap();
    assert_eq!(service.versions(), ["first"]);
    assert_eq!(service.wcwidth(65).unwrap(), Width::Narrow);
    service.set_active_version("first").unwrap();
    assert_eq!(service.wcwidth(65).unwrap(), Width::Wide);
}

#[test]
fn version_keys_match_object_keys() {
    let service = UnicodeService::new();
    for name in [
        "6",
        "11",
        "2",
        "01",
        "-0",
        "4294967294",
        "4294967295",
        "",
        "__proto__",
        "constructor",
        "0",
    ] {
        service.register(provider(name, Width::Narrow)).unwrap();
    }
    assert_eq!(
        service.versions(),
        [
            "0",
            "2",
            "6",
            "11",
            "4294967294",
            "01",
            "-0",
            "4294967295",
            "",
            "__proto__",
            "constructor"
        ]
    );
    assert_eq!(service.active_version(), "6");
    service.set_active_version("__proto__").unwrap();
    assert_eq!(service.wcwidth(0).unwrap(), Width::Narrow);
    for name in ["+1", " 1", "1 ", "1.0", "1e0", "00"] {
        service.register(provider(name, Width::Zero)).unwrap();
    }
    assert_eq!(
        &service.versions()[11..],
        ["+1", " 1", "1 ", "1.0", "1e0", "00"]
    );
}

#[test]
fn empty_name_registration_follows_source() {
    let service = UnicodeService::new();
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&events);
    let _handle = service
        .on_change()
        .subscribe(move |value| output.borrow_mut().push(value.clone()));
    service.register(provider("", Width::Zero)).unwrap();
    assert_eq!(service.active_version(), "");
    assert_eq!(service.wcwidth(65).unwrap(), Width::Zero);
    service.register(provider("next", Width::Wide)).unwrap();
    assert_eq!(service.active_version(), "next");
    assert_eq!(*events.borrow(), ["", "next"]);
}

#[test]
fn selection_commits_before_reentrant_events() {
    let service = Rc::new(v6_service());
    service.register(provider("other", Width::Wide)).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&service);
    let output = Rc::clone(&events);
    let _subscription = service.on_change().subscribe(move |version| {
        let service = weak.upgrade().unwrap();
        assert_eq!(service.active_version(), *version);
        output.borrow_mut().push(version.clone());
        if version == "other" {
            service.set_active_version("6").unwrap();
        }
    });
    service.set_active_version("other").unwrap();
    assert_eq!(*events.borrow(), ["other", "6"]);
    assert_eq!(service.active_version(), "6");
    service.set_active_version("6").unwrap();
    assert_eq!(*events.borrow(), ["other", "6", "6"]);
}

#[test]
fn event_failure_keeps_committed_state() {
    let service = UnicodeService::new();
    let handle = service
        .on_change()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    assert_eq!(
        service.register(provider("broken", Width::Wide)),
        Err(ServiceError::Event(EmitError::ListenerArrayCleared))
    );
    assert_eq!(service.active_version(), "broken");
    assert_eq!(service.versions(), ["broken"]);
    assert_eq!(service.wcwidth(65).unwrap(), Width::Wide);
    let error = service.set_active_version("broken").unwrap_err();
    assert!(error.source().is_some());
    handle.dispose();
    service.set_active_version("broken").unwrap();
}

#[test]
fn event_callback_payload_is_retained() {
    let service = v6_service();
    let failure = EmitError::callback(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "unicode host callback",
    ));
    let expected = failure.clone();
    let _listener = service
        .on_change()
        .try_subscribe(move |_| Err(failure.clone()));
    let error = service.set_active_version("6").unwrap_err();
    let ServiceError::Event(actual) = error else {
        panic!("lost event error")
    };
    assert_eq!(actual, expected);
    let payload = actual.callback_ref::<std::io::Error>().unwrap();
    assert_eq!(payload.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(payload.to_string(), "unicode host callback");
    assert_eq!(service.active_version(), "6");
}

#[test]
fn dispose_only_clears_notifications() {
    let service = v6_service();
    let count = Rc::new(RefCell::new(0));
    let output = Rc::clone(&count);
    let handle = service
        .on_change()
        .subscribe(move |_| *output.borrow_mut() += 1);
    service.set_active_version("6").unwrap();
    handle.dispose();
    service.set_active_version("6").unwrap();
    assert_eq!(*count.borrow(), 1);
    service.dispose();
    service.dispose();
    let output = Rc::clone(&count);
    let _after_disposal = service
        .on_change()
        .subscribe(move |_| *output.borrow_mut() += 1);
    service.register(provider("after", Width::Wide)).unwrap();
    service.set_active_version("after").unwrap();
    assert_eq!(service.wcwidth(65).unwrap(), Width::Wide);
    assert_eq!(service.versions(), ["6", "after"]);
    assert_eq!(*count.borrow(), 1);
}

#[test]
fn unicode_api_views_share_service() {
    let service = v6_service();
    let first = UnicodeApi::new(&service);
    let second = UnicodeApi::new(&service);
    assert_eq!(first.versions(), ["6"]);
    first.register(provider("custom", Width::Wide)).unwrap();
    second.set_active_version("custom").unwrap();
    assert_eq!(first.active_version(), "custom");
    assert_eq!(service.wcwidth(65).unwrap(), Width::Wide);
    assert_eq!(
        second.set_active_version("bad"),
        Err(ServiceError::UnknownVersion("bad".into()))
    );
}
