//! Line-protocol adapter for scripts/check-core-services.mjs. No terminal engine
//! or upstream JS runs here; this exercises the Rust services directly.
use std::cell::{Cell, RefCell};
use std::io::{self, BufRead};
use std::rc::Rc;
use xterm_core::core_service::{
    BufferPosition, CoreService, CursorStyle, DecPrivateModes, KittyKeyboardState,
};
use xterm_core::lifecycle::Disposable;
use xterm_core::log_service::{
    CustomLogger, LogError, LogLevel, LogRecord, LogRoute, LogService, LogSink, LogValue,
};
use xterm_core::options::{CoercionError, OptionObject, OptionValue, OptionsService};

struct Sink(Rc<RefCell<Vec<String>>>);
impl LogSink for Sink {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        let parameters = record
            .parameters
            .iter()
            .map(|value| match value {
                LogValue::CodeUnits(units) => hex(units),
                LogValue::Value(_) => "unexpected-value".into(),
            })
            .collect::<Vec<_>>()
            .join("/");
        self.0.borrow_mut().push(format!(
            "L:{}:{}:{}:{}",
            record.level as u8,
            match record.route {
                LogRoute::Log => "log",
                LogRoute::Info => "info",
                LogRoute::Warn => "warn",
                LogRoute::Error => "error",
                LogRoute::Custom => "custom",
            },
            hex(&record.message),
            parameters
        ));
        Ok(())
    }
}
fn hex(units: &[u16]) -> String {
    units
        .iter()
        .map(|unit| format!("{unit:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn units(input: &str) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    if input.is_empty() {
        return Ok(vec![]);
    }
    input
        .split(',')
        .map(|value| Ok(u16::from_str_radix(value, 16)?))
        .collect()
}
fn snapshot(core: &CoreService) -> String {
    let modes = core.dec_private_modes.borrow();
    let flags = [
        modes.application_cursor_keys,
        modes.application_keypad,
        modes.bracketed_paste_mode,
        modes.color_scheme_updates,
        modes.origin,
        modes.reverse_wraparound,
        modes.send_focus,
        modes.synchronized_output,
        modes.win32_input_mode,
        modes.wraparound,
    ]
    .map(|flag| if flag { '1' } else { '0' })
    .iter()
    .collect::<String>();
    let kitty = core.kitty_keyboard.borrow();
    format!(
        "{},{},{}|{}:{}:{}|{},{},{}:{:?}:{:?}",
        u8::from(core.is_cursor_initialized.get()),
        u8::from(core.is_cursor_hidden.get()),
        u8::from(core.modes.borrow().insert_mode),
        flags,
        match modes.cursor_blink {
            None => "u",
            Some(false) => "f",
            Some(true) => "t",
        },
        match modes.cursor_style {
            None => "u",
            Some(CursorStyle::Block) => "0",
            Some(CursorStyle::Underline) => "1",
            Some(CursorStyle::Bar) => "2",
        },
        kitty.flags,
        kitty.main_flags,
        kitty.alt_flags,
        kitty.main_stack,
        kitty.alt_stack
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Rc::new(OptionsService::default());
    let events = Rc::new(RefCell::new(Vec::<String>::new()));
    let sink: Rc<dyn LogSink> = Rc::new(Sink(Rc::clone(&events)));
    let logger = Rc::new(LogService::new(Rc::clone(&options), Rc::clone(&sink))?);
    let position = Rc::new(Cell::new(BufferPosition::default()));
    let read = Rc::clone(&position);
    let core = CoreService::new(Rc::clone(&options), Rc::clone(&logger), move || read.get())?;
    let captured = Rc::clone(&events);
    core.on_data()
        .subscribe(move |data| captured.borrow_mut().push(format!("D:{}", hex(data))));
    let captured = Rc::clone(&events);
    core.on_binary()
        .subscribe(move |data| captured.borrow_mut().push(format!("B:{}", hex(data))));
    let captured = Rc::clone(&events);
    core.on_user_input()
        .subscribe(move |_| captured.borrow_mut().push("user".into()));
    let captured = Rc::clone(&events);
    core.on_request_scroll_to_bottom()
        .subscribe(move |_| captured.borrow_mut().push("scroll".into()));
    for line in io::stdin().lock().lines() {
        let line = line?;
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["O", key, kind, text] => {
                let value = match *kind {
                    "u" => OptionValue::Undefined,
                    "z" => OptionValue::Null,
                    "b" => OptionValue::Bool(*text == "1"),
                    "n" => OptionValue::Number(text.parse()?),
                    "s" => OptionValue::String(units(text)?),
                    _ => return Err("unknown option type".into()),
                };
                options.set(key, value)?;
            }
            ["P", base, display] => position.set(BufferPosition {
                ybase: base.parse()?,
                ydisp: display.parse()?,
            }),
            ["D", user, text] => core.trigger_data_event(&units(text)?, *user == "1")?,
            ["B", text] => core.trigger_binary_event(&units(text)?)?,
            ["L", level, text] => logger.log(
                match *level {
                    "0" => LogLevel::Trace,
                    "1" => LogLevel::Debug,
                    "2" => LogLevel::Info,
                    "3" => LogLevel::Warn,
                    "4" => LogLevel::Error,
                    _ => return Err("unknown level".into()),
                },
                &units(text)?,
                vec![],
            )?,
            ["C", enabled] => options.set(
                "logger",
                if *enabled == "1" {
                    OptionValue::object(CustomLogger(Rc::clone(&sink)))
                } else {
                    OptionValue::Null
                },
            )?,
            ["E"] => {
                let failed = OptionValue::Object(OptionObject::new((), |_| {
                    Err(CoercionError(vec![0xd800, 0]))
                }));
                let error = options
                    .set("logLevel", failed)
                    .expect_err("throwing ToPrimitive must fail");
                events
                    .borrow_mut()
                    .push(format!("error:{}", hex(&error.message_utf16())));
            }
            ["R"] => core.reset(),
            ["M"] => {
                core.is_cursor_initialized.set(true);
                core.is_cursor_hidden.set(true);
                core.modes.borrow_mut().insert_mode = true;
                *core.dec_private_modes.borrow_mut() = DecPrivateModes {
                    application_cursor_keys: true,
                    application_keypad: true,
                    bracketed_paste_mode: true,
                    color_scheme_updates: true,
                    origin: true,
                    reverse_wraparound: true,
                    send_focus: true,
                    synchronized_output: true,
                    win32_input_mode: true,
                    wraparound: false,
                    cursor_blink: Some(true),
                    cursor_style: Some(CursorStyle::Underline),
                };
                *core.kitty_keyboard.borrow_mut() = KittyKeyboardState {
                    flags: u32::MAX,
                    main_flags: 7,
                    alt_flags: 8,
                    main_stack: vec![1, 2],
                    alt_stack: vec![3, 4],
                };
            }
            ["X"] => core.dispose(),
            ["Y"] => logger.dispose(),
            _ => return Err(format!("unknown operation: {line}").into()),
        }
        println!("{}\t{}", snapshot(&core), events.borrow().join(";"));
        events.borrow_mut().clear();
    }
    Ok(())
}
