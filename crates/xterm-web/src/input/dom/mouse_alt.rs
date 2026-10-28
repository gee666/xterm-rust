// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use web_sys::Event;

fn active(state: &State) -> Result<bool, JsValue> {
    Ok(state.service.are_mouse_events_active() && state.host.mouse_require_alt()?)
}

fn modifier(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    if !active(state)? {
        return Ok(());
    }
    let held = values::alt_modifier(&event)?;
    state
        .element
        .class_list()
        .toggle_with_force(ENABLE_MOUSE_EVENTS, held)?;
    Ok(())
}

pub(super) fn sync(state: &Rc<State>) -> Result<(), JsValue> {
    let mut old = std::mem::take(&mut state.dynamic.borrow_mut().alt_listeners);
    for listener in &mut old {
        listener.dispose()?;
    }
    if !active(state)? {
        return Ok(());
    }
    let mut listeners = vec![
        listen(state, state.document.as_ref(), "keydown", false, modifier)?,
        listen(state, state.document.as_ref(), "keyup", false, modifier)?,
        listen(state, state.element.as_ref(), "mousemove", false, modifier)?,
    ];
    if let Some(window) = state
        .element
        .owner_document()
        .and_then(|document| document.default_view())
    {
        listeners.push(listen(
            state,
            window.as_ref(),
            "blur",
            false,
            |state, _| {
                if active(state)? {
                    state.element.class_list().remove_1(ENABLE_MOUSE_EVENTS)?;
                }
                Ok(())
            },
        )?);
    }
    state.dynamic.borrow_mut().alt_listeners = listeners;
    Ok(())
}
