// Licensed under MIT. Shared deterministic trace adapter, not production code.
use std::cell::{Cell, RefCell};
use std::error::Error;
use xterm_core::input::keyboard::KeyboardEvent;
use xterm_web::input::keyboard::KeyboardModes;
use xterm_web::input::keyboard_dispatch::*;

pub fn hex(units: &[u16]) -> String {
    units
        .iter()
        .map(|u| format!("{u:x}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn action_name(action: KeyboardAction<'_>) -> String {
    match action {
        KeyboardAction::UpdateCursorStyle => "cursor".into(),
        KeyboardAction::ScrollLines(count) => format!("scroll:{count}"),
        KeyboardAction::ScrollToBottom => "bottom".into(),
        KeyboardAction::SelectAll => "select".into(),
        KeyboardAction::Cancel => "cancel".into(),
        KeyboardAction::ClearTextarea => "clear".into(),
        KeyboardAction::OnKey(key) => format!("key:{}", hex(key)),
        KeyboardAction::ShowCursor => "show".into(),
        KeyboardAction::Data { key, user_input } => {
            format!("data:{}:{}", u8::from(user_input), hex(key))
        }
        KeyboardAction::Focus => "focus".into(),
    }
}

pub struct Host {
    pub snapshot: Cell<KeyboardSnapshot>,
    pub accepted: Cell<bool>,
    pub has_custom: Cell<bool>,
    pub composition: Cell<bool>,
    pub actions: RefCell<Vec<String>>,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            snapshot: Cell::new(KeyboardSnapshot {
                rows: 24,
                ..KeyboardSnapshot::default()
            }),
            accepted: Cell::new(true),
            has_custom: Cell::new(true),
            composition: Cell::new(true),
            actions: RefCell::new(Vec::new()),
        }
    }
}

impl KeyboardReadHost for Host {
    type Error = &'static str;
    fn snapshot(&self) -> Result<KeyboardSnapshot, Self::Error> {
        Ok(self.snapshot.get())
    }
}
impl KeyboardHost for Host {
    fn custom_key_handler(
        &self,
        _: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        if self.has_custom.get() {
            self.actions.borrow_mut().push("custom".into());
            Ok(self.accepted.get())
        } else {
            Ok(true)
        }
    }
    fn composition_key_down(
        &self,
        _: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        self.actions.borrow_mut().push("composition".into());
        Ok(self.composition.get())
    }
    fn perform(
        &self,
        _: Option<&dyn KeyEventSource<Error = Self::Error>>,
        action: KeyboardAction<'_>,
    ) -> Result<(), Self::Error> {
        self.actions.borrow_mut().push(action_name(action));
        Ok(())
    }
}

fn units(text: &str) -> Result<Vec<u16>, std::num::ParseIntError> {
    text.split(',')
        .filter(|s| !s.is_empty())
        .map(|s| u16::from_str_radix(s, 16))
        .collect()
}

fn optional(text: &str) -> Result<Option<u32>, std::num::ParseIntError> {
    if text == "-" {
        Ok(None)
    } else {
        text.parse().map(Some)
    }
}

pub fn row(dispatch: &KeyboardDispatch, host: &Host, line: &str) -> Result<String, Box<dyn Error>> {
    let fields: Vec<_> = line.split('\t').collect();
    if fields.len() != 15 {
        return Err("expected 15 trace input fields".into());
    }
    let key = units(fields[2])?;
    let code = units(fields[3])?;
    let modifiers: u8 = fields[4].parse()?;
    let event_bits: u8 = fields[5].parse()?;
    let options: u32 = fields[8].parse()?;
    host.snapshot.set(KeyboardSnapshot {
        modes: KeyboardModes {
            win32_extension: options & 1 != 0,
            win32_mode: options & 2 != 0,
            kitty_extension: options & 4 != 0,
            application_cursor_keys: options & 8 != 0,
            is_mac: options & 4096 != 0,
            mac_option_is_meta: options & 32 != 0,
            kitty_flags: fields[9].parse()?,
        },
        is_mac: options & 16 != 0,
        is_windows: options & 64 != 0,
        screen_reader_mode: options & 128 != 0,
        scroll_on_user_input: options & 256 != 0,
        rows: fields[10].parse()?,
        ybase: fields[11].parse()?,
        ydisp: fields[12].parse()?,
    });
    host.accepted.set(options & 512 != 0);
    host.has_custom.set(options & 8192 == 0);
    host.composition.set(options & 1024 != 0);
    host.actions.borrow_mut().clear();
    let event = DispatchKeyEvent {
        key: KeyboardEvent {
            key_code: fields[1].parse()?,
            key: &key,
            code: &code,
            shift_key: modifiers & 1 != 0,
            alt_key: modifiers & 2 != 0,
            ctrl_key: modifiers & 4 != 0,
            meta_key: modifiers & 8 != 0,
        },
        kind: match fields[0] {
            "P" => KeyEventKind::Press,
            "U" => KeyEventKind::Up,
            "D" | "I" => KeyEventKind::Down,
            _ => return Err("unknown trace event kind".into()),
        },
        repeat: event_bits & 1 != 0,
        alt_graph: event_bits & 2 != 0,
        char_code: optional(fields[6])?,
        which: optional(fields[7])?,
    };
    let returned = match fields[0] {
        "D" => dispatch
            .key_down(&event, host)?
            .map(|value| u8::from(value).to_string())
            .unwrap_or_else(|| "u".into()),
        "P" => u8::from(dispatch.key_press(&event, host)?).to_string(),
        "U" => {
            dispatch.key_up(&event, host)?;
            "u".into()
        }
        "I" => {
            let data = if fields[13] == "-" {
                None
            } else {
                Some(units(fields[13])?)
            };
            let input_bits: u8 = fields[14].parse()?;
            u8::from(dispatch.input(
                TextInputEvent {
                    data: data.as_deref(),
                    insert_text: input_bits & 1 != 0,
                    composed: input_bits & 2 != 0,
                },
                host,
            )?)
            .to_string()
        }
        _ => return Err("unknown trace event kind".into()),
    };
    let state = dispatch.state();
    let state_bits = u8::from(state.key_down_handled)
        | (u8::from(state.key_down_seen) << 1)
        | (u8::from(state.key_press_handled) << 2)
        | (u8::from(state.unprocessed_dead_key) << 3);
    Ok(format!(
        "{returned}\t{state_bits}\t{}",
        host.actions.borrow().join("|")
    ))
}
