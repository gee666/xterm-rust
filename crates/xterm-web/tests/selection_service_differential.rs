// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Replays operations/observations captured from all unchanged source unit cases.
use std::{cell::RefCell, num::NonZeroU32, rc::Rc};
use xterm_core::{
    buffer::{BufferLine, BufferService, CellData, CharData},
    options::{OptionValue, OptionsService},
};
use xterm_web::selection::{
    model::{Dimensions, Position, SelectionModel},
    service::{SelectionMode, SelectionPlatform, SelectionService},
};

fn units(text: &str) -> Vec<u16> {
    if text == "-" {
        return Vec::new();
    }
    text.split('.')
        .map(|unit| u16::from_str_radix(unit, 16).unwrap())
        .collect()
}
fn position(column: &str, row: &str) -> Option<Position> {
    (column != "-").then(|| Position::new(column.parse().unwrap(), row.parse().unwrap()))
}
fn model(text: &str) -> SelectionModel {
    let fields: Vec<_> = text.split(',').collect();
    SelectionModel {
        is_select_all_active: fields[0] == "1",
        selection_start_length: fields[1].parse().unwrap(),
        selection_start: position(fields[2], fields[3]),
        selection_end: position(fields[4], fields[5]),
    }
}
fn encoded_position(position: Option<Position>) -> String {
    position.map_or_else(
        || "-,-".into(),
        |point| format!("{},{}", point.column, point.row),
    )
}
fn mode(text: &str) -> SelectionMode {
    match text {
        "0" => SelectionMode::Normal,
        "1" => SelectionMode::Word,
        "2" => SelectionMode::Line,
        "3" => SelectionMode::Column,
        _ => panic!("invalid mode {text}"),
    }
}

#[test]
fn faithfully_replay_source_suite_operations() {
    let mut count = 0;
    let cases = include_str!("fixtures/selection-service.tsv")
        .lines()
        .chain(include_str!("fixtures/selection-cache-regression.tsv").lines());
    for (index, row) in cases.enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 15, "case{index}");
        let columns: u32 = fields[1].parse().unwrap();
        let rows: u32 = fields[2].parse().unwrap();
        let scrollback: u32 = fields[3].parse().unwrap();
        let dimensions = Dimensions {
            columns: NonZeroU32::new(columns).unwrap(),
            rows: NonZeroU32::new(rows).unwrap(),
            scrollback,
        };
        let options = Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(f64::from(columns))),
                ("rows", OptionValue::Number(f64::from(rows))),
            ],
        ));
        let buffers = BufferService::new(options).unwrap();
        let active = buffers.buffer();
        {
            let mut buffer = active.borrow_mut();
            let rows: Vec<_> = if fields[8].is_empty() {
                Vec::new()
            } else {
                fields[8].split('|').collect()
            };
            buffer.lines.set_length(rows.len());
            buffer.ybase = usize::try_from(scrollback).unwrap();
            for (index, row) in rows.into_iter().enumerate() {
                let (wrapped, cells) = row.split_once(':').unwrap();
                let cells: Vec<_> = if cells.is_empty() {
                    Vec::new()
                } else {
                    cells.split(';').collect()
                };
                let mut line = BufferLine::new(cells.len(), None, wrapped == "1");
                for (index, cell) in cells.into_iter().enumerate() {
                    let (width, chars) = cell.split_once(',').unwrap();
                    let chars = units(chars);
                    let cell = CellData::from_char_data(&CharData {
                        attr: 0,
                        code: chars.last().copied().map(u32::from),
                        chars,
                        width: width.parse().unwrap(),
                    });
                    line.set_cell(index, &cell);
                }
                buffer
                    .lines
                    .set(index, Some(Rc::new(RefCell::new(line))))
                    .unwrap();
            }
        }
        let service = SelectionService::new(SelectionPlatform::default());
        service.set_mode(mode(fields[4]));
        service.update_model(|state| *state = model(fields[7]));
        let args: Vec<_> = fields[5].split(',').collect();
        match fields[0] {
            "word" => service
                .select_word_at(
                    &active.borrow(),
                    dimensions,
                    Position::new(args[0].parse().unwrap(), args[1].parse().unwrap()),
                    &units(fields[6]),
                    args[2] == "1",
                )
                .unwrap(),
            "toword" => service
                .select_to_word_at(
                    &active.borrow(),
                    dimensions,
                    Position::new(args[0].parse().unwrap(), args[1].parse().unwrap()),
                    &units(fields[6]),
                )
                .unwrap(),
            "line" => service
                .select_line_at(&active.borrow(), dimensions, args[0].parse().unwrap())
                .unwrap(),
            "all" => service.select_all(&buffers, dimensions).unwrap(),
            "lines" => service
                .select_lines(
                    &buffers,
                    dimensions,
                    args[0].parse().unwrap(),
                    args[1].parse().unwrap(),
                )
                .unwrap(),
            "text" | "has" => {}
            other => panic!("unknown operation {other}"),
        }
        assert_eq!(
            service.model(),
            model(fields[9]),
            "case{index} raw model, op{}",
            fields[0]
        );
        assert_eq!(
            encoded_position(service.selection_start()),
            fields[10],
            "case{index} final start"
        );
        assert_eq!(
            encoded_position(service.selection_end(dimensions).unwrap()),
            fields[11],
            "case{index} final end"
        );
        assert_eq!(
            service.has_selection(dimensions).unwrap(),
            fields[12] == "1",
            "case{index} has selection"
        );
        assert_eq!(
            service
                .selection_text(&active.borrow(), dimensions)
                .unwrap(),
            units(fields[13]),
            "case{index} text"
        );
        count += 1;
    }
    assert_eq!(count, 4298, "source operation inventory changed");
}
