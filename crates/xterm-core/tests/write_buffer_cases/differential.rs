// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{cell::RefCell, rc::Rc};
use xterm_core::input::write_buffer::{WriteAction, WriteBuffer, WriteData};
fn data(kind: &str, value: &str) -> WriteData {
    let units = if value == "-" {
        vec![]
    } else {
        value
            .split(',')
            .map(|v| u16::from_str_radix(v, 16).unwrap())
            .collect()
    };
    if kind == "t" {
        WriteData::Text(units)
    } else {
        WriteData::Bytes(units.into_iter().map(|u| u as u8).collect())
    }
}
fn action(data: &WriteData, result: Option<bool>) -> String {
    let (kind, units) = match data {
        WriteData::Text(text) => ("t", text.clone()),
        WriteData::Bytes(bytes) => ("b", bytes.iter().map(|&b| u16::from(b)).collect()),
        WriteData::External(_) => panic!("the fixed corpus contains only native chunks"),
    };
    let text = if units.is_empty() && kind == "t" {
        "-".into()
    } else {
        units
            .iter()
            .map(|u| format!("{u:x}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "action:{kind}:{text}:{}",
        result.map_or("undefined", |v| if v { "true" } else { "false" })
    )
}
fn make(host: &Host, log: &Calls, cost: f64) -> WriteBuffer<Host> {
    let clock = host.clone();
    let output = log.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, data, result| {
        output.borrow_mut().push(action(data, result));
        clock.advance(cost);
        Ok(WriteAction::Complete)
    });
    let output = log.clone();
    queue
        .on_write_parsed()
        .subscribe(move |_| output.borrow_mut().push("parsed".into()));
    queue
}
#[test]
fn all_seeded_queue_actions_callbacks_events_and_offsets_match_source() {
    let fixture = include_str!("../fixtures/write-buffer-traces.tsv");
    assert_eq!(fixture.lines().count(), 8192);
    let mut host = Host::default();
    let mut log = Rc::new(RefCell::new(Vec::new()));
    let mut queue = make(&host, &log, 0.0);
    for row in fixture.lines() {
        let f: Vec<_> = row.split('\t').collect();
        assert_eq!(f.len(), 12);
        let trace = f[0].parse::<usize>().unwrap();
        let step = f[1].parse::<usize>().unwrap();
        if step == 0 {
            host = Host::default();
            host.advance(100.0);
            log = Calls::default();
            queue = make(&host, &log, f[5].parse().unwrap());
        }
        let value = data(f[3], f[4]);
        match f[2] {
            "write" => {
                let output = log.clone();
                let label = format!("callback:{trace},{step}");
                queue
                    .write(
                        value,
                        Some(Rc::new(move |_| {
                            output.borrow_mut().push(label.clone());
                            Ok(())
                        })),
                    )
                    .unwrap();
            }
            "sync" => queue.write_sync(value, None).unwrap(),
            "flush" => queue.flush_sync().unwrap(),
            "input" => queue.handle_user_input(),
            "tick" => {
                host.task().unwrap();
            }
            "dispose" => queue.dispose(),
            op => panic!("unknown operation {op}"),
        }
        let context = format!("trace{trace} step{step} op{}", f[2]);
        assert_eq!(
            queue.pending_units(),
            f[6].parse::<f64>().unwrap(),
            "{context}"
        );
        assert_eq!(
            queue.buffer_offset(),
            f[7].parse::<usize>().unwrap(),
            "{context}"
        );
        assert_eq!(
            queue.buffered_chunks(),
            f[8].parse::<usize>().unwrap(),
            "{context}"
        );
        assert_eq!(queue.is_disposed(), f[9] == "1", "{context}");
        assert_eq!(
            host.pending_tasks(),
            f[10].parse::<usize>().unwrap(),
            "{context}"
        );
        assert_eq!(
            if log.borrow().is_empty() {
                "-".into()
            } else {
                log.borrow().join("|")
            },
            f[11],
            "{context}"
        );
    }
}
