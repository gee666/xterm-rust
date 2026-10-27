//! Source-generated fixtures include every parser invocation made by the 38
//! unchanged ProgramStatus unit cases, boundary inputs, and seeded fuzz cases.
#[path = "support/status.rs"]
mod support;
use serde_json::{json, Map, Value};
use std::rc::Rc;
use support::FixtureTerminal;
use xterm_addons::program_status::{
    parse_report_utf16, StatusKind, StatusRecord, StatusReport, StatusState,
};
use xterm_addons::progress::ProgressAddon;

fn record_json(record: &StatusRecord) -> Value {
    let mut result = Map::new();
    result.insert("id".into(), json!(record.id));
    result.insert(
        "state".into(),
        json!(match record.state {
            StatusState::Idle => "idle",
            StatusState::Working => "working",
            StatusState::Done => "done",
            StatusState::Blocked => "blocked",
            StatusState::Error => "error",
        }),
    );
    if let Some(kind) = record.kind {
        result.insert(
            "kind".into(),
            json!(match kind {
                StatusKind::Permission => "permission",
                StatusKind::Question => "question",
                StatusKind::Auth => "auth",
            }),
        );
    }
    if let Some(progress) = record.progress {
        result.insert("progress".into(), json!(progress));
    }
    for (key, value) in [
        ("app", &record.app),
        ("title", &record.title),
        ("message", &record.message),
    ] {
        if let Some(value) = value {
            result.insert(key.into(), json!(value));
        }
    }
    Value::Object(result)
}

fn input(case: &Value) -> Vec<u16> {
    case["input"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| u16::try_from(v.as_u64().unwrap()).unwrap())
        .collect()
}

#[test]
fn status_parser_matches_source_baseline_and_seed_0x750109() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/status-addons.json")).unwrap();
    for (index, case) in fixture["parse"].as_array().unwrap().iter().enumerate() {
        let actual = match parse_report_utf16(&input(case)) {
            Some(StatusReport::Clear(id)) => json!({"clear":true,"id":id}),
            Some(StatusReport::Record(record)) => {
                json!({"clear":false,"record":record_json(&record)})
            }
            None => Value::Null,
        };
        assert_eq!(
            actual, case["expected"],
            "case {index}, input {:?}",
            case["input"]
        );
    }
}

#[test]
fn progress_matches_source_seed_0x750109() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/status-addons.json")).unwrap();
    let terminal = Rc::new(FixtureTerminal::default());
    let addon = ProgressAddon::new();
    addon.activate(terminal.clone());
    let events = Rc::new(std::cell::RefCell::new(Vec::new()));
    let captured = events.clone();
    addon.on_change().unwrap().subscribe(move |value| {
        captured
            .borrow_mut()
            .push(json!({"state":value.state,"value":value.value}))
    });
    for (index, case) in fixture["progress"].as_array().unwrap().iter().enumerate() {
        events.borrow_mut().clear();
        assert_eq!(
            terminal.osc_units(9, &input(case)).unwrap(),
            case["handled"].as_bool().unwrap(),
            "case {index}"
        );
        let actual = addon.progress();
        assert_eq!(
            actual.state,
            case["expected"]["state"].as_f64().unwrap(),
            "case {index}"
        );
        assert_eq!(
            actual.value,
            case["expected"]["value"].as_f64().unwrap(),
            "case {index}"
        );
        let actual_events = events.borrow();
        let expected_events = case["events"].as_array().unwrap();
        assert_eq!(actual_events.len(), expected_events.len(), "case {index}");
        for (actual, expected) in actual_events.iter().zip(expected_events) {
            for key in ["state", "value"] {
                assert_eq!(
                    actual[key].as_f64(),
                    expected[key].as_f64(),
                    "case {index}, {key}"
                );
            }
        }
    }
}

#[test]
fn fixture_ris_remains_unhandled_and_string_dispatch_is_available() {
    let terminal = FixtureTerminal::default();
    assert!(!terminal.ris());
    assert!(!terminal.osc(9, "4;1;20"));
}
