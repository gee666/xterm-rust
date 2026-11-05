// Copyright (c) 2020 The xterm.js authors. MIT license.
use super::{
    controls::{self, Control},
    Cleanup, State,
};
use crate::{
    image::ImageRendererLifecycle,
    js_error::{callback_error, to_js_error},
    terminal::Terminal,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::ImageStorage;
use xterm_core::{
    lifecycle::DisposableHandle, parser::escape::FunctionIdentifier, terminal::TerminalEvent,
};
pub(super) fn token(token: DisposableHandle) -> Cleanup {
    Rc::new(move || {
        token.dispose();
        Ok(())
    })
}
impl State {
    pub(super) fn parent_registrations(
        self: &Rc<Self>,
        terminal: &Terminal,
        renderer: Rc<ImageRendererLifecycle>,
        storage: ImageStorage,
    ) -> Result<Vec<Cleanup>, JsValue> {
        // Evaluate the entire source _disposeLater argument list before appending
        // tokens. A registration failure must not invent transactional rollback.
        let mut cleanups: Vec<Cleanup> = vec![
            {
                let renderer = Rc::clone(&renderer);
                Rc::new(move || renderer.dispose())
            },
            {
                let storage = storage.clone();
                Rc::new(move || storage.dispose().map_err(|e| to_js_error(&e)))
            },
        ];
        for (prefix, final_byte, control) in [
            (Some("?"), "h", Control::Set),
            (Some("?"), "l", Control::Reset),
            (None, "c", Control::Attributes),
            (Some("?"), "S", Control::Graphics),
        ] {
            let weak = Rc::downgrade(self);
            let registration = terminal
                .shared
                .native
                .with_parser(|parser| {
                    parser.register_csi_handler(
                        FunctionIdentifier {
                            prefix,
                            intermediates: None,
                            final_byte,
                        },
                        move |params| {
                            controls::result(
                                weak.upgrade()
                                    .map_or(Ok(false), |state| state.control(control, params)),
                            )
                        },
                    )
                })
                .map_err(|e| to_js_error(&e))?
                .map_err(|e| to_js_error(&e))?;
            cleanups.push(token(registration));
        }
        let weak = Rc::downgrade(self);
        cleanups.push(token(
            terminal
                .shared
                .browser_events
                .render
                .event()
                .try_subscribe(move |range| {
                    if let Some(storage) = weak.upgrade().and_then(|s| s.storage()) {
                        storage
                            .render(range.start as usize, range.end as usize)
                            .map_err(|e| callback_error(to_js_error(&e)))?;
                    }
                    Ok(())
                }),
        ));
        let weak = Rc::downgrade(self);
        cleanups.push(token(
            terminal
                .shared
                .native
                .with_parser(|parser| {
                    parser.register_csi_handler(
                        FunctionIdentifier {
                            prefix: None,
                            intermediates: Some("!"),
                            final_byte: "p",
                        },
                        move |_| controls::result(weak.upgrade().map_or(Ok(false), |s| s.reset())),
                    )
                })
                .map_err(|e| to_js_error(&e))?
                .map_err(|e| to_js_error(&e))?,
        ));
        let weak = Rc::downgrade(self);
        cleanups.push(token(
            terminal
                .shared
                .native
                .with_parser(|parser| {
                    parser.register_esc_handler(FunctionIdentifier::new("c"), move || {
                        controls::result(weak.upgrade().map_or(Ok(false), |s| s.reset()))
                    })
                })
                .map_err(|e| to_js_error(&e))?
                .map_err(|e| to_js_error(&e))?,
        ));
        let weak = Rc::downgrade(self);
        cleanups.push(token(terminal.shared.native.on_event().try_subscribe(
            move |event| {
                if matches!(event, TerminalEvent::RequestReset) {
                    if let Some(state) = weak.upgrade() {
                        state.reset().map_err(callback_error)?;
                    }
                }
                Ok(())
            },
        )));
        let weak = Rc::downgrade(self);
        cleanups.push(token(
            terminal
                .shared
                .native
                .buffer()
                .map_err(|e| to_js_error(&e))?
                .on_buffer_change()
                .try_subscribe(move |_| {
                    if let Some(storage) = weak.upgrade().and_then(|s| s.storage()) {
                        storage
                            .wipe_alternate()
                            .map_err(|e| callback_error(to_js_error(&e)))?;
                    }
                    Ok(())
                }),
        ));
        let weak = Rc::downgrade(self);
        cleanups.push(token(terminal.shared.native.on_resize().try_subscribe(
            move |metrics| {
                if let Some(storage) = weak.upgrade().and_then(|s| s.storage()) {
                    storage
                        .viewport_resize(metrics.cols, metrics.rows)
                        .map_err(|e| callback_error(to_js_error(&e)))?;
                }
                Ok(())
            },
        )));
        Ok(cleanups)
    }
}
