// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Rust web-links addon to browser linkifier bridge.
use crate::links::{
    browser_linkifier::{BrowserLink, BrowserLinkProvider, BrowserProviders, LinkReply},
    Link, Position, Range,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::MouseEvent;
use xterm_addons::{
    web_link_provider::{Provider, WebLinkTerminal},
    web_links::{LinkBuffer, LinkError},
};
use xterm_core::lifecycle::DisposableHandle;

pub type Snapshot = Rc<dyn Fn() -> Result<Box<dyn LinkBuffer>, LinkError>>;
pub struct BrowserWebLinkTerminal {
    pub providers: BrowserProviders,
    pub snapshot: Snapshot,
}
struct ProviderBridge(Provider<MouseEvent>);
fn error(error: LinkError) -> JsValue {
    crate::js_error::to_js_error(&error)
}
impl BrowserLinkProvider for ProviderBridge {
    fn provide_links(&self, y: usize, callback: LinkReply) -> Result<(), JsValue> {
        let links = (self.0)(y).map_err(error)?;
        let mut result = Vec::with_capacity(links.len());
        for link in links {
            let activate = link.activate;
            let hover = link.hover;
            result.push(BrowserLink {
                data: Rc::new(Link {
                    text: link.text,
                    range: Range {
                        start: Position {
                            x: link.range.start.x,
                            y: link.range.start.y,
                        },
                        end: Position {
                            x: link.range.end.x,
                            y: link.range.end.y,
                        },
                    },
                }),
                decorations: None,
                dispose: None,
                activate: Rc::new(move |event, link| activate(event, &link.text).map_err(error)),
                hover: Some(Rc::new(move |event, link| {
                    hover(event, &link.text).map_err(error)
                })),
                leave: link.leave.map(|leave| {
                    Rc::new(move |event: &MouseEvent, link: &Link| {
                        leave(event, &link.text).map_err(error)
                    }) as crate::links::browser_linkifier::LinkCallback
                }),
            });
        }
        callback(Some(result))
    }
}
impl WebLinkTerminal<MouseEvent> for BrowserWebLinkTerminal {
    fn buffer(&self) -> Result<Box<dyn LinkBuffer>, LinkError> {
        (self.snapshot)()
    }
    fn register_link_provider(
        &self,
        provider: Provider<MouseEvent>,
    ) -> Result<DisposableHandle, LinkError> {
        Ok(Rc::new(
            self.providers.register(Rc::new(ProviderBridge(provider))),
        ))
    }
    fn open_link(&self, _event: &MouseEvent, uri: &[u16]) -> Result<(), LinkError> {
        super::open_link::open_link(uri)
    }
}
