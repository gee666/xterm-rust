// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use projection::{events, parse_hex, snapshot};
use std::rc::Rc;
use xterm_core::{
    terminal::CoreTerminal,
    unicode::{UnicodeCharProperties, UnicodeCharWidth, UnicodeVersionProvider},
};

struct Provider {
    first: u32,
    joined: u32,
}
impl UnicodeVersionProvider for Provider {
    fn version(&self) -> &str {
        "negative-target579"
    }
    fn wcwidth(&self, _: u32) -> UnicodeCharWidth {
        UnicodeCharWidth::Narrow
    }
    fn char_properties(&self, code: u32, _: UnicodeCharProperties) -> UnicodeCharProperties {
        let width = match code {
            65 => self.first,
            66 => self.joined,
            _ => 1,
        };
        UnicodeCharProperties::new(1, width, code == 66)
    }
}

#[test]
fn custom_provider_negative_and_valid_join_targets_match_real_source() {
    let mut active = None;
    let mut name = "";
    let mut cases = 0;
    let mut snapshots = 0;
    let mut failures = Vec::new();
    for (index, row) in include_str!("fixtures/print-negative-target579.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 8, "fixture row {}", index + 1);
        if name != fields[0] {
            name = fields[0];
            let mut terminal = CoreTerminal::new(6, 3, 4).unwrap();
            if fields[3] == "1" {
                terminal.write("\x1b[4h").unwrap();
            }
            terminal
                .state()
                .unicode
                .register(Rc::new(Provider {
                    first: fields[1].parse().unwrap(),
                    joined: fields[2].parse().unwrap(),
                }))
                .unwrap();
            terminal
                .state()
                .unicode
                .set_active_version("negative-target579")
                .unwrap();
            terminal.drain_events();
            active = Some(terminal);
            cases += 1;
        }
        let terminal = active.as_mut().unwrap();
        terminal.write_utf16(&parse_hex(fields[4])).unwrap();
        let actual = snapshot(terminal);
        let notifications = events(terminal);
        let preceding = terminal.parser_mut().preceding_join_state.get();
        if actual != fields[5]
            || notifications != fields[6]
            || preceding != fields[7].parse::<u32>().unwrap()
        {
            failures.push(format!(
                "{name} row {}: expected {} / {} / {}; actual {actual} / {notifications} / {preceding}",
                index + 1, fields[5], fields[6], fields[7]
            ));
        }
        snapshots += 1;
    }
    assert_eq!(cases, 64);
    assert_eq!(snapshots, 160);
    for failure in failures.iter().take(3) {
        eprintln!("{failure}");
    }
    assert_eq!(
        failures.len(),
        0,
        "{snapshots} snapshots, {} mismatches",
        failures.len()
    );
}
