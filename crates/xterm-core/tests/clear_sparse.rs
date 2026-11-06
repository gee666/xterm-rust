// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use std::{
    cell::{Cell, RefCell},
    error::Error,
    rc::Rc,
};
use xterm_core::{buffer::LineRef, event::EmitError, headless::Terminal, terminal::CoreTerminal};

type Observations = Rc<RefCell<Vec<String>>>;
type Selected = Rc<RefCell<Option<LineRef>>>;
fn capture(terminal: &Terminal, events: &Observations, tag: &str) {
    let state = terminal.with_state(projection::snapshot_state).unwrap();
    events.borrow_mut().push(format!("{tag}#{state}"));
}
fn mutate(
    terminal: &Terminal,
    events: &Observations,
    selected: &Selected,
    pattern: &str,
    x: usize,
) {
    terminal
        .with_state(|state| {
            let active = state.active_buffer();
            let mut buffer = active.borrow_mut();
            buffer.x = x;
            buffer.y = if pattern == "outside" {
                state.rows() + 5
            } else {
                state.rows() - 1
            };
            let current = buffer.ybase + buffer.y;
            if pattern != "dense" {
                buffer.lines.set(current, None).unwrap();
            }
            if pattern == "truncated" {
                buffer.lines.set_length(current);
            }
            if pattern == "empty" {
                buffer.lines.set_length(0);
            }
            *selected.borrow_mut() = buffer.lines.get(current);
        })
        .unwrap();
    capture(terminal, events, "ready");
}
fn has_original(error: &(dyn Error + 'static), original: &EmitError) -> bool {
    let mut current = error;
    loop {
        if current.downcast_ref::<EmitError>() == Some(original) {
            return true;
        }
        match current.source() {
            Some(source) => current = source,
            None => return false,
        }
    }
}

#[test]
fn core_and_headless_clear_share_sparse_row_and_scroll_semantics() {
    let mut core = CoreTerminal::new(5, 3, 3).unwrap();
    let headless = Terminal::new(5, 3, 3).unwrap();
    let input = projection::parse_hex("1b,5b,33,31,6d,41,41,1b,5b,33,3b,32,48");
    core.write_utf16(&input).unwrap();
    headless.write_utf16(&input).unwrap();
    core.drain_events();
    let make_sparse = |state: &xterm_core::terminal::TerminalState| {
        let active = state.active_buffer();
        let buffer = active.borrow();
        buffer.lines.set(buffer.ybase + buffer.y, None).unwrap();
    };
    make_sparse(&core.state());
    headless.with_state(make_sparse).unwrap();
    core.clear().unwrap();
    headless.clear().unwrap();
    assert_eq!(projection::events(&mut core), "S0");
    assert_eq!(
        projection::snapshot(&core),
        headless.with_state(projection::snapshot_state).unwrap()
    );
    let state = core.state();
    let active = state.active_buffer();
    let buffer = active.borrow();
    assert!(buffer.lines.get(0).is_none());
    assert_eq!((buffer.x, buffer.y, buffer.lines.len()), (1, 0, 3));
    for row in 1..3 {
        assert_eq!(
            buffer.lines.get(row).unwrap().borrow().get_trimmed_length(),
            0
        );
    }
}

#[test]
fn unchanged_public_clear_assertions() {
    let terminal = Terminal::new(80, 5, 1000).unwrap();
    for index in 0..10 {
        terminal.write(&format!("\n\rtest{index}")).unwrap();
    }
    terminal.clear().unwrap();
    terminal
        .with_state(|state| {
            let active = state.active_buffer();
            let buffer = active.borrow();
            assert_eq!(buffer.lines.len(), 5);
            for row in 0..5 {
                let line = buffer.lines.get(row).unwrap();
                let text = line.borrow().translate_to_string(true, None, None, None);
                assert_eq!(
                    text,
                    if row == 0 { "test9" } else { "" }
                        .encode_utf16()
                        .collect::<Vec<_>>()
                );
            }
        })
        .unwrap();
}

#[test]
fn unchanged_public_clear_disposes_markers_assertions() {
    let terminal = Terminal::new(80, 5, 1000).unwrap();
    for index in 0..10 {
        terminal.write(&format!("\n\rtest{index}")).unwrap();
    }
    let markers: Vec<_> = (1..=4)
        .map(|offset| terminal.register_marker(offset).unwrap())
        .collect();
    let disposed = Rc::new(Cell::new(0));
    let subscriptions: Vec<_> = markers
        .iter()
        .map(|marker| {
            let count = Rc::clone(&disposed);
            marker
                .on_dispose()
                .subscribe(move |_| count.set(count.get() + 1))
        })
        .collect();
    terminal.clear().unwrap();
    assert_eq!(disposed.get(), markers.len());
    assert!(markers.iter().all(|marker| marker.is_disposed()));
    assert!(terminal.markers().unwrap().is_empty());
    for subscription in subscriptions {
        subscription.dispose();
    }
}

#[test]
fn clear_preserves_sparse_row_transfer_callback_order_and_errors() {
    let fixture = include_str!("fixtures/clear-sparse.tsv");
    assert_eq!(fixture.lines().count(), 280);
    let mut failures = Vec::new();
    for (index, row) in fixture.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 10);
        let alternate = fields[0] == "1";
        let rows = fields[1].parse().unwrap();
        let x = fields[2].parse().unwrap();
        let pattern = fields[3];
        let action = fields[4];
        let terminal = Rc::new(Terminal::new(5, rows, 3).unwrap());
        terminal
            .write(&format!(
                "{}\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD\r\nEEEEE",
                if alternate { "\x1b[?1049h" } else { "" }
            ))
            .unwrap();
        let marker = terminal.register_marker(0).unwrap();
        let second = terminal.register_marker(-1).unwrap();
        let events = Rc::new(RefCell::new(Vec::new()));
        let selected: Selected = Rc::new(RefCell::new(None));
        let ready = Rc::new(Cell::new(false));
        let original = EmitError::callback(std::io::Error::other("sparse clear sentinel"));
        let output = Rc::clone(&events);
        let weak = Rc::downgrade(&terminal);
        let throw = action == "scroll-error";
        let sentinel = original.clone();
        let scroll = terminal.on_scroll().try_subscribe(move |position| {
            let terminal = weak.upgrade().unwrap();
            capture(&terminal, &output, &format!("S{position}"));
            if throw {
                return Err(sentinel.clone());
            }
            Ok(())
        });
        let output = Rc::clone(&events);
        let weak = Rc::downgrade(&terminal);
        let resize = terminal.on_resize().subscribe(move |event| {
            let terminal = weak.upgrade().unwrap();
            capture(
                &terminal,
                &output,
                &format!("Z{}:{}", event.cols, event.rows),
            );
        });
        let output = Rc::clone(&events);
        let weak = Rc::downgrade(&terminal);
        let selected_row = Rc::clone(&selected);
        let is_ready = Rc::clone(&ready);
        let callback_action = action.to_owned();
        let callback_pattern = pattern.to_owned();
        let sentinel = original.clone();
        let first = marker.on_dispose().try_subscribe(move |_| {
            let terminal = weak.upgrade().unwrap();
            capture(&terminal, &output, "M1");
            if !is_ready.get() {
                match callback_action.as_str() {
                    "reset" => terminal.reset().map_err(EmitError::callback)?,
                    "resize" => terminal.resize(7, 6).map_err(EmitError::callback)?,
                    "activate" => {
                        let buffers = terminal.with_state(|s| Rc::clone(&s.buffers)).unwrap();
                        if alternate {
                            buffers.activate_normal_buffer()
                        } else {
                            buffers.activate_alt_buffer(None)
                        }
                        .map_err(EmitError::callback)?;
                    }
                    "callback" | "marker-error" => {}
                    other => panic!("unexpected callback action {other}"),
                }
                mutate(&terminal, &output, &selected_row, &callback_pattern, x);
                is_ready.set(true);
            }
            if callback_action == "marker-error" {
                return Err(sentinel.clone());
            }
            Ok(())
        });
        let output = Rc::clone(&events);
        let weak = Rc::downgrade(&terminal);
        let later = second.on_dispose().subscribe(move |_| {
            capture(&weak.upgrade().unwrap(), &output, "M2");
        });
        if action == "direct" || action == "scroll-error" {
            mutate(&terminal, &events, &selected, pattern, x);
            ready.set(true);
        }
        let result = terminal.clear();
        let error_matches = if fields[5] == "1" {
            result
                .as_ref()
                .err()
                .is_some_and(|error| has_original(error, &original))
        } else {
            result.is_ok()
        };
        capture(&terminal, &events, "final");
        let same = terminal
            .with_state(|state| {
                let first = state.active_buffer().borrow().lines.get(0);
                match (&*selected.borrow(), first) {
                    (Some(before), Some(after)) => Rc::ptr_eq(before, &after),
                    (None, None) => true,
                    _ => false,
                }
            })
            .unwrap();
        let actual = events.borrow().join("^");
        if !error_matches
            || same != (fields[6] == "1")
            || marker.is_disposed() != (fields[7] == "1")
            || second.is_disposed() != (fields[8] == "1")
            || actual != fields[9]
        {
            failures.push(format!(
                "case {index} {:?}: result={result:?} same={same} markers={},{}\nexpected {}\nactual {actual}",
                &fields[..9], marker.is_disposed(), second.is_disposed(), fields[9]
            ));
        }
        first.dispose();
        later.dispose();
        scroll.dispose();
        resize.dispose();
        terminal.dispose().unwrap();
    }
    let output =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/clear-sparse");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(output.join("counterexamples.txt"), failures.join("\n\n")).unwrap();
    std::fs::write(
        output.join("native-summary.txt"),
        format!(
            "cases=280\npassed={}\nfailed={}\n",
            280 - failures.len(),
            failures.len()
        ),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "{} mismatches; first={:?}",
        failures.len(),
        failures.first()
    );
}
