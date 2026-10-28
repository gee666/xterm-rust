// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use super::terminal_snapshot::{
    event as event_label, events, hex, parse_hex, snapshot, snapshot_state,
};
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    options::OptionValue,
    terminal::{CoreTerminal, TerminalEvent},
};

type TraceRows = Vec<(Vec<String>, Vec<String>)>;

/// Return independent expected/actual records. Expected values never influence
/// terminal state; the recorder example prints only the actual side.
pub fn replay() -> Result<TraceRows, Box<dyn std::error::Error>> {
    let mut terminal: Option<CoreTerminal> = None;
    let mut previous = String::new();
    let callbacks = Rc::new(RefCell::new(Vec::<String>::new()));
    let ordered_events = Rc::new(RefCell::new(Vec::<String>::new()));
    let mut output = Vec::new();
    for row in include_str!("../fixtures/input-handler-core.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 10 || fields[5] != "W" {
            return Err("invalid input handler fixture record".into());
        }
        if previous != fields[0] {
            previous = fields[0].into();
            let mut next =
                CoreTerminal::new(fields[1].parse()?, fields[2].parse()?, fields[3].parse()?)?;
            let options = next.state().options.clone();
            options.set("screenReaderMode", OptionValue::Bool(fields[4] == "1"))?;
            // A callback must not keep its owning terminal state alive in a cycle.
            let weak = Rc::downgrade(&next.state_handle());
            let saved = Rc::clone(&callbacks);
            let ordered = Rc::clone(&ordered_events);
            next.set_event_handler(Some(Rc::new(move |event| {
                if let Some(label) = event_label(&event) {
                    ordered.borrow_mut().push(label);
                }
                let label = match event {
                    TerminalEvent::AccessibilityChar(text) => format!("C{}", hex(&text)),
                    TerminalEvent::AccessibilityTab(count) => format!("T{count}"),
                    _ => return Ok(()),
                };
                let state = weak.upgrade().ok_or_else(|| {
                    xterm_core::event::EmitError::Callback(Rc::new(std::io::Error::other(
                        "terminal state expired during callback",
                    )))
                })?;
                saved
                    .borrow_mut()
                    .push(format!("{label}:{}", snapshot_state(&state.borrow())));
                Ok(())
            })));
            terminal = Some(next);
        }
        // The first nonempty fixture ID constructs a terminal before use.
        let terminal = terminal.as_mut().ok_or("missing terminal fixture setup")?;
        callbacks.borrow_mut().clear();
        ordered_events.borrow_mut().clear();
        terminal.drain_events();
        terminal.write_utf16(&parse_hex(fields[6]))?;
        let mut actual = fields[..7]
            .iter()
            .map(|field| (*field).to_owned())
            .collect::<Vec<_>>();
        actual.push(snapshot(terminal));
        if !events(terminal).is_empty() {
            return Err("registered terminal events were not delivered to the observer".into());
        }
        actual.push(ordered_events.borrow().join("/"));
        actual.push(callbacks.borrow().join("~"));
        output.push((
            fields.iter().map(|field| (*field).to_owned()).collect(),
            actual,
        ));
    }
    Ok(output)
}
