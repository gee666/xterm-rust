// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of WebLinksAddon registration/callback lifecycle.
use crate::web_links::{compute_links, LinkBuffer, LinkError, LinkMatcher, LinkRange, RegexSpec};
use std::{cell::RefCell, rc::Rc};
use xterm_core::lifecycle::{Disposable, DisposableHandle};

pub type Activate<E> = Rc<dyn Fn(&E, &[u16]) -> Result<(), LinkError>>;
pub type Hover<E> = Rc<dyn Fn(&E, &[u16], LinkRange) -> Result<(), LinkError>>;
pub struct LinkOptions<E> {
    pub regex: Option<RegexSpec>,
    pub hover: Option<Hover<E>>,
    pub leave: Option<Activate<E>>,
}
impl<E> Default for LinkOptions<E> {
    fn default() -> Self {
        Self {
            regex: None,
            hover: None,
            leave: None,
        }
    }
}
pub struct ProvidedLink<E> {
    pub text: Vec<u16>,
    pub range: LinkRange,
    pub activate: Activate<E>,
    pub hover: Activate<E>,
    pub leave: Option<Activate<E>>,
}
pub type Provider<E> = Rc<dyn Fn(usize) -> Result<Vec<ProvidedLink<E>>, LinkError>>;
pub trait WebLinkTerminal<E> {
    /// Snapshot must describe the live active buffer at request time.
    fn buffer(&self) -> Result<Box<dyn LinkBuffer>, LinkError>;
    fn register_link_provider(&self, provider: Provider<E>) -> Result<DisposableHandle, LinkError>;
    fn open_link(&self, event: &E, uri: &[u16]) -> Result<(), LinkError>;
}
pub struct WebLinksAddon<E> {
    pub options: Rc<RefCell<LinkOptions<E>>>,
    handler: Option<Activate<E>>,
    matcher: Rc<dyn LinkMatcher>,
    registration: RefCell<Option<DisposableHandle>>,
}
impl<E: 'static> WebLinksAddon<E> {
    pub fn new(
        matcher: Rc<dyn LinkMatcher>,
        handler: Option<Activate<E>>,
        options: LinkOptions<E>,
    ) -> Self {
        Self {
            matcher,
            handler,
            options: Rc::new(RefCell::new(options)),
            registration: RefCell::default(),
        }
    }
    pub fn activate(&self, terminal: Rc<dyn WebLinkTerminal<E>>) -> Result<(), LinkError> {
        let target = Rc::downgrade(&terminal);
        let open_target = target.clone();
        let handler = self.handler.clone().unwrap_or_else(|| {
            Rc::new(move |event, uri| {
                open_target
                    .upgrade()
                    .ok_or_else(|| LinkError("link terminal was dropped".into()))?
                    .open_link(event, uri)
            })
        });
        let matcher = Rc::clone(&self.matcher);
        let options = Rc::clone(&self.options);
        let regex = options.borrow().regex.clone().unwrap_or_default();
        let provider = Rc::new(move |y| {
            let buffer = target
                .upgrade()
                .ok_or_else(|| LinkError("link terminal was dropped".into()))?
                .buffer()?;
            let links = compute_links(y, &regex, &*buffer, &*matcher)?;
            Ok(links
                .into_iter()
                .map(|link| {
                    let range = link.range;
                    let options = Rc::clone(&options);
                    let leave = options.borrow().leave.clone();
                    ProvidedLink {
                        text: link.text,
                        range,
                        activate: handler.clone(),
                        leave,
                        hover: Rc::new(move |event, uri| {
                            // Clone before callback so changing options reentrantly is safe.
                            let callback = options.borrow().hover.clone();
                            if let Some(callback) = callback {
                                callback(event, uri, range)?;
                            }
                            Ok(())
                        }),
                    }
                })
                .collect())
        });
        *self.registration.borrow_mut() = Some(terminal.register_link_provider(provider)?);
        Ok(())
    }
}
impl<E> Disposable for WebLinksAddon<E> {
    fn dispose(&self) {
        let registration = self.registration.borrow().clone();
        if let Some(registration) = registration {
            registration.dispose();
        }
    }
}
impl<E> Drop for WebLinksAddon<E> {
    fn drop(&mut self) {
        self.dispose();
    }
}
