// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use std::{
    cell::{Cell, RefCell},
    error::Error,
    rc::Rc,
};
use xterm_core::{
    event::EmitError, headless::Terminal, options::OptionValue, terminal::CoreTerminal,
};

const PRELUDE: &str = "AAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD";

#[test]
fn index_and_next_line_resume_after_live_scroll_and_marker_callbacks() {
    for (index, row) in include_str!("fixtures/index-callbacks.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 7);
        let terminal = Rc::new(Terminal::new(5, 3, 1).unwrap());
        terminal.write(PRELUDE).unwrap();
        // Both native entry points must start from the exact source precondition.
        let mut core = CoreTerminal::new(5, 3, 1).unwrap();
        core.write(PRELUDE).unwrap();
        assert_eq!(
            projection::snapshot(&core),
            terminal.with_state(projection::snapshot_state).unwrap()
        );
        assert!(!projection::events(&mut core).is_empty());
        let marker = terminal.register_marker(-3).unwrap();
        let records = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&records);
        let called = Cell::new(false);
        let weak = Rc::downgrade(&terminal);
        let action = fields[2].to_owned();
        let original = EmitError::callback(std::io::Error::other("index callback sentinel"));
        let captured = original.clone();
        let callback = move |_: &usize| {
            if called.replace(true) {
                return Ok(());
            }
            let terminal = weak.upgrade().unwrap();
            output.borrow_mut().push(
                terminal
                    .with_state(projection::snapshot_state)
                    .map_err(EmitError::callback)?,
            );
            match action.as_str() {
                "error" => return Err(captured.clone()),
                "reset" => terminal.reset().map_err(EmitError::callback)?,
                "inspect" => {
                    terminal
                        .options()
                        .set("convertEol", OptionValue::Bool(true))
                        .map_err(EmitError::callback)?;
                    terminal.register_marker(0).map_err(EmitError::callback)?;
                }
                other => panic!("unknown source action {other}"),
            }
            output.borrow_mut().push(
                terminal
                    .with_state(projection::snapshot_state)
                    .map_err(EmitError::callback)?,
            );
            Ok(())
        };
        let subscription = match fields[1] {
            "scroll" => terminal.on_scroll().try_subscribe(callback),
            "marker" => marker.on_dispose().try_subscribe(move |_| callback(&0)),
            other => panic!("unknown source trigger {other}"),
        };
        let renders = Rc::new(RefCell::new(Vec::new()));
        let render_output = Rc::clone(&renders);
        let render_subscription = terminal.on_render().subscribe(move |event| {
            render_output
                .borrow_mut()
                .push(format!("{}:{}", event.start, event.end));
        });
        let units = projection::parse_hex(fields[0]);
        assert_eq!(projection::hex(&units), fields[0]);
        let command = String::from_utf16(&units).unwrap() + "Z";
        let result = terminal.write(&command);
        if fields[2] == "error" {
            let error = result.unwrap_err();
            let mut source = &error as &dyn Error;
            let mut found = false;
            loop {
                if let Some(actual) = source.downcast_ref::<EmitError>() {
                    found |= actual == &original;
                }
                match source.source() {
                    Some(next) => source = next,
                    None => break,
                }
            }
            assert!(found, "case {index}: {error:?}");
            assert!(std::ptr::eq(
                source.downcast_ref::<std::io::Error>().unwrap(),
                original.callback_ref::<std::io::Error>().unwrap()
            ));
        } else {
            result.unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        }
        assert_eq!(
            u8::from(marker.is_disposed()).to_string(),
            fields[3],
            "case {index}"
        );
        assert_eq!(
            records.borrow().join("~"),
            fields[4],
            "case {index} callback state"
        );
        assert_eq!(
            terminal.with_state(projection::snapshot_state).unwrap(),
            fields[5],
            "case {index} final state"
        );
        assert_eq!(
            renders.borrow().join(","),
            fields[6],
            "case {index} render ranges"
        );
        render_subscription.dispose();
        subscription.dispose();
        terminal.dispose().unwrap();
    }
}
