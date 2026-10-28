// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. All original command inputs and complete source snapshots.

use super::{parse_command, KittyCommand};
use std::collections::BTreeMap;

fn units(data: &[u16]) -> String {
    data.iter()
        .map(|unit| format!("{unit:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn snapshot(command: &KittyCommand) -> String {
    let mut fields = BTreeMap::new();
    macro_rules! number {
        ($native:ident, $js:literal) => {
            if let Some(value) = command.$native {
                fields.insert($js, format!("n:{:016x}", value.to_bits()));
            }
        };
    }
    macro_rules! string {
        ($native:ident, $js:literal) => {
            if let Some(value) = &command.$native {
                fields.insert($js, format!("s:{}", units(value)));
            }
        };
    }
    string!(action, "action");
    string!(compression, "compression");
    string!(transmission, "transmission");
    string!(delete_selector, "deleteSelector");
    number!(format, "format");
    number!(id, "id");
    number!(image_number, "imageNumber");
    number!(width, "width");
    number!(height, "height");
    number!(x, "x");
    number!(y, "y");
    number!(source_width, "sourceWidth");
    number!(source_height, "sourceHeight");
    number!(x_offset, "xOffset");
    number!(y_offset, "yOffset");
    number!(columns, "columns");
    number!(rows, "rows");
    number!(more, "more");
    number!(quiet, "quiet");
    number!(cursor_movement, "cursorMovement");
    number!(z_index, "zIndex");
    number!(placement_id, "placementId");
    fields
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("|")
}

#[test]
fn all_original_and_seeded_commands_match_complete_javascript_fields() {
    let corpus = include_str!("../../../tests/fixtures/kitty-command.tsv");
    assert_eq!(corpus.lines().count(), 1411);
    for (index, row) in corpus.lines().enumerate() {
        let (input, expected) = row.split_once('\t').unwrap();
        let data: Vec<_> = input
            .split(',')
            .filter(|part| !part.is_empty())
            .map(|part| u16::from_str_radix(part, 16).unwrap())
            .collect();
        assert_eq!(
            snapshot(&parse_command(&data)),
            expected,
            "seed 0x6b677431 case {index}: {input}"
        );
    }
}

#[test]
fn empty_nan_signed_zero_and_utf16_strings_remain_distinct() {
    let absent = parse_command(&[]);
    assert!(absent.action.is_none());
    assert!(absent.format.is_none());
    let data: Vec<_> = "a=,f=,z=-0".encode_utf16().collect();
    let command = parse_command(&data);
    assert_eq!(command.action, Some(vec![]));
    assert!(command.format.unwrap().is_nan());
    assert_eq!(command.z_index.unwrap().to_bits(), (-0.0_f64).to_bits());
    let command = parse_command(&[97, 61, 0xd800, 61, 0xdfff]);
    assert_eq!(command.action, Some(vec![0xd800, 61, 0xdfff]));
}
