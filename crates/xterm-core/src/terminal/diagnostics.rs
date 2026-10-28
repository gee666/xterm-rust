//! InputHandler diagnostic arguments are built at their source callback phase.
//! No terminal-state borrow crosses logging or a retained input-source callback.
use super::TerminalError;
use crate::{
    event::EmitError,
    input::parse_source::{ParseInputSource, ParseSourceError},
    log_service::{LogError, LogLevel, LogParameter, LogService, LogValue},
    options::OptionValue,
    parser::{
        apc::ApcAction,
        dcs::DcsAction,
        escape::{EscapeSequenceParser, ParseError},
        handler::DispatchError,
        osc::OscAction,
        params::{ParamValue, Params},
    },
};
use std::rc::Rc;

impl From<ParseSourceError> for TerminalError {
    fn from(error: ParseSourceError) -> Self {
        Self::Event(EmitError::callback(error))
    }
}

/// Call after cursor capture and before dirty-range clearing or decoding. On a
/// resumed parse, call only after the paused parser finishes its current chunk.
pub fn log_source(logger: &LogService, source: &dyn ParseInputSource) -> Result<(), TerminalError> {
    if logger.level().is_some_and(|level| level <= LogLevel::Debug) {
        let mut message: Vec<_> = "parsing data  \"".encode_utf16().collect();
        message.extend(source.debug_units()?);
        message.push(u16::from(b'"'));
        logger.log(LogLevel::Debug, &message, Vec::new())?;
    }
    // Debug callbacks can replace the level or mutate the live byte view.
    if logger.level() == Some(LogLevel::Trace) {
        let payload = source.trace_value()?;
        logger.log(
            LogLevel::Trace,
            &"parsing data (codes)".encode_utf16().collect::<Vec<_>>(),
            vec![LogParameter::Value(payload)],
        )?;
    }
    Ok(())
}

fn dispatch_error(error: LogError) -> DispatchError {
    DispatchError::Callback(EmitError::callback(error))
}
fn parse_error(error: LogError) -> ParseError {
    ParseError::Dispatch(dispatch_error(error))
}
fn text(value: &str) -> LogValue {
    LogValue::Value(value.into())
}
fn units(value: Vec<u16>) -> LogValue {
    LogValue::Value(OptionValue::String(value))
}
fn number(value: f64) -> LogValue {
    LogValue::Value(value.into())
}
fn boolean(value: bool) -> LogValue {
    LogValue::Value(value.into())
}
fn identifier(value: u32) -> LogValue {
    text(&EscapeSequenceParser::ident_to_string(value))
}
fn params_array(params: &Params) -> LogValue {
    LogValue::Array(
        params
            .to_array()
            .into_iter()
            .map(|entry| match entry {
                ParamValue::Param(value) => number(f64::from(value)),
                ParamValue::SubParams(values) => LogValue::Array(
                    values
                        .into_iter()
                        .map(|value| number(f64::from(value)))
                        .collect(),
                ),
            })
            .collect(),
    )
}
fn debug(
    logger: &LogService,
    message: &str,
    fields: Vec<(&str, LogValue)>,
) -> Result<(), LogError> {
    let fields = fields
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    logger.log(
        LogLevel::Debug,
        &message.encode_utf16().collect::<Vec<_>>(),
        vec![LogParameter::Value(LogValue::Object(fields))],
    )
}

/// Installs real checked callbacks, so a logger throw aborts at that fallback,
/// before subsequent parser state clearing or characters can run.
pub fn install_fallbacks(parser: &mut EscapeSequenceParser, logger: Rc<LogService>) {
    let log = Rc::clone(&logger);
    parser.set_csi_handler_fallback_checked(move |id, params| {
        debug(
            &log,
            "Unknown CSI code: ",
            vec![
                ("identifier", identifier(id)),
                ("params", params_array(params)),
            ],
        )
        .map_err(parse_error)
    });
    let log = Rc::clone(&logger);
    parser.set_esc_handler_fallback_checked(move |id| {
        debug(
            &log,
            "Unknown ESC code: ",
            vec![("identifier", identifier(id))],
        )
        .map_err(parse_error)
    });
    let log = Rc::clone(&logger);
    parser.set_execute_handler_fallback_checked(move |code| {
        debug(
            &log,
            "Unknown EXECUTE code: ",
            vec![("code", number(f64::from(code)))],
        )
        .map_err(parse_error)
    });
    let log = Rc::clone(&logger);
    parser.osc.set_handler_fallback_checked(move |id, action| {
        let (action, data) = match action {
            OscAction::Start => ("START", LogValue::Value(OptionValue::Undefined)),
            OscAction::Put(value) => ("PUT", units(value)),
            OscAction::End(value) => ("END", boolean(value)),
        };
        debug(
            &log,
            "Unknown OSC code: ",
            vec![
                ("identifier", number(id)),
                ("action", text(action)),
                ("data", data),
            ],
        )
        .map_err(dispatch_error)
    });
    let log = Rc::clone(&logger);
    parser.dcs.set_handler_fallback_checked(move |id, action| {
        let (action, payload) = match action {
            DcsAction::Hook(params) => ("HOOK", params_array(&params)),
            DcsAction::Put(value) => ("PUT", units(value)),
            DcsAction::Unhook(value) => ("UNHOOK", boolean(value)),
        };
        // Packed protocol identifiers retain their int32 bit pattern.
        let id = u32::from_ne_bytes(id.to_ne_bytes());
        debug(
            &log,
            "Unknown DCS code: ",
            vec![
                ("identifier", identifier(id)),
                ("action", text(action)),
                ("payload", payload),
            ],
        )
        .map_err(dispatch_error)
    });
    parser.apc.set_handler_fallback_checked(move |id, action| {
        let (action, payload) = match action {
            ApcAction::Start => ("START", LogValue::Value(OptionValue::Undefined)),
            ApcAction::Put(value) => ("PUT", units(value)),
            ApcAction::End(value) => ("END", boolean(value)),
        };
        debug(
            &logger,
            "Unknown APC code: ",
            vec![
                ("identifier", identifier(id)),
                ("action", text(action)),
                ("payload", payload),
            ],
        )
        .map_err(dispatch_error)
    });
}

pub fn keypad(logger: &LogService, application: bool) -> Result<(), LogError> {
    let message = if application {
        "Serial port requested application keypad."
    } else {
        "Switching back to normal keypad."
    };
    logger.log(
        LogLevel::Debug,
        &message.encode_utf16().collect::<Vec<_>>(),
        Vec::new(),
    )
}
