// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::super::service::SelectionMode;
use super::*;

pub(super) fn bind(state: &Rc<State>) -> Result<(), JsValue> {
    let weak = Rc::downgrade(state);
    let listener = DomListener::new(state.element.as_ref(), "mousedown", false, move |event| {
        if let Some(state) = weak.upgrade() {
            if state.disposed.get() {
                return;
            }
            match event.dyn_into::<MouseEvent>() {
                Ok(event) => {
                    if let Err(error) = mouse_down(&state, &event) {
                        state.host.report_error(error);
                    }
                }
                Err(_) => state
                    .host
                    .report_error(error("selection requires MouseEvent")),
            }
        }
    })?;
    state.listeners.borrow_mut().push(listener);
    Ok(())
}
fn mouse_down(state: &Rc<State>, event: &MouseEvent) -> Result<(), JsValue> {
    state.down_timestamp.set(event.time_stamp());
    let dimensions = state.dimensions()?;
    if event.button() == 2
        && state
            .selection
            .has_selection(dimensions)
            .map_err(selection_error)?
    {
        return Ok(());
    }
    if event.button() != 0 {
        return Ok(());
    }
    let options = state.host.options()?;
    if options.mouse.mouse_events_require_alt
        && options.mouse.mouse_events_active
        && event.alt_key()
    {
        return Ok(());
    }
    if !state.selection.is_enabled() {
        if !pointer::should_force_selection(options.platform, options.mouse, modifiers(event)) {
            return Ok(());
        }
        event.stop_propagation();
    }
    event.prevent_default();
    state.drag_amount.set(0.0);
    if state.selection.is_enabled() && event.shift_key() {
        state
            .selection
            .incremental_click(state.buffer_coords(event, true, false)?);
    } else {
        match event.detail() {
            1 => state
                .selection
                .single_click(
                    &state.buffers,
                    dimensions,
                    state.buffer_coords(event, true, false)?,
                    pointer::should_column_select(
                        options.platform,
                        options.mouse,
                        modifiers(event),
                    ),
                )
                .map_err(selection_error)?,
            2 => {
                if select_word(state, event, true, &options)? {
                    state.selection.set_mode(SelectionMode::Word);
                }
            }
            3 => {
                if let Some(position) = state.buffer_coords(event, true, false)? {
                    state.selection.set_mode(SelectionMode::Line);
                    let active = state.buffers.buffer();
                    let buffer = active.try_borrow().map_err(error)?;
                    state
                        .selection
                        .select_line_at(
                            &buffer,
                            dimensions,
                            usize::try_from(position.row).map_err(error)?,
                        )
                        .map_err(selection_error)?;
                }
            }
            _ => {}
        }
    }
    schedule::drag(state)?;
    state
        .selection
        .refresh(&state.buffers, dimensions, true)
        .map_err(selection_error)
}
fn select_word(
    state: &Rc<State>,
    event: &MouseEvent,
    allow_whitespace: bool,
    options: &SelectionDomOptions,
) -> Result<bool, JsValue> {
    if let Some(link) = state.host.current_link()? {
        state.selection.update_model(|model| {
            model.selection_start = Some(link.start);
            model.selection_start_length = link.length;
            model.selection_end = None;
        });
        return Ok(true);
    }
    let Some(position) = state.buffer_coords(event, true, false)? else {
        return Ok(false);
    };
    let active = state.buffers.buffer();
    let buffer = active.try_borrow().map_err(error)?;
    state
        .selection
        .select_word_at(
            &buffer,
            state.dimensions()?,
            position,
            &options.word_separator,
            allow_whitespace,
        )
        .map_err(selection_error)?;
    state
        .selection
        .update_model(|model| model.selection_end = None);
    Ok(true)
}
pub(super) fn mouse_move(state: &Rc<State>, event: &MouseEvent) -> Result<(), JsValue> {
    event.stop_immediate_propagation();
    if state.selection.model().selection_start.is_none() {
        return Ok(());
    }
    let options = state.host.options()?;
    let position = state.buffer_coords(event, true, false)?;
    let amount = if position.is_some() {
        let geometry = (state.geometry)()?;
        let rect = state.screen.get_bounding_client_rect();
        let offset = f64::from(event.client_y()) - rect.top() - geometry.padding_top;
        let amount = pointer::drag_scroll_amount(offset, geometry.canvas_height);
        state.drag_amount.set(amount);
        amount
    } else {
        state.drag_amount.get()
    };
    state
        .selection
        .drag_move(
            &state.buffers,
            state.dimensions()?,
            position,
            amount,
            &options.word_separator,
        )
        .map_err(selection_error)
}
pub(super) fn mouse_up(state: &Rc<State>, event: &MouseEvent) -> Result<(), JsValue> {
    let elapsed = event.time_stamp() - state.down_timestamp.get();
    state.stop_drag()?;
    let dimensions = state.dimensions()?;
    let text_length = {
        let active = state.buffers.buffer();
        let buffer = active.try_borrow().map_err(error)?;
        state
            .selection
            .selection_text(&buffer, dimensions)
            .map_err(selection_error)?
            .len()
    };
    let options = state.host.options()?;
    if text_length <= 1
        && elapsed < pointer::ALT_CLICK_MOVE_CURSOR_TIME_MS
        && event.alt_key()
        && options.alt_click_moves_cursor
    {
        let at_bottom = {
            let active = state.buffers.buffer();
            let buffer = active.try_borrow().map_err(error)?;
            buffer.ybase == buffer.ydisp
        };
        if at_bottom {
            if let Some(position) = state.buffer_coords(event, false, true)? {
                state
                    .host
                    .move_cursor(position, options.application_cursor_keys)?;
            }
        }
    } else {
        state
            .selection
            .fire_if_changed(dimensions)
            .map_err(selection_error)?;
    }
    Ok(())
}
pub(super) fn right_click(state: &Rc<State>, event: &MouseEvent) -> Result<(), JsValue> {
    if state.disposed.get() {
        return Err(error("Selection component is disposed"));
    }
    let dimensions = state.dimensions()?;
    let inside = match state.buffer_coords(event, true, false)? {
        Some(position) => state
            .selection
            .is_cell_in_selection(position, dimensions)
            .map_err(selection_error)?,
        None => false,
    };
    if !inside {
        if select_word(state, event, false, &state.host.options()?)? {
            state
                .selection
                .refresh(&state.buffers, dimensions, true)
                .map_err(selection_error)?;
        }
        state
            .selection
            .fire_if_changed(dimensions)
            .map_err(selection_error)?;
    }
    Ok(())
}
