// Licensed under MIT. Fallible Kitty policy retaining original getter order.
use super::live_fields::{self, CodeValue};
use super::{flags, inherited::inherited_name, keys, legacy_sequence, EventType};
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

    if let Some(suffix) = keys::letter(&event.key()?) {
        result.key = Some(
            legacy_sequence("1", suffix, false, modifiers, event_type, report_events)
                .encode_utf16()
                .collect(),
        );
        result.cancel = true;
        return Ok(result);
    }
    if let Some(suffix) = keys::ss3(&event.key()?) {
        result.key = Some(
            legacy_sequence("1", suffix, true, modifiers, event_type, report_events)
                .encode_utf16()
                .collect(),
        );
        result.cancel = true;
        return Ok(result);
    }
    let tilde_key = event.key()?;
    let tilde = keys::tilde(&tilde_key)
        .map(|number| CodeValue::Number(f64::from(number)))
        .or_else(|| inherited_name(&tilde_key).map(CodeValue::Native));
    if let Some(number) = tilde {
        result.key = Some(
            legacy_sequence(
                &number.text(),
                "~",
                false,
                modifiers,
                event_type,
                report_events,
            )
            .encode_utf16()
            .collect(),
        );
        result.cancel = true;
        return Ok(result);
    }
    let Some(code) = live_fields::key_code(event, mac_option_as_alt)? else {
        return Ok(result);
    };
    let special = code.is_special();
    if special && event_type == EventType::Release && !report_all {
        return Ok(result);
    }
    let is_func =
        live_fields::functional(&event.key()?).is_some() || live_fields::numpad(event)?.is_some();
    let use_csi_u = report_all
        || (report_events && event_type == EventType::Release)
        || ((flags & flags::DISAMBIGUATE_ESCAPE_CODES != 0 || report_events)
            && ((is_func && !special)
                || ((modifiers > 0 && event.key()?.len() != 1) || modifiers > 2)));
    if use_csi_u {
        result.key = Some(
            csi_u(event, &code, modifiers, event_type, flags, is_func, is_mod)?
                .encode_utf16()
                .collect(),
        );
        result.cancel = true;
    } else if special {
        if let CodeValue::Number(code) = code {
            result.key = Some(vec![code as u16]);
        }
    } else if event.key()?.len() == 1
        && !event.ctrl_key()?
        && !event.alt_key()?
        && !event.meta_key()?
    {
        result.key = Some(event.key()?.into_owned());
    }
    Ok(result)
}

fn csi_u<E: KeyboardEventSource + ?Sized>(
    event: &E,
    code: &CodeValue,
    modifiers: u8,
    event_type: EventType,
    flags: u32,
    is_func: bool,
    is_mod: bool,
) -> Result<String, E::Error> {
    let mut sequence = format!("\x1b[{}", code.text());
    if flags & flags::REPORT_ALTERNATE_KEYS != 0
        && event.shift_key()?
        && event.key()?.len() == 1
        && !is_func
        && !is_mod
    {
        let shifted = code_point_at_zero(&event.key()?);
        sequence.push(':');
        if let Some(code) = shifted {
            sequence.push_str(&code.to_string());
        } else {
            sequence.push_str("undefined");
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
        sequence.push(';');
        if modifiers > 0 {
            sequence.push_str(&modifiers.to_string());
        } else if needs_event {
            sequence.push('1');
        }
        if needs_event {
            sequence.push_str(&format!(":{}", event_type as u8));
        }
    }
    if let Some(text) = text {
        sequence.push_str(&format!(";{text}"));
    }
    sequence.push('u');
    Ok(sequence)
}
