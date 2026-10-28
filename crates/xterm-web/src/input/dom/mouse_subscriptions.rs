// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;

pub(super) fn bind(state: &Rc<State>) -> (DisposableHandle, DisposableHandle) {
    let weak = Rc::downgrade(state);
    let protocol = state
        .service
        .on_protocol_change()
        .try_subscribe(move |events| {
            if let Some(state) = weak.upgrade() {
                if !state.disposed.get() {
                    sync_protocol(&state, *events).map_err(crate::js_error::callback_error)?;
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
                        .map_err(crate::js_error::callback_error)?;
                }
            }
        }
        Ok(())
    });
    (protocol, options)
}
