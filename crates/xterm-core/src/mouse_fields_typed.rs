// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{to_int32, MouseEvent};
use crate::mouse_fields::{Field, Modifier, MouseFields};
use std::convert::Infallible;

pub enum Value {
    Number(f64),
    Boolean(bool),
    Undefined,
}

impl Value {
    fn number(&self) -> f64 {
        match self {
            Self::Number(value) => *value,
            Self::Boolean(value) => f64::from(*value),
            Self::Undefined => f64::NAN,
        }
    }
}

impl MouseFields for MouseEvent {
    type Value = Value;
    type Error = Infallible;

    fn get(&self, field: Field) -> Result<Value, Infallible> {
        let modifier = |value: Option<bool>| value.map_or(Value::Undefined, Value::Boolean);
        Ok(match field {
            Field::Ctrl => modifier(self.ctrl),
            Field::Alt => modifier(self.alt),
            Field::Shift => modifier(self.shift),
            Field::Button => Value::Number(self.button.numeric_value()),
            Field::Action => Value::Number(f64::from(self.action as u8)),
            Field::Col => Value::Number(self.col),
            Field::Row => Value::Number(self.row),
            Field::X => Value::Number(self.x),
            Field::Y => Value::Number(self.y),
        })
    }
    fn set_false(&mut self, modifier: Modifier) -> Result<(), Infallible> {
        match modifier {
            Modifier::Ctrl => self.ctrl = Some(false),
            Modifier::Alt => self.alt = Some(false),
            Modifier::Shift => self.shift = Some(false),
        }
        Ok(())
    }
    fn truth(value: &Value) -> bool {
        match value {
            Value::Boolean(value) => *value,
            Value::Number(value) => *value != 0.0 && !value.is_nan(),
            Value::Undefined => false,
        }
    }
    fn equals_number(value: &Value, number: i32) -> bool {
        matches!(value, Value::Number(value) if *value == f64::from(number))
    }
    fn bit_and(value: &Value, mask: i32) -> Result<i32, Infallible> {
        Ok(to_int32(value.number()) & mask)
    }
    fn bit_or(code: i32, value: &Value) -> Result<i32, Infallible> {
        Ok(code | to_int32(value.number()))
    }
    fn add_number(value: &Value, number: i32) -> Result<Value, Infallible> {
        Ok(Value::Number(value.number() + f64::from(number)))
    }
    fn greater(value: &Value, number: i32) -> Result<bool, Infallible> {
        Ok(value.number() > f64::from(number))
    }
    fn char_code(value: &Value) -> Result<u16, Infallible> {
        let value = value.number();
        Ok(if value.is_finite() && value != 0.0 {
            value.trunc().rem_euclid(65536.0) as u16
        } else {
            0
        })
    }
    fn text(value: &Value) -> Result<Vec<u16>, Infallible> {
        Ok(match value {
            Value::Number(number) => ryu_js::Buffer::new()
                .format(*number)
                .encode_utf16()
                .collect(),
            Value::Boolean(true) => "true".encode_utf16().collect(),
            Value::Boolean(false) => "false".encode_utf16().collect(),
            Value::Undefined => "undefined".encode_utf16().collect(),
        })
    }
}
