// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Independent real-terminal queue traces from scripts/native-queue-oracle.mjs.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    headless::{QueuedTerminal, Terminal},
    input::write_buffer::{WriteCallback, WriteData},
    options::OptionValue,
};
type Log = Rc<RefCell<Vec<String>>>;
fn units(value: &str) -> Vec<u16> {
    if value == "-" {
        vec![]
    } else {
        value
            .split(',')
            .map(|s| u16::from_str_radix(s, 16).unwrap())
            .collect()
    }
}
fn hex(units: &[u16]) -> String {
    if units.is_empty() {
        "-".into()
    } else {
        units
            .iter()
            .map(|u| format!("{u:x}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}
fn host(log: &Log) -> QueuedTerminal {
    let host = QueuedTerminal::new(Rc::new(Terminal::new(10, 3, 10).unwrap())).unwrap();
    let output = log.clone();
    host.terminal()
        .on_write_parsed()
        .subscribe(move |_| output.borrow_mut().push("parsed".into()));
    let output = log.clone();
    host.terminal().on_resize().subscribe(move |event| {
        output
            .borrow_mut()
            .push(format!("resize:{},{}", event.cols, event.rows))
    });
    let output = log.clone();
    host.terminal()
        .on_data()
        .subscribe(move |data| output.borrow_mut().push(format!("data:{}", hex(data))));
    host
}
fn callback(
    log: &Log,
    label: String,
) -> Option<WriteCallback<xterm_core::headless::NativeWriteScheduler>> {
    let log = log.clone();
    Some(Rc::new(move |_| {
        log.borrow_mut().push(format!("callback:{label}"));
        Ok(())
    }))
}
fn snapshot(host: &QueuedTerminal, log: &Log) -> Vec<String> {
    let terminal = host.terminal();
    let buffer = terminal.buffer().unwrap().active().unwrap();
    let len = buffer.len().unwrap();
    let lines = (0..len)
        .map(|i| {
            let row = buffer.get_line(i).unwrap().unwrap();
            let text = row.borrow().translate_to_string(true, None, None, None);
            hex(&text)
        })
        .collect::<Vec<_>>()
        .join(";");
    vec![
        terminal.cols().unwrap().to_string(),
        terminal.rows().unwrap().to_string(),
        buffer.cursor_x().unwrap().to_string(),
        buffer.cursor_y().unwrap().to_string(),
        buffer.base_y().unwrap().to_string(),
        buffer.viewport_y().unwrap().to_string(),
        len.to_string(),
        lines,
        host.queue().pending_units().to_string(),
        host.queue().buffer_offset().to_string(),
        host.queue().buffered_chunks().to_string(),
        u8::from(host.queue().has_scheduled_write()).to_string(),
        if log.borrow().is_empty() {
            "-".into()
        } else {
            log.borrow().join("|")
        },
    ]
}
#[test]
fn real_native_queued_terminal_matches_original_headless_operation_traces() {
    let fixture = include_str!("fixtures/native-queue.tsv");
    assert_eq!(fixture.lines().count(), 24);
    let mut log = Log::default();
    let mut current = host(&log);
    for row in fixture.lines() {
        let f: Vec<_> = row.split('\t').collect();
        assert_eq!(f.len(), 18);
        if f[1] == "0" {
            log = Log::default();
            current = host(&log);
        }
        let label = format!("{},{}", f[0], f[1]);
        match f[2] {
            "text" => current
                .write(WriteData::Text(units(f[3])), callback(&log, label))
                .unwrap(),
            "bytes" => current
                .write(
                    WriteData::Bytes(units(f[3]).into_iter().map(|u| u as u8).collect()),
                    callback(&log, label),
                )
                .unwrap(),
            "writeln" => current
                .writeln(WriteData::Text(units(f[3])), callback(&log, label))
                .unwrap(),
            "resize" => current
                .resize(f[3].parse().unwrap(), f[4].parse().unwrap())
                .unwrap(),
            "input" => current.terminal().input(&units(f[3]), true).unwrap(),
            "disable" => current
                .terminal()
                .options()
                .set("disableStdin", OptionValue::Bool(true))
                .unwrap(),
            "tick" => {
                current.pump().poll().unwrap();
            }
            op => panic!("unknown operation {op}"),
        }
        assert_eq!(
            snapshot(&current, &log),
            f[5..],
            "scenario{} step{} op{}",
            f[0],
            f[1],
            f[2]
        );
    }
}
