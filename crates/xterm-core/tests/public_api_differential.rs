// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::public_api::validation::{
    verify_integers, verify_mutable_option, verify_positive_integers, verify_proposed_api, ApiError,
};
use xterm_core::{
    core_service::{BufferPosition, CoreService},
    headless::ModesSnapshot,
    log_service::{LogError, LogRecord, LogService, LogSink},
    mouse_state::{MouseEventTypes, MouseProtocol, MouseStateService},
    options::OptionsService,
};

fn observed(result: Result<(), ApiError>) -> String {
    match result {
        Ok(()) => "ok".into(),
        Err(error) => error.to_string(),
    }
}

#[test]
fn pa01_pa02_source_generated_number_and_error_contracts() {
    let mut rows = 0;
    for line in include_str!("fixtures/public-api-numeric.tsv").lines() {
        let mut fields = line.splitn(3, '\t');
        let kind = fields.next().expect("fixture kind");
        let value = fields.next().expect("fixture value");
        let expected = fields.next().expect("fixture outcome");
        let result = match kind {
            "integer" | "positive" => {
                let number = f64::from_bits(u64::from_str_radix(value, 16).expect("f64 bits"));
                if kind == "integer" {
                    verify_integers(&[number])
                } else {
                    verify_positive_integers(&[number])
                }
            }
            "option" => verify_mutable_option(value),
            "proposed" => verify_proposed_api(value.parse().expect("boolean fixture")),
            other => panic!("unexpected fixture operation {other}"),
        };
        assert_eq!(observed(result), expected, "row {rows}: {kind} {value}");
        rows += 1;
    }
    assert_eq!(rows, 8238, "regenerate with scripts/public-api-oracle.mjs");
}

#[derive(Default)]
struct Records(RefCell<Vec<LogRecord>>);
impl LogSink for Records {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.0.borrow_mut().push(record);
        Ok(())
    }
}

#[test]
fn pa05_source_generated_complete_modes_snapshots() {
    let options = Rc::new(OptionsService::default());
    let logger =
        Rc::new(LogService::new(Rc::clone(&options), Rc::new(Records::default())).unwrap());
    let core = CoreService::new(options, logger, BufferPosition::default).unwrap();
    let mouse = MouseStateService::<()>::new();
    mouse.add_protocol(
        "CUSTOM",
        MouseProtocol {
            events: MouseEventTypes::NONE,
            restrict: Rc::new(|_| false),
        },
    );
    let mut rows = 0;
    for line in include_str!("fixtures/public-api-modes.tsv").lines() {
        let mut fields = line.splitn(3, '\t');
        let mask: u32 = fields.next().unwrap().parse().unwrap();
        let protocol = fields.next().unwrap();
        let expected = fields.next().unwrap();
        {
            let mut modes = core.dec_private_modes.borrow_mut();
            modes.application_cursor_keys = mask & 1 != 0;
            modes.application_keypad = mask & 2 != 0;
            modes.bracketed_paste_mode = mask & 4 != 0;
            modes.origin = mask & 8 != 0;
            modes.reverse_wraparound = mask & 16 != 0;
            modes.send_focus = mask & 32 != 0;
            modes.synchronized_output = mask & 64 != 0;
            modes.win32_input_mode = mask & 128 != 0;
            modes.wraparound = mask & 256 != 0;
        }
        core.modes.borrow_mut().insert_mode = mask & 512 != 0;
        core.is_cursor_hidden.set(mask & 1024 != 0);
        mouse.set_active_protocol(protocol).unwrap();
        let snapshot = ModesSnapshot::from_services(&core, &mouse).unwrap();
        let booleans = [
            snapshot.application_cursor_keys_mode,
            snapshot.application_keypad_mode,
            snapshot.bracketed_paste_mode,
            snapshot.insert_mode,
            snapshot.origin_mode,
            snapshot.reverse_wraparound_mode,
            snapshot.send_focus_mode,
            snapshot.show_cursor,
            snapshot.synchronized_output_mode,
            snapshot.win32_input_mode,
            snapshot.wraparound_mode,
        ];
        let mut values: Vec<String> = booleans
            .iter()
            .map(|value| u8::from(*value).to_string())
            .collect();
        values.insert(4, snapshot.mouse_tracking_mode.as_str().into());
        assert_eq!(
            values.join(","),
            expected,
            "mask {mask}, protocol {protocol}"
        );
        // A previously returned snapshot cannot observe later service mutation.
        core.is_cursor_hidden.set(!core.is_cursor_hidden.get());
        assert_eq!(snapshot.show_cursor, mask & 1024 == 0);
        rows += 1;
    }
    assert_eq!(rows, 12_288);
}
