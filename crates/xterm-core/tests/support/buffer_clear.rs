// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use crate::projection;
use std::{cell::RefCell, rc::Rc};
use xterm_core::{event::EmitError, headless::Terminal};

type Observations = Rc<RefCell<Vec<String>>>;
fn capture(terminal: &Terminal, events: &Observations, tag: &str) -> Result<(), EmitError> {
    let state = terminal
        .with_state(projection::snapshot_state)
        .map_err(EmitError::callback)?;
    let markers = terminal.markers().map_err(EmitError::callback)?;
    let markers = if markers.is_empty() {
        "-".into()
    } else {
        markers
            .iter()
            .map(|marker| {
                format!(
                    "{},{}",
                    marker.line_number(),
                    u8::from(marker.is_disposed())
                )
            })
            .collect::<Vec<_>>()
            .join("|")
    };
    events.borrow_mut().push(format!("{tag}#{state}#{markers}"));
    Ok(())
}
/// Execute the same public marker callback as the source, retaining any native
/// exception and already-observed partial state rather than reporting no-op success.
pub fn trace(action: &str) -> String {
    let terminal = Rc::new(Terminal::new(8, 3, 4).unwrap());
    terminal.write("old0\r\nold1\r\nold2\r\nprompt").unwrap();
    let marker = terminal.register_marker(0).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = events.clone();
    let weak = Rc::downgrade(&terminal);
    let scroll_subscription = terminal.on_scroll().try_subscribe(move |position| {
        let terminal = weak
            .upgrade()
            .expect("terminal remains owned during synchronous callbacks");
        capture(&terminal, &output, &format!("scroll,{position}"))
    });
    let output = events.clone();
    let weak = Rc::downgrade(&terminal);
    let action = action.to_owned();
    let marker_subscription = marker.on_dispose().try_subscribe(move |_| {
        let terminal = weak
            .upgrade()
            .expect("terminal remains owned during synchronous callbacks");
        capture(&terminal, &output, "dispose")?;
        match action.as_str() {
            "read" => Ok(()),
            "scroll" => terminal.scroll_to_top(),
            "resize" => terminal.resize(4, 4),
            "clear" => terminal.clear(),
            "reset" => terminal.reset(),
            other => panic!("unknown fixture callback {other}"),
        }
        .map_err(EmitError::callback)?;
        capture(&terminal, &output, "callback-return")
    });
    if let Err(error) = terminal.clear() {
        events.borrow_mut().push(format!("!clear-error#{error}"));
    }
    if let Err(error) = capture(&terminal, &events, "final") {
        events.borrow_mut().push(format!("!capture-error#{error}"));
    }
    assert!(marker.is_disposed());
    if let Err(error) = terminal.dispose() {
        events.borrow_mut().push(format!("!dispose-error#{error}"));
    }
    scroll_subscription.dispose();
    marker_subscription.dispose();
    let result = events.borrow().join("^");
    result
}
