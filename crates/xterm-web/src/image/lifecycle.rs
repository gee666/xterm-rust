// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{terminal_host::TerminalCanvasHost, BrowserImageRenderer};
use crate::{
    js_error::{callback_error, to_js_error},
    renderer::{service::RenderService, service_api::RendererChangeAction},
    terminal::{Shared, Terminal},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::ImageLayer;
use xterm_core::lifecycle::{Disposable, DisposableHandle, DisposableStore};

struct State {
    terminal: Weak<Shared>,
    renderer: Rc<BrowserImageRenderer>,
    service: RefCell<Option<Rc<RenderService>>>,
    renderer_hook: RefCell<Option<DisposableHandle>>,
    listeners: DisposableStore,
    disposed: Cell<bool>,
    delegates: Cell<bool>,
}
impl State {
    fn open(self: &Rc<Self>) {
        let service = self
            .terminal
            .upgrade()
            .and_then(|shared| shared.services.borrow().clone())
            .map(|services| Rc::clone(&services.render_service));
        let Some(service) = service else { return };
        let previous = self.renderer_hook.borrow_mut().take();
        if let Some(previous) = previous {
            previous.dispose();
        }
        *self.service.borrow_mut() = Some(Rc::clone(&service));
        self.delegates.set(true);
        let weak = Rc::downgrade(self);
        let hook = service.intercept_renderer_change(move || {
            if let Some(state) = weak.upgrade() {
                // Snapshot keys before removal, as the source spread of Map.keys.
                let layers: Vec<_> = state
                    .renderer
                    .layers
                    .keys()
                    .into_iter()
                    .collect::<Result<_, _>>()?;
                for key in layers {
                    let layer = if key == JsValue::from_str("top") {
                        ImageLayer::Top
                    } else {
                        ImageLayer::Bottom
                    };
                    state.renderer.remove_layer(layer)?;
                }
            }
            Ok(
                if weak.upgrade().is_some_and(|state| state.delegates.get()) {
                    RendererChangeAction::Continue
                } else {
                    RendererChangeAction::Stop
                },
            )
        });
        *self.renderer_hook.borrow_mut() = Some(hook);
    }
    fn refresh(&self) -> Result<(), JsValue> {
        let service = self.service.borrow().clone();
        if let Some(service) = service {
            if let Some(shared) = self.terminal.upgrade() {
                let rows = shared.native.rows().map_err(|error| to_js_error(&error))?;
                // Source deliberately passes rows, not rows - 1.
                service.refresh_rows(0.0, rows as f64, false, false)?;
            }
        }
        Ok(())
    }
    fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        // Source registers optionsRefresh before its canvas cleanup callback.
        self.listeners.dispose();
        self.renderer.remove_layer(ImageLayer::Top)?;
        self.renderer.remove_layer(ImageLayer::Bottom)?;
        self.delegates.set(false);
        let hook = self.renderer_hook.borrow_mut().take();
        if let Some(hook) = hook {
            hook.dispose();
        }
        self.service.borrow_mut().take();
        self.renderer.clear_placeholder()
    }
}

/// Live terminal owner for the image canvas component. The complete ImageAddon
/// owns this alongside ImageStorage and the protocol handlers. Rust hooks replace
/// upstream's private method monkeypatches; no original JS engine executes here.
/// Dropping this owner unregisters listeners and releases canvas/bitmap resources.
pub struct ImageRendererLifecycle {
    state: Rc<State>,
}
impl ImageRendererLifecycle {
    pub fn new(terminal: &Terminal) -> Result<Self, JsValue> {
        let renderer = Rc::new(BrowserImageRenderer::new(Rc::new(TerminalCanvasHost(
            Rc::downgrade(&terminal.shared),
        )))?);
        let state = Rc::new(State {
            terminal: Rc::downgrade(&terminal.shared),
            renderer,
            service: RefCell::new(None),
            renderer_hook: RefCell::new(None),
            listeners: DisposableStore::new(),
            disposed: Cell::new(false),
            delegates: Cell::new(false),
        });
        let weak = Rc::downgrade(&state);
        state
            .listeners
            .add(terminal.shared.did_open.event().subscribe(move |_| {
                if let Some(state) = weak.upgrade() {
                    state.open();
                }
            }));
        if terminal.shared.dom.borrow().is_some() {
            state.open();
        }
        let weak = Rc::downgrade(&state);
        state.listeners.add(
            terminal
                .shared
                .options
                .service()
                .on_option_change()
                .try_subscribe(move |key| {
                    if *key == "fontSize" {
                        if let Some(state) = weak.upgrade() {
                            state.renderer.rescale_canvas().map_err(callback_error)?;
                            state.refresh().map_err(callback_error)?;
                        }
                    }
                    Ok(())
                }),
        );
        Ok(Self { state })
    }
    /// Share the exact renderer instance with ImageStorage.
    pub fn renderer(&self) -> Rc<BrowserImageRenderer> {
        Rc::clone(&self.state.renderer)
    }
    pub fn show_placeholder(&self, value: bool) -> Result<(), JsValue> {
        self.state.renderer.show_placeholder(value)?;
        self.state.refresh()
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.dispose()
    }
}
impl Drop for ImageRendererLifecycle {
    fn drop(&mut self) {
        if let Err(error) = self.state.dispose() {
            // A destructor cannot return a JS exception. Explicit dispose does.
            web_sys::console::error_1(&error);
        }
    }
}
