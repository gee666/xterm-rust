// Licensed under MIT. Source Kitty ordinary-object table identities.
use super::keys;
pub use crate::input::keyboard_event_source::lookup::{Hint, HostValue, Value};

/// The four objects owned by one Kitty handler, in source declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    Functional,
    Tilde,
    Letter,
    Ss3,
}

/// Immutable native event lookup uses the documented Node/V8 intrinsic text.
/// Runtime event sources replace this with their own persistent ordinary maps.
pub fn native_lookup<E>(table: Table, key: &[u16]) -> Value<E> {
    let own = match table {
        Table::Functional => keys::functional(key).map(|n| Value::Number(f64::from(n))),
        Table::Tilde => keys::tilde(key).map(|n| Value::Number(f64::from(n))),
        Table::Letter => keys::letter(key).map(|text| Value::Text(text.encode_utf16().collect())),
        Table::Ss3 => keys::ss3(key).map(|text| Value::Text(text.encode_utf16().collect())),
    };
    own.unwrap_or_else(|| {
        crate::input::keyboard_event_source::inherited::inherited_name(key)
            .map(|text| Value::Text(text.encode_utf16().collect()))
            .unwrap_or(Value::Undefined)
    })
}
