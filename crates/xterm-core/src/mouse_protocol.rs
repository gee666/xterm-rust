// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from common/services/MouseStateService.ts.

use std::fmt;
use std::ops::BitOr;
use std::rc::Rc;

/// Button numbers are protocol values, including the eight auxiliary buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
    None = 3,
    Wheel = 4,
    Aux1 = 8,
    Aux2 = 9,
    Aux3 = 10,
    Aux4 = 11,
    Aux5 = 12,
    Aux6 = 13,
    Aux7 = 14,
    Aux8 = 15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseAction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Move = 32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidMouseCode(pub u8);

impl fmt::Display for InvalidMouseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid mouse protocol code {}", self.0)
    }
}
impl std::error::Error for InvalidMouseCode {}

impl TryFrom<u8> for MouseButton {
    type Error = InvalidMouseCode;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Middle),
            2 => Ok(Self::Right),
            3 => Ok(Self::None),
            4 => Ok(Self::Wheel),
            8 => Ok(Self::Aux1),
            9 => Ok(Self::Aux2),
            10 => Ok(Self::Aux3),
            11 => Ok(Self::Aux4),
            12 => Ok(Self::Aux5),
            13 => Ok(Self::Aux6),
            14 => Ok(Self::Aux7),
            15 => Ok(Self::Aux8),
            _ => Err(InvalidMouseCode(value)),
        }
    }
}
impl TryFrom<u8> for MouseAction {
    type Error = InvalidMouseCode;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Up),
            1 => Ok(Self::Down),
            2 => Ok(Self::Left),
            3 => Ok(Self::Right),
            32 => Ok(Self::Move),
            _ => Err(InvalidMouseCode(value)),
        }
    }
}

/// Floating point coordinates retain JavaScript number behavior. Optional
/// modifiers preserve `undefined` for custom protocols rather than forcing false.
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
    pub col: f64,
    pub row: f64,
    pub x: f64,
    pub y: f64,
    pub button: MouseButton,
    pub action: MouseAction,
    pub ctrl: Option<bool>,
    pub alt: Option<bool>,
    pub shift: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseEventTypes(pub u32);
impl MouseEventTypes {
    pub const NONE: Self = Self(0);
    pub const DOWN: Self = Self(1);
    pub const UP: Self = Self(2);
    pub const DRAG: Self = Self(4);
    pub const MOVE: Self = Self(8);
    pub const WHEEL: Self = Self(16);
}
impl BitOr for MouseEventTypes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub type MouseRestriction = Rc<dyn Fn(&mut MouseEvent) -> bool>;
/// Encodings return UTF-16 because JavaScript strings can contain lone surrogates.
pub type MouseEncoding = Rc<dyn Fn(&MouseEvent) -> Vec<u16>>;

#[derive(Clone)]
pub struct MouseProtocol {
    pub events: MouseEventTypes,
    pub restrict: MouseRestriction,
}

pub(super) fn default_protocols() -> Vec<(String, MouseProtocol)> {
    use MouseEventTypes as Events;
    let all = Events::DOWN | Events::UP | Events::WHEEL | Events::DRAG | Events::MOVE;
    let specifications: [(&str, MouseEventTypes, MouseRestriction); 5] = [
        ("NONE", Events::NONE, Rc::new(|_| false)),
        (
            "X10",
            Events::DOWN,
            Rc::new(|event| {
                if event.button == MouseButton::Wheel || event.action != MouseAction::Down {
                    return false;
                }
                event.ctrl = Some(false);
                event.alt = Some(false);
                event.shift = Some(false);
                true
            }),
        ),
        (
            "VT200",
            Events::DOWN | Events::UP | Events::WHEEL,
            Rc::new(|event| event.action != MouseAction::Move),
        ),
        (
            "DRAG",
            all_without_move(),
            Rc::new(|event| event.action != MouseAction::Move || event.button != MouseButton::None),
        ),
        ("ANY", all, Rc::new(|_| true)),
    ];
    specifications
        .into_iter()
        .map(|(name, events, restrict)| (name.into(), MouseProtocol { events, restrict }))
        .collect()
}

fn all_without_move() -> MouseEventTypes {
    MouseEventTypes::DOWN | MouseEventTypes::UP | MouseEventTypes::WHEEL | MouseEventTypes::DRAG
}

const SHIFT: u8 = 4;
const ALT: u8 = 8;
const CTRL: u8 = 16;
const WHEEL_CODE: u8 = 64;
const AUXILIARY_HIGH_CODE: u8 = 128;
const ENCODING_BIAS: f64 = 32.0;
const MAX_ENCODED_BYTE: f64 = 255.0;

fn event_code(event: &MouseEvent, sgr: bool) -> u8 {
    let mut code = if event.ctrl == Some(true) { CTRL } else { 0 }
        | if event.shift == Some(true) { SHIFT } else { 0 }
        | if event.alt == Some(true) { ALT } else { 0 };
    if event.button == MouseButton::Wheel {
        code |= WHEEL_CODE | event.action as u8;
    } else {
        let button = event.button as u8;
        code |= button & MouseButton::None as u8;
        if button & MouseButton::Wheel as u8 != 0 {
            code |= WHEEL_CODE;
        }
        if button & MouseButton::Aux1 as u8 != 0 {
            code |= AUXILIARY_HIGH_CODE;
        }
        if event.action == MouseAction::Move {
            code |= MouseAction::Move as u8;
        } else if event.action == MouseAction::Up && !sgr {
            code |= MouseButton::None as u8;
        }
    }
    code
}

fn to_uint16(number: f64) -> u16 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    // The remainder is integral and within u16 bounds after truncation.
    number.trunc().rem_euclid(65536.0) as u16
}

fn encode_default(event: &MouseEvent) -> Vec<u16> {
    let params = [
        f64::from(event_code(event, false)) + ENCODING_BIAS,
        event.col + ENCODING_BIAS,
        event.row + ENCODING_BIAS,
    ];
    if params.iter().any(|value| *value > MAX_ENCODED_BYTE) {
        return Vec::new();
    }
    let mut result = vec![0x1b, u16::from(b'['), u16::from(b'M')];
    result.extend(params.into_iter().map(to_uint16));
    result
}

fn encode_sgr(event: &MouseEvent, pixels: bool) -> Vec<u16> {
    let final_character = if event.action == MouseAction::Up && event.button != MouseButton::Wheel {
        'm'
    } else {
        'M'
    };
    let (x, y) = if pixels {
        (event.x, event.y)
    } else {
        (event.col, event.row)
    };
    let mut x_buffer = ryu_js::Buffer::new();
    let mut y_buffer = ryu_js::Buffer::new();
    format!(
        "\x1b[<{};{};{}{final_character}",
        event_code(event, true),
        x_buffer.format(x),
        y_buffer.format(y)
    )
    .encode_utf16()
    .collect()
}

pub(super) fn default_encodings() -> Vec<(String, MouseEncoding)> {
    vec![
        ("DEFAULT".into(), Rc::new(encode_default)),
        ("SGR".into(), Rc::new(|event| encode_sgr(event, false))),
        (
            "SGR_PIXELS".into(),
            Rc::new(|event| encode_sgr(event, true)),
        ),
    ]
}
