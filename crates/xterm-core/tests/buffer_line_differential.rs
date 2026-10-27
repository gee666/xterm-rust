// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::buffer::{AttributeData, BufferLine, CellData, CharData, ExtendedAttrs};

fn units(value: &str) -> Vec<u16> {
    if value == "-" {
        return Vec::new();
    }
    value
        .split(',')
        .map(|n| u16::from_str_radix(n, 16).unwrap())
        .collect()
}

fn hex(value: &[u16]) -> String {
    if value.is_empty() {
        return "-".to_owned();
    }
    value
        .iter()
        .map(|n| format!("{n:x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn number(value: &str) -> u32 {
    value.parse().unwrap()
}

fn snapshot(line: &BufferLine, start: usize, end: usize) -> Vec<String> {
    let mut cells = Vec::new();
    for i in 0..line.len() {
        let cell = line.get_cell(i).unwrap();
        let numbers = [
            i64::from(cell.content),
            i64::from(line.get_fg(i).unwrap()),
            i64::from(line.get_bg(i).unwrap()),
            i64::from(cell.extended.ext()),
            i64::from(cell.extended.url_id()),
            i64::from(line.get_code_point(i).unwrap()),
            i64::from(line.get_width(i).unwrap()),
            i64::from(line.has_width(i).unwrap()),
            i64::from(line.has_content(i).unwrap()),
            i64::from(line.is_combined(i).unwrap()),
            i64::from(line.is_protected(i).unwrap()),
        ];
        cells.push(format!(
            "{}:{}",
            numbers.map(|n| n.to_string()).join(","),
            hex(&line.get_string(i).unwrap())
        ));
    }
    let mut columns = Vec::new();
    let full = line.translate_to_string(false, Some(0), Some(line.len()), Some(&mut columns));
    let trimmed = line.translate_to_string(true, Some(0), Some(line.len()), None);
    let mut range_columns = Vec::new();
    let range = line.translate_to_string(false, Some(start), Some(end), Some(&mut range_columns));
    let mut trim_columns = Vec::new();
    let trim_range =
        line.translate_to_string(true, Some(start), Some(end), Some(&mut trim_columns));
    vec![
        if cells.is_empty() {
            "-".into()
        } else {
            cells.join("|")
        },
        line.get_trimmed_length().to_string(),
        line.get_no_bg_trimmed_length().to_string(),
        hex(&full),
        hex(&trimmed),
        if columns.is_empty() {
            "-".into()
        } else {
            columns
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        },
        hex(&line.translate_to_string(false, None, None, None)),
        hex(&line.translate_to_string(true, None, None, None)),
        u8::from(line.is_wrapped).to_string(),
        hex(&range),
        range_columns
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(","),
        hex(&trim_range),
        trim_columns
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(","),
    ]
}

#[test]
fn seeded_buffer_line_operations_match_javascript() {
    let data = include_str!("fixtures/buffer-line-traces.tsv");
    assert_eq!(data.lines().count(), 8192);
    let mut line = BufferLine::new(0, None, false);
    let mut counts = [0_usize; 14];
    for row in data.lines() {
        let f: Vec<_> = row.split('\t').collect();
        assert_eq!(f.len(), 28);
        let trace = number(f[0]);
        let step = number(f[1]);
        if step == 0 {
            line = BufferLine::new(12, None, trace % 2 == 1);
        }
        let a = number(f[3]) as usize;
        let b = number(f[4]) as usize;
        let c = number(f[5]) as usize;
        let flag = f[6] == "1";
        let width = f[11].parse::<i32>().unwrap();
        let mut cell = CellData::from_char_data(&CharData {
            attr: number(f[7]),
            chars: units(f[12]),
            width,
            code: Some(0),
        });
        cell.attributes = AttributeData {
            fg: number(f[7]),
            bg: number(f[8]),
            extended: ExtendedAttrs::new(number(f[9]), number(f[10])),
        };
        let cp = number(f[13]);
        let mut result = "-".to_owned();
        let op = match f[2] {
            "cell" => {
                line.set_cell(a, &cell);
                0
            }
            "legacy" => {
                line.set(a, &cell.get_as_char_data());
                1
            }
            "codepoint" => {
                line.set_cell_from_codepoint(a, cp, width, &cell);
                2
            }
            "add" => {
                line.add_codepoint_to_cell(a, cp, width);
                3
            }
            "insert" => {
                line.insert_cells(a, b, &cell);
                4
            }
            "delete" => {
                line.delete_cells(a, b, &cell);
                5
            }
            "replace" => {
                line.replace_cells(a, b, &cell, flag);
                6
            }
            "resize" => {
                result = u8::from(line.resize(b, &cell)).to_string();
                7
            }
            "fill" => {
                line.fill(&cell, flag);
                8
            }
            "within" => {
                line.copy_cells_within(a, b, c, flag);
                9
            }
            "clone" => {
                line = line.clone_line(false);
                10
            }
            "copy" => {
                let src = line;
                line = BufferLine::new(step as usize % 5, None, false);
                line.copy_from(&src, false);
                11
            }
            "other" => {
                let src = BufferLine::new(line.len(), Some(&cell), flag);
                line.copy_cells_from(&src, a, b, c, flag);
                12
            }
            "cleanup" => {
                result = u8::from(line.cleanup_memory()).to_string();
                13
            }
            op => panic!("unknown fixture op {op}"),
        };
        counts[op] += 1;
        assert_eq!(
            snapshot(&line, a, b),
            f[14..27],
            "trace {trace} step {step} op {}",
            f[2]
        );
        assert_eq!(
            result, f[27],
            "return trace {trace} step {step} op {}",
            f[2]
        );
    }
    assert!(counts.iter().all(|&n| n >= 584));
}

#[test]
fn shared_extended_attributes_survive_all_copy_paths() {
    let mut cell = CellData {
        attributes: AttributeData {
            bg: xterm_core::buffer::constants::bg_flags::HAS_EXTENDED,
            ..AttributeData::default()
        },
        ..CellData::default()
    };
    cell.extended.set_payload(Some(std::rc::Rc::new("payload")));
    let mut line = BufferLine::new(4, Some(&cell), false);
    let cloned = line.clone_line(false);
    let mut copied = BufferLine::new(1, None, false);
    copied.copy_from(&line, false);
    let mut other = BufferLine::new(4, None, false);
    other.copy_cells_from(&line, 0, 0, 4, false);
    line.copy_cells_within(0, 1, 3, true);
    cell.extended.set_url_id(19);
    for target in [&line, &cloned, &copied, &other] {
        for i in 0..target.len() {
            let loaded = target.get_cell(i).unwrap();
            assert!(loaded.extended.same_identity(&cell.extended));
            assert_eq!(loaded.extended.url_id(), 19);
            assert!(std::rc::Rc::ptr_eq(
                &loaded.extended.payload().unwrap(),
                &cell.extended.payload().unwrap()
            ));
        }
    }
}
