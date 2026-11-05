// Licensed under MIT. Source-ordered Kitty field and key-number resolution.
use super::{
    keys,
    lookup::{Table, Value},
};
use crate::input::keyboard_event_source::{code_point_at_zero, KeyboardEventSource};

pub(super) fn numpad<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<Option<f64>, E::Error> {
    if keys::starts_with(&event.code()?, "Numpad") {
        let code = event.code()?;
        return Ok(keys::numpad_suffix(code.get(6..).unwrap_or_default()));
    }
    Ok(None)
}

pub(super) fn key_code<E: KeyboardEventSource + ?Sized>(
    event: &E,
    mac_option_as_alt: bool,
) -> Result<Value<E::Error>, E::Error> {
    if let Some(code) = numpad(event)? {
        return Ok(Value::Number(code));
    }
    if let Some(code) = keys::modifier(&event.code()?) {
        return Ok(Value::Number(f64::from(code)));
    }
    let code = event.kitty_lookup(Table::Functional, &event.key()?)?;
    if !code.is_undefined() {
        return Ok(code);
    }
    if (event.shift_key()? || (mac_option_as_alt && event.alt_key()?)) && !event.code()?.is_empty()
    {
        if keys::starts_with(&event.code()?, "Digit") && event.code()?.len() == 6 {
            if let Some(digit) = event
                .code()?
                .get(5)
                .filter(|unit| (48..=57).contains(*unit))
            {
                return Ok(Value::Number(f64::from(*digit)));
            }
        }
        if keys::starts_with(&event.code()?, "Key") && event.code()?.len() == 4 {
            let code = event.code()?;
            let number = code
                .get(3)
                .map(|unit| f64::from(crate::input::keyboard::lowercase::lowercase_first(*unit)))
                .unwrap_or(f64::NAN);
            return Ok(Value::Number(number));
        }
    }
    if event.key()?.len() == 1 {
        let Some(code) = code_point_at_zero(&event.key()?) else {
            return Ok(Value::Undefined);
        };
        return Ok(Value::Number(f64::from(if (65..=90).contains(&code) {
            code + 32
        } else {
            code
        })));
    }
    Ok(Value::Undefined)
}

pub(super) fn modifier<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<bool, E::Error> {
    Ok(keys::equals(&event.key()?, "Shift")
        || keys::equals(&event.key()?, "Control")
        || keys::equals(&event.key()?, "Alt")
        || keys::equals(&event.key()?, "Meta"))
}

pub(super) fn lock<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<bool, E::Error> {
    Ok(keys::equals(&event.key()?, "CapsLock")
        || keys::equals(&event.key()?, "NumLock")
        || keys::equals(&event.key()?, "ScrollLock"))
}
