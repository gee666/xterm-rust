// Copyright (c) 2014 The xterm.js authors. All rights reserved.
// Copyright (c) 2012-2013, Christopher Jeffrey (MIT License)
// Licensed under MIT. Default-key branch of common/input/Keyboard.ts.
use super::super::{
    equals, key_mapping, lowercase, uppercase, KeyboardResult, KeyboardResultType, DEL, ESC, NUL,
    RS, US,
};
use super::{to_uint16, KeyboardEventSource};

pub(super) fn default_key<E: KeyboardEventSource + ?Sized>(
    event: &E,
    is_mac: bool,
    mac_option_is_meta: bool,
    result: &mut KeyboardResult,
) -> Result<Option<Vec<u16>>, E::Error> {
    let key =
        if event.ctrl_key()? && !event.shift_key()? && !event.alt_key()? && !event.meta_key()? {
            control_key(event)?
        } else if (!is_mac || mac_option_is_meta) && event.alt_key()? && !event.meta_key()? {
            alt_key(event, result)?
        } else if is_mac
            && !event.alt_key()?
            && !event.ctrl_key()?
            && !event.shift_key()?
            && event.meta_key()?
        {
            if event.key_code()? == 65.0 {
                result.result_type = KeyboardResultType::SelectAll;
            }
            None
        } else if !event.key()?.is_empty()
            && !event.ctrl_key()?
            && !event.alt_key()?
            && !event.meta_key()?
            && event.key_code()? >= 48.0
            && event.key()?.len() == 1
        {
            Some(event.key()?.into_owned())
        } else if !event.key()?.is_empty() && event.ctrl_key()? && event.shift_key()? {
            let code = event.code()?;
            if equals(&code, "Minus") {
                Some(vec![US])
            } else if equals(&code, "Digit2") {
                Some(vec![NUL])
            } else if equals(&code, "Digit6") {
                Some(vec![RS])
            } else {
                None
            }
        } else {
            None
        };
    Ok(key)
}
fn control_key<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<Option<Vec<u16>>, E::Error> {
    let unit = if event.key_code()? >= 65.0 && event.key_code()? <= 90.0 {
        to_uint16(event.key_code()? - 64.0)
    } else if event.key_code()? == 32.0 {
        NUL
    } else if event.key_code()? >= 51.0 && event.key_code()? <= 55.0 {
        to_uint16(event.key_code()? - 51.0 + 27.0)
    } else if event.key_code()? == 56.0 {
        DEL
    } else if equals(&event.key()?, "/") {
        US
    } else if event.key_code()? == 219.0 {
        ESC
    } else if event.key_code()? == 220.0 {
        0x1c
    } else if event.key_code()? == 221.0 {
        0x1d
    } else {
        return Ok(None);
    };
    Ok(Some(vec![unit]))
}
fn alt_key<E: KeyboardEventSource + ?Sized>(
    event: &E,
    result: &mut KeyboardResult,
) -> Result<Option<Vec<u16>>, E::Error> {
    if let Some((plain, shifted)) = key_mapping(event.key_code()?) {
        return Ok(Some(vec![
            ESC,
            u16::from(if !event.shift_key()? { plain } else { shifted }),
        ]));
    }
    if event.key_code()? >= 65.0 && event.key_code()? <= 90.0 {
        let code = if event.ctrl_key()? {
            event.key_code()? - 64.0
        } else {
            event.key_code()? + 32.0
        };
        let unit = to_uint16(code);
        let mut key = vec![ESC];
        if event.shift_key()? {
            uppercase::append_uppercase(&mut key, unit);
        } else {
            key.push(unit);
        }
        Ok(Some(key))
    } else if event.key_code()? == 32.0 {
        Ok(Some(vec![ESC, if event.ctrl_key()? { NUL } else { 32 }]))
    } else if equals(&event.key()?, "Dead") && event.code()?.starts_with(&[75, 101, 121]) {
        let code = event.code()?;
        let mut key = vec![ESC];
        // JS slice(3,4) happens before the shift getter. Keep this one-unit
        // value even if that getter mutates the event's code property again.
        let suffix = code.get(3).copied();
        if !event.shift_key()? {
            if let Some(unit) = suffix {
                lowercase::append_lowercase(&mut key, unit);
            }
        } else if let Some(unit) = suffix {
            key.push(unit);
        }
        result.cancel = true;
        Ok(Some(key))
    } else {
        Ok(None)
    }
}
