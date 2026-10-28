// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Original Terminal2.test.ts erase fixtures at 80x25. This explicitly models
//! PTY ONLCR for native Rust replay; it is not an observed native PTY/browser run.
use xterm_core::terminal::CoreTerminal;

#[test]
fn original_applicable_erase_vt_text_assertions_with_explicit_onlcr() {
    let cases: [(&str, &[u8], &str); 4] = [
        (
            "t0054-ECH",
            include_bytes!("fixtures/erase-vt/t0054-ECH.in"),
            include_str!("fixtures/erase-vt/t0054-ECH.text"),
        ),
        (
            "t0056-ED",
            include_bytes!("fixtures/erase-vt/t0056-ED.in"),
            include_str!("fixtures/erase-vt/t0056-ED.text"),
        ),
        (
            "t0057-ED3",
            include_bytes!("fixtures/erase-vt/t0057-ED3.in"),
            include_str!("fixtures/erase-vt/t0057-ED3.text"),
        ),
        (
            "t0091-alt_screen_ED3",
            include_bytes!("fixtures/erase-vt/t0091-alt_screen_ED3.in"),
            include_str!("fixtures/erase-vt/t0091-alt_screen_ED3.text"),
        ),
    ];
    for (name, input, expected) in cases {
        let mut data = Vec::new();
        for &byte in input {
            if byte == b'\n' {
                data.push(b'\r');
            }
            data.push(byte);
        }
        let mut terminal = CoreTerminal::new(80, 25, 1000).unwrap();
        terminal.write_bytes(&data).unwrap();
        let active = terminal.state().active_buffer();
        let b = active.borrow();
        let mut actual = String::new();
        for y in b.ybase..b.ybase + 25 {
            let line = b.lines.get(y).unwrap();
            let line = line.borrow();
            let text =
                String::from_utf16(&line.translate_to_string(true, None, None, None)).unwrap();
            actual.push_str(text.trim_end());
            actual.push('\n');
        }
        let expected = expected
            .split('\n')
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(actual, expected, "unchanged upstream fixture {name}");
    }
    // Upstream Terminal2.test.ts explicitly skips t0055-EL because its pending
    // wrap behavior differs from xterm. csi-erase-fixtures.mjs still records the
    // exact unchanged JS result for Rust differential replay, not a fake pass.
}
