//! Real parser/engine integration, not a simulation of terminal cell writes.
#[path = "support/terminal_snapshot.rs"]
mod projection;
#[path = "support/charset_trace.rs"]
mod trace;

use std::{cell::RefCell, collections::BTreeMap, fmt::Write, rc::Rc};
use trace::{canonical, custom_map, slots, PROFILES};
use xterm_core::{charset::builtin_charset, event::EmitError, terminal::TerminalEvent};

#[test]
fn callbacks_observe_pre_write_cells_and_live_captured_map_identity() {
    for (edit, fixture) in [
        (false, include_str!("fixtures/charset-callback.tsv")),
        (true, include_str!("fixtures/charset-mutation.tsv")),
    ] {
        let mut cases = 0;
        for row in fixture.lines() {
            let fields: Vec<_> = row.split('\t').collect();
            assert_eq!(fields.len(), 5);
            let custom = custom_map();
            let mut terminal = trace::terminal(8.0, 3.0);
            projection::events(&mut terminal);
            if edit {
                terminal
                    .state_handle()
                    .borrow_mut()
                    .charset
                    .set_g_charset(0, Some(custom.clone()))
                    .unwrap();
            }
            let observations = Rc::new(RefCell::new(Vec::new()));
            let recorded = Rc::clone(&observations);
            let state = Rc::downgrade(&terminal.state_handle());
            let callback_map = custom.clone();
            terminal.set_event_handler(Some(Rc::new(move |event| {
                if let TerminalEvent::AccessibilityChar(chars) = event {
                    let state = state.upgrade().expect("the active terminal owns its state");
                    let mut state = state.try_borrow_mut().map_err(EmitError::callback)?;
                    let active = state.active_buffer();
                    let buffer = active.borrow();
                    let value = buffer
                        .lines
                        .get(buffer.ybase + buffer.y)
                        .and_then(|line| line.borrow().get_cell(buffer.x))
                        .map_or_else(
                            || "missing".into(),
                            |cell| projection::hex(&cell.get_chars()),
                        );
                    let service = format!(
                        "{}/{}/{}",
                        state.charset.glevel,
                        canonical(&state.charset.charset, &callback_map),
                        slots(state.charset.charsets(), &callback_map)
                    );
                    recorded.borrow_mut().push(format!(
                        "{}/{}/{}/{}/{service}",
                        projection::hex(&chars),
                        buffer.x,
                        buffer.y,
                        value
                    ));
                    drop(buffer);
                    if recorded.borrow().len() == 1 {
                        if edit {
                            callback_map
                                .set(u16::from(b'q'), Some(vec![0x58]))
                                .map_err(EmitError::callback)?;
                        } else {
                            state
                                .charset
                                .set_g_charset(0, builtin_charset('A'))
                                .map_err(EmitError::callback)?;
                        }
                    }
                }
                Ok(())
            })));
            trace::write(&mut terminal, &projection::parse_hex(fields[1]), fields[0]).unwrap();
            assert_eq!(
                observations.borrow().join(";"),
                fields[2],
                "callback order {} edit{edit}",
                fields[0]
            );
            assert_eq!(
                projection::snapshot(&terminal),
                fields[3],
                "printed run {} edit{edit}",
                fields[0]
            );
            assert_eq!(trace::service(&terminal, &custom), fields[4]);
            cases += 1;
        }
        assert_eq!(cases, 3);
    }
}

#[test]
fn native_charset_states_and_ordered_events_match_each_source_stream_profile() {
    let custom = custom_map();
    let mut expected_events = BTreeMap::new();
    for row in include_str!("fixtures/charset-terminal-events.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 3);
        assert!(expected_events
            .insert((fields[0], fields[1]), fields[2])
            .is_none());
    }
    assert_eq!(expected_events.len(), 402);
    let mut failures = Vec::new();
    let mut report = String::from("case\tprofile\tstatus\n");
    let mut count = 0;
    for row in include_str!("fixtures/charset-terminal.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 11);
        let input = projection::parse_hex(fields[5]);
        for profile in PROFILES {
            let mut terminal =
                trace::terminal(fields[2].parse().unwrap(), fields[3].parse().unwrap());
            projection::events(&mut terminal);
            if fields[4] == "1" {
                terminal
                    .state_handle()
                    .borrow_mut()
                    .charset
                    .set_g_charset(0, Some(custom.clone()))
                    .unwrap();
            }
            trace::write(&mut terminal, &input, profile).unwrap();
            let actual = trace::capture(&mut terminal, &custom);
            let expected = &fields[6..];
            let events = expected_events.get(&(fields[0], profile)).unwrap();
            let passed = actual
                .state
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())
                && actual.events == *events;
            writeln!(
                report,
                "{}\t{}\t{}",
                fields[0],
                profile,
                if passed { "passed" } else { "failed" }
            )
            .unwrap();
            if !passed {
                let differences = actual
                    .state
                    .iter()
                    .zip(expected)
                    .enumerate()
                    .filter(|(_, (a, b))| a.as_str() != **b)
                    .map(|(field, (a, b))| format!("field{field}\nexpected:{b}\nactual:{a}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                failures.push(format!("case{} {} profile{profile}\n{differences}\nevents expected:{events}\nevents actual:{}\n", fields[0], fields[1], actual.events));
            }
            count += 1;
        }
    }
    assert_eq!(count, 402);
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/charset-terminal");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("rust-cases.tsv"), report).unwrap();
    std::fs::write(root.join("rust-failures.txt"), failures.join("\n")).unwrap();
    for failure in failures.iter().take(2) {
        eprintln!(
            "{}; full differences retained in rust-failures.txt",
            failure.lines().next().unwrap()
        );
    }
    assert!(failures.is_empty(), "{} / {count} charset engine runs differ; retained results/charset-terminal/rust-failures.txt", failures.len());
}
