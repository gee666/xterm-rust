use std::rc::Rc;
use xterm_core::{
    buffer::Buffer,
    charset::{builtin_charset, Charset},
    core_service::CursorStyle,
    terminal::CoreTerminal,
};

fn map_id(map: &Option<Charset>) -> String {
    match map {
        None => "-".into(),
        Some(map) => "04567=ACEHKQRYZ"
            .chars()
            .find(|&id| builtin_charset(id).is_some_and(|other| Charset::ptr_eq(map, &other)))
            .map_or_else(|| format!("custom:{map:?}"), |id| id.to_string()),
    }
}
fn maps(maps: &[Option<Charset>]) -> String {
    maps.iter().map(map_id).collect::<Vec<_>>().join(",")
}
fn saved(buffer: &Buffer) -> String {
    let attr = &buffer.saved_cur_attr_data;
    [
        format!("{},{}", buffer.saved_x, buffer.saved_y),
        format!(
            "{},{},{},{}",
            attr.fg,
            attr.bg,
            attr.extended.ext(),
            attr.extended.url_id()
        ),
        map_id(&buffer.saved_charset),
        maps(&buffer.saved_charsets),
        buffer.saved_glevel.to_string(),
        format!(
            "{},{}",
            u8::from(buffer.saved_origin_mode),
            u8::from(buffer.saved_wraparound_mode)
        ),
        buffer
            .tabs
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        buffer.lines.max_length().to_string(),
    ]
    .join("~")
}
pub fn snapshot(terminal: &CoreTerminal) -> Result<String, Box<dyn std::error::Error>> {
    let state = terminal.state();
    let normal = state.buffers.normal();
    let alternate = state.buffers.alt();
    let modes = state.core.dec_private_modes.borrow();
    let style = match modes.cursor_style {
        None => "-",
        Some(CursorStyle::Block) => "block",
        Some(CursorStyle::Underline) => "underline",
        Some(CursorStyle::Bar) => "bar",
    };
    let blink = modes
        .cursor_blink
        .map_or_else(|| "-".into(), |value| u8::from(value).to_string());
    let meta = [
        map_id(&state.charset.charset),
        maps(state.charset.charsets()),
        state.charset.glevel.to_string(),
        super::base::hex(&state.title),
        super::base::hex(&state.icon_name),
        style.into(),
        blink,
        u8::from(modes.color_scheme_updates).to_string(),
        u8::from(modes.win32_input_mode).to_string(),
        u8::from(state.core.is_cursor_initialized.get()).to_string(),
        u8::from(state.core.is_cursor_hidden.get()).to_string(),
        super::base::hex(&state.options.get("cursorStyle")?.to_utf16()?),
        u8::from(state.options.get("cursorBlink")?.is_truthy()).to_string(),
        super::base::hex(
            &state
                .unicode
                .active_version()
                .encode_utf16()
                .collect::<Vec<_>>(),
        ),
    ]
    .join("~");
    let active = if Rc::ptr_eq(&state.active_buffer(), &alternate) {
        1
    } else {
        0
    };
    let extra = format!(
        "{}|{}|{}|{active}",
        saved(&normal.borrow()),
        saved(&alternate.borrow()),
        meta
    );
    drop(modes);
    drop(state);
    Ok(format!("RICH2|{}|{extra}", super::base::snapshot(terminal)))
}
pub fn events(terminal: &mut CoreTerminal) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    for event in terminal.drain_events() {
        use xterm_core::terminal::{TerminalEvent as E, WindowReport};
        let encoded = match &event {
            E::RequestReset => Some("RESET".into()),
            E::SyncScrollBar => Some("SYNC".into()),
            E::SendFocus => Some("FOCUS".into()),
            E::ColorSchemeQuery => Some("SCHEME".into()),
            E::WindowReport(kind) => Some(format!(
                "WIN{}",
                match kind {
                    WindowReport::TextAreaPixels => 0,
                    WindowReport::CellPixels => 1,
                }
            )),
            E::Resize { cols, rows } => Some(format!("SIZE{cols}:{rows}")),
            E::Binary(text) => Some(format!("BIN{}", super::base::hex(text))),
            // WriteParsed is an asynchronous host-queue observation, covered by
            // queue/public API cases rather than these completed handler steps.
            E::WriteParsed | E::InputScroll(_) => None,
            E::AccessibilityChar(text) => Some(format!("A{}", super::base::hex(text))),
            E::AccessibilityTab(amount) => Some(format!("TAB{amount}")),
            E::Color(_) => {
                return Err(
                    "color request capture is outside the print/control trace domain".into(),
                )
            }
            E::Bell
            | E::CursorMove
            | E::LineFeed
            | E::Scroll(_)
            | E::Data(_)
            | E::TitleChange(_)
            | E::Render { .. } => super::base::event(&event),
        };
        if let Some(encoded) = encoded {
            output.push(encoded);
        }
    }
    Ok(output.join("/"))
}
