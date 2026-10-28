//! Raw values and ordered cache lookups captured from actual source Buffer/DecorationService.
use std::{cell::RefCell, fmt::Write as _, rc::Rc};
use xterm_core::{
    buffer::{marker::Marker, marker_line::MarkerLine, Buffer},
    circular_list::ListChange,
    decoration::{Decoration, DecorationError, DecorationOptions, DecorationService},
    options::{OptionValue, OptionsService},
};
fn decode(value: &str) -> MarkerLine {
    match value.as_bytes()[0] {
        b'n' => MarkerLine::number(match &value[1..] {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            number => number.parse().unwrap(),
        }),
        b's' if &value[1..] == "_" => MarkerLine::string(Vec::new()),
        b's' => MarkerLine::string(
            (1..value.len())
                .step_by(4)
                .map(|i| u16::from_str_radix(&value[i..i + 4], 16).unwrap())
                .collect(),
        ),
        _ => panic!("invalid raw decoration fixture"),
    }
}
fn encode(value: &MarkerLine) -> String {
    if let Some(number) = value.as_number() {
        if number == 0.0 && number.is_sign_negative() {
            return "n-0".into();
        }
        format!("n{}", String::from_utf16(&value.to_utf16()).unwrap())
    } else {
        let units = value.as_string().unwrap();
        if units.is_empty() {
            return "s_".into();
        }
        let mut encoded = String::from("s");
        for unit in units {
            write!(&mut encoded, "{unit:04x}").unwrap();
        }
        encoded
    }
}
fn queries() -> Vec<MarkerLine> {
    let mut queries: Vec<_> = ["", "0", "1", "2", "10", "a", "Infinity"]
        .into_iter()
        .map(MarkerLine::from)
        .collect();
    queries.extend([
        MarkerLine::string(vec![0xd800]),
        MarkerLine::number(f64::NAN),
        MarkerLine::number(-0.0),
    ]);
    queries.extend((-1..101).map(MarkerLine::from));
    queries.extend(
        [
            0.5,
            1.5,
            2.5,
            9_007_199_254_740_991.0,
            9_007_199_254_740_992.0,
            9_007_199_254_740_994.0,
        ]
        .map(MarkerLine::number),
    );
    assert_eq!(queries.len(), 118);
    queries
}
fn snapshot(
    service: &DecorationService,
    decoration: &Decoration,
    queries: &[MarkerLine],
) -> String {
    let marker = decoration.marker();
    let count = service.decorations().unwrap().count();
    let counts = queries
        .iter()
        .map(|query| {
            service
                .get_decorations_at_cell(0.0, query.clone(), None)
                .count()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}|{}|{}|{}|{}",
        encode(&marker.line_value()),
        u8::from(marker.is_disposed()),
        u8::from(decoration.is_disposed()),
        count,
        counts
    )
}
#[test]
fn all_405_raw_marker_snapshots_match_actual_source() {
    let fixture = include_str!("fixtures/decoration-raw.tsv");
    let rows: Vec<_> = fixture.lines().collect();
    assert_eq!(rows.len(), 405);
    let queries = queries();
    for (case, rows) in rows.chunks_exact(5).enumerate() {
        let fields: Vec<_> = rows[0].split('\t').collect();
        let line = decode(fields[0]);
        let height = fields[1].parse().unwrap();
        let options = Rc::new(OptionsService::new(
            false,
            std::iter::empty::<(&str, OptionValue)>(),
        ));
        let mut buffer = Buffer::new(true, 80, 24, options).unwrap();
        let service = DecorationService::new();
        service.attach_to_buffer_lines(&buffer.lines);
        let mut options = DecorationOptions::new(buffer.add_marker_value(line));
        options.height = Some(height);
        let decoration = service
            .register_decoration(Rc::new(RefCell::new(options)))
            .unwrap()
            .unwrap();
        for (step, row) in rows.iter().enumerate() {
            let fields: Vec<_> = row.split('\t').collect();
            assert_eq!(
                fields[2],
                ["add", "insert", "trim", "delete", "dispose"][step]
            );
            match step {
                0 => {}
                1 => {
                    buffer
                        .lines
                        .on_insert_emitter
                        .fire(&ListChange {
                            index: 1,
                            amount: 2,
                        })
                        .unwrap();
                }
                2 => {
                    buffer.lines.on_trim_emitter.fire(&1).unwrap();
                }
                3 => {
                    buffer
                        .lines
                        .on_delete_emitter
                        .fire(&ListChange {
                            index: 2,
                            amount: 1,
                        })
                        .unwrap();
                }
                4 => {
                    decoration.dispose().unwrap();
                }
                _ => unreachable!(),
            }
            service.line_cache().flush_pending().unwrap();
            assert_eq!(
                snapshot(&service, &decoration, &queries),
                fields[3],
                "case{case} step{step} {row}"
            );
        }
        service.dispose().unwrap();
    }
}
fn register(service: &DecorationService, line: impl Into<MarkerLine>, height: u32) -> Decoration {
    let mut options = DecorationOptions::new(Marker::new(line));
    options.height = Some(height);
    service
        .register_decoration(Rc::new(RefCell::new(options)))
        .unwrap()
        .unwrap()
}
#[test]
fn first_string_bucket_is_not_its_numeric_projection() {
    let service = DecorationService::new();
    let string = register(&service, "2", 1);
    let number = register(&service, 2, 1);
    for (query, expected) in [
        (MarkerLine::from("2"), &string),
        (MarkerLine::from(2), &number),
        (MarkerLine::from(3), &string),
    ] {
        let bucket = service.line_cache().decorations_on_line(query).unwrap();
        assert_eq!(bucket.len(), 1);
        assert!(bucket.get(0).unwrap().ptr_eq(expected));
    }
    assert_eq!(service.get_decorations_at_cell(0.0, "3", None).count(), 0);
}
#[test]
fn string_zero_height_still_indexes_its_concatenated_span() {
    let service = DecorationService::new();
    let decoration = register(&service, "2", 0);
    assert_eq!(service.get_decorations_at_cell(0.0, "2", None).count(), 1);
    assert_eq!(service.get_decorations_at_cell(0.0, 2, None).count(), 0);
    assert_eq!(service.get_decorations_at_cell(0.0, 19, None).count(), 1);
    assert_eq!(service.get_decorations_at_cell(0.0, 20, None).count(), 0);
    decoration.dispose().unwrap();
    assert_eq!(service.get_decorations_at_cell(0.0, "2", None).count(), 0);
    assert_eq!(service.get_decorations_at_cell(0.0, 19, None).count(), 0);
}
#[test]
fn trim_merges_invalid_string_keys_into_one_persistent_nan_bucket() {
    let options = Rc::new(OptionsService::new(
        false,
        std::iter::empty::<(&str, OptionValue)>(),
    ));
    let mut buffer = Buffer::new(true, 80, 24, options).unwrap();
    let service = DecorationService::new();
    service.attach_to_buffer_lines(&buffer.lines);
    let decorations: Vec<_> = ["a", "b"]
        .into_iter()
        .map(|line| {
            let options = DecorationOptions::new(buffer.add_marker_value(MarkerLine::from(line)));
            service
                .register_decoration(Rc::new(RefCell::new(options)))
                .unwrap()
                .unwrap()
        })
        .collect();
    for _ in 0..2 {
        buffer.lines.on_trim_emitter.fire(&1).unwrap();
        let bucket = service.line_cache().decorations_on_line(f64::NAN).unwrap();
        assert_eq!(bucket.len(), 2);
        assert!(bucket.get(0).unwrap().ptr_eq(&decorations[0]));
        assert!(bucket.get(1).unwrap().ptr_eq(&decorations[1]));
        assert_eq!(service.get_decorations_at_cell(0.0, "a", None).count(), 0);
        assert_eq!(service.get_decorations_at_cell(0.0, "b", None).count(), 0);
    }
}
#[test]
fn string_nonprogress_is_reported_before_indexing() {
    let service = DecorationService::new();
    let options = DecorationOptions::new(Marker::new("9007199254740992"));
    assert!(matches!(
        service.register_decoration(Rc::new(RefCell::new(options))),
        Err(DecorationError::NonAdvancingSpan(_))
    ));
    assert_eq!(service.decorations().unwrap().count(), 0);
}
#[test]
fn evolved_numeric_span_reports_nonprogress_on_disposal() {
    let service = DecorationService::new();
    let decoration = register(&service, 9_007_199_254_740_994.0, 3);
    decoration.marker().set_line(9_007_199_254_740_996.0);
    // Re-indexing after an insert updates indexedStartLine without rebuilding buckets.
    let lines = xterm_core::circular_list::CircularList::<()>::new(1).unwrap();
    service.attach_to_buffer_lines(&lines);
    lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 1,
            amount: 2,
        })
        .unwrap();
    service.line_cache().flush_pending().unwrap();
    let error = decoration.dispose().unwrap_err();
    assert!(matches!(error, xterm_core::event::EmitError::Callback(_)));
}
