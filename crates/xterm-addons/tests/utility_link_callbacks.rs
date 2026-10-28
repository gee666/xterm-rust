// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Callback timing uses a real Rust buffer. URL validation has separate oracles.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_addons::{link_snapshot::LinkSnapshot, web_link_provider::*, web_links::*};
use xterm_core::{
    lifecycle::{to_disposable, Disposable, DisposableHandle},
    terminal::CoreTerminal,
};
type Calls = Rc<RefCell<Vec<&'static str>>>;
struct Matcher {
    enabled: Rc<Cell<bool>>,
    calls: Calls,
}
impl LinkMatcher for Matcher {
    fn matches(&self, _: &[u16], _: &RegexSpec) -> Result<Vec<LinkMatch>, LinkError> {
        self.calls.borrow_mut().push("match");
        Ok(if self.enabled.get() {
            vec![LinkMatch {
                index: 0,
                text: "http://example.com".encode_utf16().collect(),
            }]
        } else {
            Vec::new()
        })
    }
}
struct Host {
    engine: CoreTerminal,
    provider: RefCell<Option<Provider<()>>>,
    disposed: Rc<Cell<bool>>,
}
impl WebLinkTerminal<()> for Host {
    fn buffer(&self) -> Result<Box<dyn LinkBuffer>, LinkError> {
        Ok(Box::new(LinkSnapshot::from_engine(&self.engine)?))
    }
    fn register_link_provider(
        &self,
        provider: Provider<()>,
    ) -> Result<DisposableHandle, LinkError> {
        *self.provider.borrow_mut() = Some(provider);
        let disposed = self.disposed.clone();
        Ok(to_disposable(move || disposed.set(true)))
    }
    fn open_link(&self, _: &(), _: &[u16]) -> Result<(), LinkError> {
        Err(LinkError::Message("test does not open windows".into()))
    }
}
#[test]
fn link_factories_follow_matches_and_release_options_before_callbacks() {
    let mut engine = CoreTerminal::new(80, 24, 1000).unwrap();
    engine.write("http://example.com").unwrap();
    let host = Rc::new(Host {
        engine,
        provider: RefCell::default(),
        disposed: Rc::default(),
    });
    let calls = Calls::default();
    let enabled = Rc::new(Cell::new(true));
    let addon = WebLinksAddon::new(
        Rc::new(Matcher {
            enabled: enabled.clone(),
            calls: calls.clone(),
        }),
        None,
        LinkOptions::default(),
    );
    let options = Rc::downgrade(&addon.options);
    let leave_calls = calls.clone();
    addon.options.borrow_mut().leave_factory = Some(Rc::new(move || {
        leave_calls.borrow_mut().push("leave getter");
        // Source option getters can mutate the same option object reentrantly.
        options.upgrade().unwrap().borrow_mut().leave = None;
        Ok(None)
    }));
    let hover_calls = calls.clone();
    addon.options.borrow_mut().hover_factory = Some(Rc::new(move |_| {
        hover_calls.borrow_mut().push("hover factory");
        let calls = hover_calls.clone();
        Ok(Rc::new(move |_: &(), _: &[u16]| {
            calls.borrow_mut().push("hover");
            Ok(())
        }))
    }));
    addon.activate(host.clone()).unwrap();
    let provider = host.provider.borrow().clone().unwrap();
    let links = provider(1).unwrap();
    assert_eq!(*calls.borrow(), ["match", "leave getter", "hover factory"]);
    (links[0].hover)(&(), &links[0].text).unwrap();
    assert_eq!(
        *calls.borrow(),
        ["match", "leave getter", "hover factory", "hover"]
    );
    calls.borrow_mut().clear();
    enabled.set(false);
    assert!(provider(1).unwrap().is_empty());
    assert_eq!(
        *calls.borrow(),
        ["match"],
        "no link means no per-link option getter"
    );
    addon.dispose();
    assert!(host.disposed.get());
}
