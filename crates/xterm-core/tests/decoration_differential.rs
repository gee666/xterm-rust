// MIT license. Expected states come from actual read-only JavaScript sources.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::Buffer,
    circular_list::ListChange,
    decoration::{Decoration, DecorationLayer, DecorationOptions, DecorationService},
    options::{OptionValue, OptionsService},
};
fn ids(items: impl Iterator<Item = Decoration>, all: &[Decoration]) -> String {
    let value = items
        .map(|d| all.iter().position(|a| a.ptr_eq(&d)).unwrap().to_string())
        .collect::<Vec<_>>()
        .join(",");
    if value.is_empty() {
        "-".into()
    } else {
        value
    }
}
fn snapshot(service: &DecorationService, all: &[Decoration]) -> String {
    let states = all
        .iter()
        .map(|d| {
            format!(
                "{},{},{}",
                d.marker().line_number(),
                u8::from(d.marker().is_disposed()),
                u8::from(d.is_disposed())
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    let states = if states.is_empty() {
        "-".into()
    } else {
        states
    };
    let order = ids(service.decorations().unwrap(), all);
    let mut lookups = Vec::new();
    for line in 0..20 {
        for x in [0.0, 2.0, 5.0, 8.0] {
            for layer in [
                None,
                Some(DecorationLayer::Bottom),
                Some(DecorationLayer::Top),
            ] {
                lookups.push(ids(service.get_decorations_at_cell(x, line, layer), all));
            }
        }
    }
    format!("{states}|{order}|{}", lookups.join(";"))
}
#[test]
fn all_1024_seeded_operations_match_actual_javascript() {
    let options = Rc::new(OptionsService::new(
        false,
        std::iter::empty::<(&str, OptionValue)>(),
    ));
    let mut buffer = Buffer::new(true, 80, 24, options).unwrap();
    let service = DecorationService::new();
    service.attach_to_buffer_lines(&buffer.lines);
    let mut all: Vec<Decoration> = Vec::new();
    let fixture = include_str!("fixtures/decoration.tsv");
    assert_eq!(fixture.lines().count(), 1024);
    for (index, row) in fixture.lines().enumerate() {
        let fields = row.split('\t').collect::<Vec<_>>();
        let op: usize = fields[0].parse().unwrap();
        let a: usize = fields[1].parse().unwrap();
        let b: usize = fields[2].parse().unwrap();
        match op {
            0 => {
                let mut options = DecorationOptions::new(buffer.add_marker(a as i64));
                options.x = Some(b as f64);
                options.width = Some((b + 1) as f64);
                options.height = Some(b as u32);
                options.layer = Some(if b % 2 == 1 {
                    DecorationLayer::Top
                } else {
                    DecorationLayer::Bottom
                });
                all.push(
                    service
                        .register_decoration(Rc::new(RefCell::new(options)))
                        .unwrap()
                        .unwrap(),
                );
            }
            1 if !all.is_empty() => all[a % all.len()].dispose().unwrap(),
            2 => buffer.lines.on_trim_emitter.fire(&b).unwrap(),
            3 => buffer
                .lines
                .on_insert_emitter
                .fire(&ListChange {
                    index: a,
                    amount: b,
                })
                .unwrap(),
            4 => buffer
                .lines
                .on_delete_emitter
                .fire(&ListChange {
                    index: a,
                    amount: b,
                })
                .unwrap(),
            5 => service.reset().unwrap(),
            6 if !all.is_empty() => {
                all[a % all.len()].options().borrow_mut().layer = Some(if b % 2 == 1 {
                    DecorationLayer::Top
                } else {
                    DecorationLayer::Bottom
                })
            }
            1 | 6 => {}
            _ => panic!("invalid fixture operation"),
        }
        service.line_cache().flush_pending().unwrap();
        assert_eq!(
            snapshot(&service, &all),
            fields[3],
            "operation {index}: {op}/{a}/{b}"
        );
    }
}
