// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Same operations and complete mode snapshots as the unchanged live JS terminal.
use std::collections::BTreeMap;
use xterm_core::{
    options::OptionValue,
    terminal::{CoreTerminal, TerminalEvent},
};

fn numbers(values: &[u32]) -> String {
    if values.is_empty() {
        "-".into()
    } else {
        values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
fn units(values: &[u16]) -> String {
    if values.is_empty() {
        "-".into()
    } else {
        values
            .iter()
            .map(|unit| format!("{unit:x}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}
fn snapshot(terminal: &mut CoreTerminal) -> String {
    let replies = terminal
        .drain_events()
        .into_iter()
        .filter_map(|event| {
            if let TerminalEvent::Data(data) = event {
                Some(units(&data))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let state = terminal.state();
    let modes = state.core.dec_private_modes.borrow();
    let kitty = state.core.kitty_keyboard.borrow();
    let flags = [
        state.core.modes.borrow().insert_mode,
        modes.application_cursor_keys,
        modes.application_keypad,
        modes.bracketed_paste_mode,
        modes.color_scheme_updates,
        modes.origin,
        modes.reverse_wraparound,
        modes.send_focus,
        modes.synchronized_output,
        modes.win32_input_mode,
        modes.wraparound,
        state.core.is_cursor_hidden.get(),
    ]
    .map(|value| u8::from(value).to_string())
    .join(",");
    let active = state.active_buffer();
    let buffer = active.borrow();
    let alternate = std::rc::Rc::ptr_eq(&active, &state.buffers.alt());
    [
        flags,
        state.mouse.active_protocol(),
        state.mouse.active_encoding(),
        u8::from(state.options.get("cursorBlink").unwrap().is_truthy()).to_string(),
        u8::from(state.options.get("convertEol").unwrap().is_truthy()).to_string(),
        u8::from(alternate).to_string(),
        state.cols().to_string(),
        state.rows().to_string(),
        buffer.x.to_string(),
        buffer.y.to_string(),
        buffer.ybase.to_string(),
        buffer.ydisp.to_string(),
        kitty.flags.to_string(),
        kitty.main_flags.to_string(),
        kitty.alt_flags.to_string(),
        numbers(&kitty.main_stack),
        numbers(&kitty.alt_stack),
        if replies.is_empty() {
            "-".into()
        } else {
            replies.join(";")
        },
    ]
    .join("|")
}

#[test]
fn live_terminal_modes_match_all_source_operations_and_replies() {
    let mut terminal = CoreTerminal::new(80, 30, 1000).unwrap();
    terminal.drain_events();
    let mut bags: BTreeMap<String, BTreeMap<String, OptionValue>> = BTreeMap::new();
    let mut count = 0;
    for (index, row) in include_str!("fixtures/terminal-modes.tsv")
        .lines()
        .enumerate()
    {
        let (operation, expected) = row.split_once('\t').unwrap();
        let fields = operation.split(':').collect::<Vec<_>>();
        match fields[0] {
            "w" => {
                let units = fields[1]
                    .split(',')
                    .map(|value| u16::from_str_radix(value, 16).unwrap())
                    .collect::<Vec<_>>();
                terminal.write_utf16(&units).unwrap();
            }
            "o" => {
                let bag = bags.entry(fields[1].into()).or_default();
                bag.insert(fields[2].into(), OptionValue::Bool(fields[3] == "1"));
                terminal
                    .state()
                    .options
                    .set(
                        fields[1],
                        OptionValue::property_map(
                            bag.iter().map(|(key, value)| (key.clone(), value.clone())),
                        ),
                    )
                    .unwrap();
            }
            _ => panic!("unknown source fixture operation"),
        }
        assert_eq!(
            snapshot(&mut terminal),
            expected,
            "mode operation {index}: {operation}"
        );
        count += 1;
    }
    assert_eq!(count, 2136, "source operation inventory changed");
}
