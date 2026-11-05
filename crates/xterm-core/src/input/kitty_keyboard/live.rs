// Licensed under MIT. Fallible Kitty policy retaining original getter order.
use super::live_fields;
use super::lookup::{Hint, Table, Value};
use super::{flags, EventType};
use crate::input::keyboard::{KeyboardResult, KeyboardResultType};
use crate::input::keyboard_event_source::{code_point_at_zero, KeyboardEventSource};

pub(super) fn evaluate<E: KeyboardEventSource + ?Sized>(
    event: &E,
    flags: u32,
    event_type: EventType,
    mac_option_as_alt: bool,
) -> Result<KeyboardResult, E::Error> {
    let mut result = KeyboardResult {
        result_type: KeyboardResultType::SendKey,
        ..KeyboardResult::default()
    };
    let bits = u8::from(event.shift_key()?)
        | (u8::from(event.alt_key()?) << 1)
        | (u8::from(event.ctrl_key()?) << 2)
        | (u8::from(event.meta_key()?) << 3);
    let modifiers = if bits == 0 { 0 } else { bits + 1 };
    let is_mod = live_fields::modifier(event)?;
    let report_events = flags & flags::REPORT_EVENT_TYPES != 0;
    let report_all = flags & flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES != 0;
    if !report_events && event_type == EventType::Release {
        return Ok(result);
    }
    if is_mod && !report_all {
        return Ok(result);
    }
    if live_fields::lock(event)? && !report_all {
        return Ok(result);
    }

    for table in [Table::Letter, Table::Ss3, Table::Tilde] {
        let value = event.kitty_lookup(table, &event.key()?)?;
        let use_value = if table == Table::Tilde {
            !value.is_undefined()
        } else {
            value.is_truthy()
        };
        if use_value {
            result.key = Some(legacy_sequence(
                &value,
                table,
                modifiers,
                event_type,
                report_events,
            )?);
            result.cancel = true;
            return Ok(result);
        }
    }
    let code = live_fields::key_code(event, mac_option_as_alt)?;
    if code.is_undefined() {
        return Ok(result);
    }
    let special = code.is_number(13.0) || code.is_number(9.0) || code.is_number(127.0);
    if special && event_type == EventType::Release && !report_all {
        return Ok(result);
    }
    let is_func = !event
        .kitty_lookup(Table::Functional, &event.key()?)?
        .is_undefined()
        || live_fields::numpad(event)?.is_some();
    let use_csi_u = report_all
        || (report_events && event_type == EventType::Release)
        || ((flags & flags::DISAMBIGUATE_ESCAPE_CODES != 0 || report_events)
            && ((is_func && !special)
                || ((modifiers > 0 && event.key()?.len() != 1) || modifiers > 2)));
    if use_csi_u {
        result.key = Some(csi_u(
            event, &code, modifiers, event_type, flags, is_func, is_mod,
        )?);
        result.cancel = true;
    } else if special {
        result.key = Some(vec![if code.is_number(13.0) {
            13
        } else if code.is_number(9.0) {
            9
        } else {
            127
        }]);
    } else if event.key()?.len() == 1
        && !event.ctrl_key()?
        && !event.alt_key()?
        && !event.meta_key()?
    {
        result.key = Some(event.key()?.into_owned());
    }
    Ok(result)
}

// The raw operand must reach string addition with the default hint, not an
// eager String conversion. Keep UTF-16 output even when it contains surrogates.
fn legacy_sequence<E>(
    value: &Value<E>,
    table: Table,
    modifiers: u8,
    event_type: EventType,
    report_events: bool,
) -> Result<Vec<u16>, E> {
    let needs_event = report_events && event_type != EventType::Press;
    let has_parameters = modifiers > 0 || needs_event;
    let prefix = if table == Table::Ss3 && !has_parameters {
        "\x1bO"
    } else {
        "\x1b["
    };
    let mut sequence: Vec<u16> = prefix.encode_utf16().collect();
    if table == Table::Tilde {
        value.append(&mut sequence, Hint::Default)?;
    } else if has_parameters {
        sequence.push(u16::from(b'1'));
    }
    if has_parameters {
        sequence.extend(format!(";{}", modifiers.max(1)).encode_utf16());
        if needs_event {
            sequence.extend(format!(":{}", event_type as u8).encode_utf16());
        }
    }
    if table == Table::Tilde {
        sequence.push(u16::from(b'~'));
    } else {
        value.append(&mut sequence, Hint::Default)?;
    }
    Ok(sequence)
}

fn csi_u<E: KeyboardEventSource + ?Sized>(
    event: &E,
    code: &Value<E::Error>,
    modifiers: u8,
    event_type: EventType,
    flags: u32,
    is_func: bool,
    is_mod: bool,
) -> Result<Vec<u16>, E::Error> {
    let mut sequence = vec![0x1b, u16::from(b'[')];
    code.append(&mut sequence, Hint::Default)?;
    if flags & flags::REPORT_ALTERNATE_KEYS != 0
        && event.shift_key()?
        && event.key()?.len() == 1
        && !is_func
        && !is_mod
    {
        let shifted = code_point_at_zero(&event.key()?);
        sequence.push(u16::from(b':'));
        if let Some(code) = shifted {
            sequence.extend(code.to_string().encode_utf16());
        } else {
            sequence.extend("undefined".encode_utf16());
        }
    }
    let associated = flags & flags::REPORT_ASSOCIATED_TEXT != 0
        && event_type != EventType::Release
        && event.key()?.len() == 1
        && !is_func
        && !is_mod
        && !event.ctrl_key()?;
    let text = if associated {
        code_point_at_zero(&event.key()?)
    } else {
        None
    };
    let needs_event = flags & flags::REPORT_EVENT_TYPES != 0
        && event_type != EventType::Press
        && (event_type == EventType::Release || text.is_none());
    if modifiers > 0 || needs_event || text.is_some() {
        sequence.push(u16::from(b';'));
        if modifiers > 0 {
            sequence.extend(modifiers.to_string().encode_utf16());
        } else if needs_event {
            sequence.push(u16::from(b'1'));
        }
        if needs_event {
            sequence.extend(format!(":{}", event_type as u8).encode_utf16());
        }
    }
    if let Some(text) = text {
        sequence.extend(format!(";{text}").encode_utf16());
    }
    sequence.push(u16::from(b'u'));
    Ok(sequence)
}
