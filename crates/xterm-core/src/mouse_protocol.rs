// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from common/services/MouseStateService.ts.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::ops::BitOr;
use std::rc::Rc;

#[path = "mouse_fields_typed.rs"]
mod typed_fields;
use crate::mouse_fields::{Encoding, Protocol};

/// Button numbers are protocol values, including the eight auxiliary buttons.
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
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
    /// DOM button values such as -1 must retain JS bitwise encoding behavior.
    Raw(i32),
    /// Live DOM getters can produce fractional/nonfinite JS numbers. Preserve
    /// these for strict comparisons and move debounce until bitwise encoding.
    RawNumber(f64),
}

impl MouseButton {
    /// JavaScript ToInt32 projection for protocol bit operations only.
    pub fn raw_value(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Middle => 1,
            Self::Right => 2,
            Self::None => 3,
            Self::Wheel => 4,
            Self::Aux1 => 8,
            Self::Aux2 => 9,
            Self::Aux3 => 10,
            Self::Aux4 => 11,
            Self::Aux5 => 12,
            Self::Aux6 => 13,
            Self::Aux7 => 14,
            Self::Aux8 => 15,
            Self::Raw(value) => value,
            Self::RawNumber(value) => to_int32(value),
        }
    }

    pub fn numeric_value(self) -> f64 {
        match self {
            Self::RawNumber(value) => value,
            _ => f64::from(self.raw_value()),
        }
    }
}

fn to_int32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let unsigned = value.trunc().rem_euclid(4_294_967_296.0);
    if unsigned >= 2_147_483_648.0 {
        (unsigned - 4_294_967_296.0) as i32
    } else {
        unsigned as i32
    }
}

impl PartialEq for MouseButton {
    fn eq(&self, other: &Self) -> bool {
        self.numeric_value() == other.numeric_value()
    }
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
pub type MouseEncoding = Rc<dyn Fn(&mut MouseEvent) -> Vec<u16>>;

#[derive(Clone)]
pub struct MouseProtocol {
    pub events: MouseEventTypes,
    pub restrict: MouseRestriction,
}

/// Shared mutable object identity for a retained JavaScript protocol object.
/// Setters release borrows before dropping replaced callback captures.
pub struct SharedMouseProtocol {
    events: Cell<MouseEventTypes>,
    restrict: RefCell<MouseRestriction>,
}
impl SharedMouseProtocol {
    pub fn new(protocol: MouseProtocol) -> Self {
        Self {
            events: Cell::new(protocol.events),
            restrict: RefCell::new(protocol.restrict),
        }
    }
    pub fn events(&self) -> MouseEventTypes {
        self.events.get()
    }
    pub fn set_events(&self, events: MouseEventTypes) {
        self.events.set(events);
    }
    pub fn set_restriction(&self, restrict: MouseRestriction) {
        let previous = self.restrict.replace(restrict);
        drop(previous);
    }
    pub(super) fn snapshot(&self) -> MouseProtocol {
        MouseProtocol {
            events: self.events.get(),
            restrict: Rc::clone(&self.restrict.borrow()),
        }
    }
}

pub(super) fn default_protocols() -> Vec<(String, MouseProtocol)> {
    use MouseEventTypes as Events;
    let all = Events::DOWN | Events::UP | Events::WHEEL | Events::DRAG | Events::MOVE;
    let specifications = [
        ("NONE", Events::NONE, Protocol::None),
        ("X10", Events::DOWN, Protocol::X10),
        (
            "VT200",
            Events::DOWN | Events::UP | Events::WHEEL,
            Protocol::Vt200,
        ),
        ("DRAG", all_without_move(), Protocol::Drag),
        ("ANY", all, Protocol::Any),
    ];
    specifications
        .into_iter()
        .map(|(name, events, policy)| {
            let restrict: MouseRestriction = Rc::new(move |event| match policy.restrict(event) {
                Ok(value) => value,
                Err(never) => match never {},
            });
            (name.into(), MouseProtocol { events, restrict })
        })
        .collect()
}

fn all_without_move() -> MouseEventTypes {
    MouseEventTypes::DOWN | MouseEventTypes::UP | MouseEventTypes::WHEEL | MouseEventTypes::DRAG
}

pub(super) fn default_encodings() -> Vec<(String, MouseEncoding)> {
    [
        ("DEFAULT", Encoding::Default),
        ("SGR", Encoding::Sgr),
        ("SGR_PIXELS", Encoding::SgrPixels),
    ]
    .into_iter()
    .map(|(name, policy)| {
        let encode: MouseEncoding = Rc::new(move |event| match policy.encode(event) {
            Ok(value) => value,
            Err(never) => match never {},
        });
        (name.into(), encode)
    })
    .collect()
}
