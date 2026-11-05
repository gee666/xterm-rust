// Licensed under MIT. Source Win32 map identities and portable lookup data.
pub use super::super::keyboard_event_source::lookup::{Hint, HostValue, Value};
use super::key_codes;

/// Each source object has independent identity and mutable own properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    VirtualKey,
    ScanCode,
    ControlChar,
}

/// Native events have no mutable JS realm. Their ordinary intrinsic values use
/// the documented Node/V8 representation; DOM events override the entire lookup.
pub fn native_lookup<E>(table: Table, key: &[u16]) -> Value<E> {
    let own = match table {
        Table::VirtualKey => key_codes::virtual_key(key),
        Table::ScanCode => key_codes::scan_code(key).map(u32::from),
        Table::ControlChar => key_codes::control_char(key),
    };
    if let Some(value) = own {
        return Value::Number(f64::from(value));
    }
    super::super::keyboard_event_source::inherited::inherited_name(key)
        .map(|value| Value::Text(value.encode_utf16().collect()))
        .unwrap_or(Value::Undefined)
}
