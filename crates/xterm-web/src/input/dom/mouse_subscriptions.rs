// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;

fn failure(state: &State, error: JsValue) -> xterm_core::event::EmitError {
    let callback_error = crate::js_error::callback_error(error.clone());
    *state.protocol_error.borrow_mut() = Some(error);
    callback_error
}

pub(super) fn bind(state: &Rc<State>) -> (DisposableHandle, DisposableHandle) {
    let weak = Rc::downgrade(state);
    let protocol = state
        .service
        .on_protocol_change()
        .try_subscribe(move |events| {
            if let Some(state) = weak.upgrade() {
                if !state.disposed.get() {
                    sync_protocol(&state, *events).map_err(|error| failure(&state, error))?;
                }
            }
            Ok(())
        });
    let weak = Rc::downgrade(state);
    let options = state.host.mouse_option_changes().try_subscribe(move |key| {
        if *key == "mouseEventsRequireAlt" {
            if let Some(state) = weak.upgrade() {
                if !state.disposed.get() {
                    sync_mode(&state)
                        .and_then(|()| alt::sync(&state))
                        .map_err(|error| failure(&state, error))?;
                }
            }
        }
        Ok(())
    });
    (protocol, options)
}
