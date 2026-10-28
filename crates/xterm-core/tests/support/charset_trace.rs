//! Lossless charset-specific state projection over the shared real-engine schema.
use crate::projection;
use std::{collections::BTreeMap, rc::Rc};
use xterm_core::{
    buffer::Buffer,
    charset::{builtin_charset, Charset},
    options::{OptionValue, OptionsService},
    terminal::{CoreTerminal, TerminalError, TerminalEvent},
};

const DESIGNATORS: &str = "04567=ABCEHKQRYZ";
pub const PROFILES: [&str; 3] = ["string", "units", "bytes"];

pub fn custom_map() -> Charset {
    Charset::new(BTreeMap::from([
        (u16::from(b'!'), vec![]),
        (u16::from(b'?'), vec![0x69, 0x6a]),
        (u16::from(b'#'), vec![0, 0x78]),
        (u16::from(b'@'), vec![0xd83d, 0xde00]),
        (u16::from(b'['), vec![0xd800]),
        (u16::from(b']'), vec![0xdc00]),
        (u16::from(b'~'), vec![0x7a]),
        (u16::from(b'q'), vec![0x2500]),
    ]))
}
pub fn canonical(map: &Option<Charset>, custom: &Charset) -> String {
    let Some(map) = map else { return "-".into() };
    if let Some(id) = DESIGNATORS
        .chars()
        .find(|&id| builtin_charset(id).is_some_and(|value| Charset::ptr_eq(map, &value)))
    {
        return id.to_string();
    }
    assert!(
        Charset::ptr_eq(map, custom),
        "unexpected cloned/custom map identity"
    );
    "custom".into()
}
pub fn slots(maps: &[Option<Charset>], custom: &Charset) -> String {
    maps.iter()
        .map(|map| canonical(map, custom))
        .collect::<Vec<_>>()
        .join(",")
}
fn saved(buffer: &Buffer, custom: &Charset) -> String {
    format!(
        "{}/{}/{}",
        buffer.saved_glevel,
        canonical(&buffer.saved_charset, custom),
        slots(&buffer.saved_charsets, custom)
    )
}
pub fn service(terminal: &CoreTerminal, custom: &Charset) -> String {
    let state = terminal.state();
    format!(
        "{}/{}/{}",
        state.charset.glevel,
        canonical(&state.charset.charset, custom),
        slots(state.charset.charsets(), custom)
    )
}
pub struct Trace {
    pub state: [String; 5],
    pub events: String,
}
pub fn capture(terminal: &mut CoreTerminal, custom: &Charset) -> Trace {
    let cells = projection::snapshot(terminal);
    let charset = service(terminal, custom);
    let state = terminal.state();
    let normal = saved(&state.buffers.normal().borrow(), custom);
    let alt = saved(&state.buffers.alt().borrow(), custom);
    drop(state);
    let mut a11y = Vec::new();
    let mut events = Vec::new();
    for event in terminal.drain_events() {
        match &event {
            TerminalEvent::AccessibilityChar(value) => {
                let value = projection::hex(value);
                a11y.push(value.clone());
                events.push(format!("A{value}"));
            }
            TerminalEvent::Binary(value) => events.push(format!("Y{}", projection::hex(value))),
            TerminalEvent::Resize { cols, rows } => events.push(format!("Z{cols}:{rows}")),
            _ => {
                if let Some(value) = projection::event(&event) {
                    events.push(value);
                }
            }
        }
    }
    Trace {
        state: [cells, charset, normal, alt, a11y.join(";")],
        events: events.join("/"),
    }
}
pub fn terminal(cols: f64, rows: f64) -> CoreTerminal {
    CoreTerminal::with_options(Rc::new(OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(cols)),
            ("rows", OptionValue::Number(rows)),
            ("scrollback", OptionValue::Number(10.0)),
            ("screenReaderMode", OptionValue::Bool(true)),
        ],
    )))
    .unwrap()
}
pub fn write(
    terminal: &mut CoreTerminal,
    input: &[u16],
    profile: &str,
) -> Result<(), TerminalError> {
    match profile {
        "string" => terminal.write_utf16(input)?,
        "units" => {
            for unit in input {
                terminal.write_utf16(&[*unit])?;
            }
        }
        "bytes" => {
            // Node Buffer.from has the same replacement policy if a future
            // transport fixture contains lone input surrogates.
            for byte in String::from_utf16_lossy(input).bytes() {
                terminal.write_bytes(&[byte])?;
            }
        }
        other => panic!("unknown fixture profile {other}"),
    }
    Ok(())
}
