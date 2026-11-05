// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{columns, options, view, SerializeAddon};
use crate::{
    js_error::to_js_error,
    terminal::{Shared, Terminal},
    theme::ColorRef,
    utility_addons::js_property::get,
};
use wasm_bindgen::prelude::*;
use xterm_core::serialize::{
    serialize_html, CellRange, HtmlConfig, HtmlFontSize, HtmlOptions, Position,
};

pub(super) fn serialize(addon: &SerializeAddon, raw: JsValue) -> Result<Vec<u16>, JsValue> {
    let host = addon.host()?;
    let namespace = host.native.buffer().map_err(|error| to_js_error(&error))?;
    let active = namespace.active().map_err(|error| to_js_error(&error))?;
    // Capture the array's color identities, not a palette copied after callbacks.
    // Theme replacement changes the array; OSC updates can mutate its colors.
    let palette: Option<Vec<ColorRef>> = host
        .services
        .borrow()
        .as_ref()
        .map(|services| services.theme.colors().borrow().ansi.clone());
    let only_selection = options::property(&raw, "onlySelection")?.is_truthy();
    let range = options::property(&raw, "range")?;
    let range = if range.is_truthy() {
        let col = get(&range, "startCol")?;
        let start_col = if col.is_undefined() {
            columns(&host)?
        } else {
            options::index(col)?
        };
        get(&range, "startLine")?;
        let start_line = options::index(get(&range, "startLine")?)?;
        let end_col = columns(&host)?;
        get(&range, "endLine")?;
        let end_line = options::index(get(&range, "endLine")?)?;
        CellRange {
            start: Position {
                x: start_col,
                y: start_line,
            },
            end: Position {
                x: end_col,
                y: end_line,
            },
        }
    } else if only_selection {
        let selection_host = addon.host()?;
        let selection = Terminal {
            shared: selection_host,
        }
        .get_selection_position()?;
        if selection.is_undefined() {
            return Ok(Vec::new());
        }
        let start = get(&selection, "start")?;
        let end = get(&selection, "end")?;
        CellRange {
            start: Position {
                x: options::index(get(&start, "x")?)?,
                y: options::index(get(&start, "y")?)?,
            },
            end: Position {
                x: options::index(get(&end, "x")?)?,
                y: options::index(get(&end, "y")?)?,
            },
        }
    } else {
        let max = active.len().map_err(|error| to_js_error(&error))?;
        let scrollback = options::property(&raw, "scrollback")?;
        let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
        let start = options::scrollback_start(max, rows, scrollback)?;
        CellRange {
            start: Position { x: 0, y: start },
            end: Position {
                x: columns(&host)?,
                y: max.saturating_sub(1),
            },
        }
    };
    let include_background = options::property(&raw, "includeGlobalBackground")?.is_truthy();
    let mut config = header(&host, include_background)?;
    if let Some(palette) = palette {
        let colors: Vec<String> = palette
            .iter()
            .map(|color| color.borrow().css.clone())
            .collect();
        config.ansi_colors = Some(
            colors
                .try_into()
                .map_err(|_| js_sys::Error::new("Theme palette does not contain 256 colors"))?,
        );
    }
    let view = view::BrowserView::new(active, &host)?;
    let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
    serialize_html(
        &view,
        columns(&host)?,
        rows,
        Some(range),
        &config,
        &HtmlOptions {
            only_selection: true,
            include_global_background: include_background,
            ..HtmlOptions::default()
        },
    )
    .map_err(|error| view::error(&error))
}
fn header(host: &Shared, include_background: bool) -> Result<HtmlConfig, JsValue> {
    let (foreground, background) = if include_background {
        let foreground = theme(host, "foreground")?;
        let background = theme(host, "background")?;
        (foreground, background)
    } else {
        (None, None)
    };
    let foreground = foreground.as_ref().map(options::concat).transpose()?;
    let background = background.as_ref().map(options::concat).transpose()?;
    let font_family = options::concat(&get(&host.options.object(), "fontFamily")?)?;
    let font_size = HtmlFontSize::Text(options::concat(&get(&host.options.object(), "fontSize")?)?);
    Ok(HtmlConfig {
        foreground,
        background,
        font_family,
        font_size,
        ansi_colors: None,
    })
}
fn theme(host: &Shared, key: &str) -> Result<Option<JsValue>, JsValue> {
    let theme = get(&host.options.object(), "theme")?;
    if theme.is_undefined() || theme.is_null() {
        return Ok(None);
    }
    let value = get(&theme, key)?;
    Ok((!value.is_undefined() && !value.is_null()).then_some(value))
}
