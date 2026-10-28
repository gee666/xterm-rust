// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Shared mouse policy over typed fields or live host properties. Each read is
//! deliberately separate: JavaScript accessors can mutate fields or throw.

#[derive(Clone, Copy)]
pub enum Field {
    Ctrl,
    Shift,
    Alt,
    Button,
    Action,
    Col,
    Row,
    X,
    Y,
}

impl Field {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ctrl => "ctrl",
            Self::Shift => "shift",
            Self::Alt => "alt",
            Self::Button => "button",
            Self::Action => "action",
            Self::Col => "col",
            Self::Row => "row",
            Self::X => "x",
            Self::Y => "y",
        }
    }
}

#[derive(Clone, Copy)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
}

impl Modifier {
    pub fn field(self) -> Field {
        match self {
            Self::Ctrl => Field::Ctrl,
            Self::Alt => Field::Alt,
            Self::Shift => Field::Shift,
        }
    }
}

/// Host operations preserve strict comparison, delayed coercion and failures.
/// Implementations must not pre-read fields or hold guards across these calls.
pub trait MouseFields {
    type Value;
    type Error;
    fn get(&self, field: Field) -> Result<Self::Value, Self::Error>;
    fn set_false(&mut self, modifier: Modifier) -> Result<(), Self::Error>;
    fn truth(value: &Self::Value) -> bool;
    fn equals_number(value: &Self::Value, number: i32) -> bool;
    fn bit_and(value: &Self::Value, mask: i32) -> Result<i32, Self::Error>;
    fn bit_or(code: i32, value: &Self::Value) -> Result<i32, Self::Error>;
    fn add_number(value: &Self::Value, number: i32) -> Result<Self::Value, Self::Error>;
    fn greater(value: &Self::Value, number: i32) -> Result<bool, Self::Error>;
    fn char_code(value: &Self::Value) -> Result<u16, Self::Error>;
    fn text(value: &Self::Value) -> Result<Vec<u16>, Self::Error>;
}

const BUTTON_WHEEL: i32 = 4;
const BUTTON_NONE: i32 = 3;
const ACTION_UP: i32 = 0;
const ACTION_DOWN: i32 = 1;
const ACTION_MOVE: i32 = 32;
const ENCODING_BIAS: i32 = 32;
const MAX_ENCODED_BYTE: i32 = 255;
const UTF16_MODULUS: f64 = 65536.0;

#[derive(Clone, Copy)]
pub enum Protocol {
    None,
    X10,
    Vt200,
    Drag,
    Any,
}

impl Protocol {
    pub fn restrict<E: MouseFields>(self, event: &mut E) -> Result<bool, E::Error> {
        Ok(match self {
            Self::None => false,
            Self::X10 => {
                if E::equals_number(&event.get(Field::Button)?, BUTTON_WHEEL)
                    || !E::equals_number(&event.get(Field::Action)?, ACTION_DOWN)
                {
                    return Ok(false);
                }
                event.set_false(Modifier::Ctrl)?;
                event.set_false(Modifier::Alt)?;
                event.set_false(Modifier::Shift)?;
                true
            }
            Self::Vt200 => !E::equals_number(&event.get(Field::Action)?, ACTION_MOVE),
            Self::Drag => {
                !(E::equals_number(&event.get(Field::Action)?, ACTION_MOVE)
                    && E::equals_number(&event.get(Field::Button)?, BUTTON_NONE))
            }
            Self::Any => true,
        })
    }
}

#[derive(Clone, Copy)]
pub enum Encoding {
    Default,
    Sgr,
    SgrPixels,
}

fn event_code<E: MouseFields>(event: &E, sgr: bool) -> Result<i32, E::Error> {
    const SHIFT: i32 = 4;
    const ALT: i32 = 8;
    const CTRL: i32 = 16;
    const WHEEL: i32 = 64;
    const EXTENDED: i32 = 128;
    let mut code = if E::truth(&event.get(Field::Ctrl)?) {
        CTRL
    } else {
        0
    } | if E::truth(&event.get(Field::Shift)?) {
        SHIFT
    } else {
        0
    } | if E::truth(&event.get(Field::Alt)?) {
        ALT
    } else {
        0
    };
    if E::equals_number(&event.get(Field::Button)?, BUTTON_WHEEL) {
        code |= WHEEL;
        code = E::bit_or(code, &event.get(Field::Action)?)?;
    } else {
        code |= E::bit_and(&event.get(Field::Button)?, BUTTON_NONE)?;
        if E::bit_and(&event.get(Field::Button)?, 4)? != 0 {
            code |= WHEEL;
        }
        if E::bit_and(&event.get(Field::Button)?, 8)? != 0 {
            code |= EXTENDED;
        }
        if E::equals_number(&event.get(Field::Action)?, ACTION_MOVE) {
            code |= ACTION_MOVE;
        } else if E::equals_number(&event.get(Field::Action)?, ACTION_UP) && !sgr {
            code |= BUTTON_NONE;
        }
    }
    Ok(code)
}

impl Encoding {
    pub fn encode<E: MouseFields>(self, event: &E) -> Result<Vec<u16>, E::Error> {
        if matches!(self, Self::Default) {
            let code = f64::from(event_code(event, false)?) + f64::from(ENCODING_BIAS);
            let col = E::add_number(&event.get(Field::Col)?, ENCODING_BIAS)?;
            let row = E::add_number(&event.get(Field::Row)?, ENCODING_BIAS)?;
            if code > f64::from(MAX_ENCODED_BYTE)
                || E::greater(&col, MAX_ENCODED_BYTE)?
                || E::greater(&row, MAX_ENCODED_BYTE)?
            {
                return Ok(Vec::new());
            }
            let mut output = vec![
                0x1b,
                u16::from(b'['),
                u16::from(b'M'),
                code.rem_euclid(UTF16_MODULUS) as u16,
            ];
            output.push(E::char_code(&col)?);
            output.push(E::char_code(&row)?);
            return Ok(output);
        }
        // The final-character expression precedes eventCode and coordinate reads.
        let release = E::equals_number(&event.get(Field::Action)?, ACTION_UP)
            && !E::equals_number(&event.get(Field::Button)?, BUTTON_WHEEL);
        let code = event_code(event, true)?;
        let mut output = format!("\x1b[<{code};").encode_utf16().collect::<Vec<_>>();
        let pixels = matches!(self, Self::SgrPixels);
        output.extend(E::text(&event.get(if pixels {
            Field::X
        } else {
            Field::Col
        })?)?);
        output.push(u16::from(b';'));
        output.extend(E::text(&event.get(if pixels {
            Field::Y
        } else {
            Field::Row
        })?)?);
        output.push(u16::from(if release { b'm' } else { b'M' }));
        Ok(output)
    }
}
