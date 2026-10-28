// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Direct producer/forwarding checks for source InputHandler onA11yChar strings.
use std::{cell::RefCell, collections::BTreeMap, error::Error, fmt, rc::Rc};
use xterm_core::{
    charset::Charset,
    event::EmitError,
    headless::Terminal,
    options::{OptionValue, OptionsService},
    terminal::TerminalEvent,
};

fn terminal(enabled: bool) -> Terminal {
    Terminal::with_options(Rc::new(OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(40.0)),
            ("rows", OptionValue::Number(3.0)),
            ("screenReaderMode", OptionValue::Bool(enabled)),
        ],
    )))
    .unwrap()
}
fn capture(terminal: &Terminal) -> Rc<RefCell<Vec<Vec<u16>>>> {
    let values = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&values);
    terminal.on_event().subscribe(move |event| {
        if let TerminalEvent::AccessibilityChar(text) = event {
            output.borrow_mut().push(text.clone());
        }
    });
    values
}
fn cursor_cell(terminal: &Terminal) -> (usize, Vec<u16>) {
    terminal
        .with_state(|state| {
            let active = state.active_buffer();
            let active = active.borrow();
            let line = active.lines.get(active.ybase + active.y).unwrap();
            let chars = line.borrow().get_cell(0).unwrap().get_chars();
            (active.x, chars)
        })
        .unwrap()
}

#[test]
fn whole_and_split_utf16_forward_exact_units_and_pair_event_boundaries() {
    for split in [false, true] {
        let terminal = terminal(true);
        let values = capture(&terminal);
        let input = [0xdc00, 0xd83d, 0xde00, 0xd800, u16::from(b'A')];
        if split {
            for unit in input {
                terminal.write_utf16(&[unit]).unwrap();
            }
        } else {
            terminal.write_utf16(&input).unwrap();
        }
        assert_eq!(
            *values.borrow(),
            [
                vec![0xdc00],
                vec![0xd83d, 0xde00],
                vec![0xd800],
                vec![u16::from(b'A')]
            ]
        );
    }
}

#[test]
fn pending_high_surrogate_survives_empty_chunk_before_nonlow() {
    let terminal = terminal(true);
    let values = capture(&terminal);
    terminal.write_utf16(&[0xd800]).unwrap();
    terminal.write_utf16(&[]).unwrap();
    assert!(values.borrow().is_empty());
    terminal.write_utf16(&[u16::from(b'A')]).unwrap();
    assert_eq!(*values.borrow(), [vec![0xd800], vec![u16::from(b'A')]]);
}

#[test]
fn custom_charset_first_unit_empty_and_nul_replacements_remain_lossless() {
    let terminal = terminal(true);
    let values = capture(&terminal);
    let map = Charset::new(BTreeMap::from([
        (u16::from(b'!'), vec![]),
        (u16::from(b'['), vec![0xd800]),
        (u16::from(b']'), vec![0xdc00]),
        (u16::from(b'@'), vec![0xd83d, 0xde00]),
        (u16::from(b'?'), vec![0x69, 0x6a]),
        (u16::from(b'#'), vec![0, 0x78]),
    ]));
    terminal
        .state_handle_for_addon()
        .borrow_mut()
        .charset
        .set_g_charset(0, Some(map))
        .unwrap();
    terminal
        .write_utf16(&"![]@?#".encode_utf16().collect::<Vec<_>>())
        .unwrap();
    assert_eq!(
        *values.borrow(),
        [
            vec![0x21],
            vec![0xd800],
            vec![0xdc00],
            vec![0xd83d],
            vec![0x69],
            vec![0]
        ]
    );
}

#[test]
fn disabled_screen_reader_emits_no_character_payloads() {
    let terminal = terminal(false);
    let values = capture(&terminal);
    terminal
        .write_utf16(&[0xdc00, 0xd800, u16::from(b'A')])
        .unwrap();
    assert!(values.borrow().is_empty());
}

#[test]
fn source_bom_and_soft_hyphen_are_not_announced() {
    let terminal = terminal(true);
    let values = capture(&terminal);
    terminal
        .write_utf16(&[0xfeff, 0xad, 0xdc00, u16::from(b'A')])
        .unwrap();
    assert_eq!(*values.borrow(), [vec![0xdc00], vec![u16::from(b'A')]]);
}

#[test]
fn malformed_utf8_drops_surrogate_encoding_without_replacement_character() {
    let terminal = terminal(true);
    let values = capture(&terminal);
    terminal
        .write_bytes(&[
            0xed, 0xa0, 0x80, 0xf0, 0x9f, 0x98, 0x80, 0xef, 0xbb, 0xbf, b'B',
        ])
        .unwrap();
    assert_eq!(
        *values.borrow(),
        [vec![0xd83d, 0xde00], vec![u16::from(b'B')]]
    );
}

#[derive(Debug)]
struct CallbackFailure;
impl fmt::Display for CallbackFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("retained a11y failure")
    }
}
impl Error for CallbackFailure {}
fn retains_callback_cause(error: &(dyn Error + 'static), expected: &Rc<dyn Error>) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(EmitError::Callback(cause)) = error.downcast_ref::<EmitError>() {
            if Rc::ptr_eq(cause, expected) {
                return true;
            }
        }
        current = error.source();
    }
    false
}

#[test]
fn callback_failure_keeps_identity_and_aborts_before_cell_mutation() {
    let terminal = terminal(true);
    let values = capture(&terminal);
    let failure: Rc<dyn Error> = Rc::new(CallbackFailure);
    let thrown = Rc::clone(&failure);
    let token = terminal.on_event().try_subscribe(move |event| {
        if matches!(event, TerminalEvent::AccessibilityChar(_)) {
            Err(EmitError::Callback(Rc::clone(&thrown)))
        } else {
            Ok(())
        }
    });
    let error = terminal
        .write_utf16(&[0xdc00, u16::from(b'B')])
        .unwrap_err();
    assert!(retains_callback_cause(&error, &failure));
    assert_eq!(*values.borrow(), [vec![0xdc00]]);
    assert_eq!(cursor_cell(&terminal), (0, vec![]));
    token.dispose();
    terminal.write_utf16(&[u16::from(b'C')]).unwrap();
    assert_eq!(cursor_cell(&terminal), (1, vec![u16::from(b'C')]));
    assert_eq!(*values.borrow(), [vec![0xdc00], vec![u16::from(b'C')]]);
}
