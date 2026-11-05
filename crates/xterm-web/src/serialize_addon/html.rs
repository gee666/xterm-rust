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
    serialize_html, serialize_html_numeric_rows, CellRange, HtmlConfig, HtmlFontSize,
    HtmlNumericRange, HtmlOptions, Position,
};

enum Range {
    Integral(CellRange),
    Numeric(HtmlNumericRange),
}

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
        explicit_range(&host, &range)?
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
        Range::Integral(CellRange {
            start: Position {
                x: options::index(get(&start, "x")?)?,
                y: options::index(get(&start, "y")?)?,
            },
            end: Position {
                x: options::index(get(&end, "x")?)?,
                y: options::index(get(&end, "y")?)?,
            },
        })
    } else {
        let max = active.len().map_err(|error| to_js_error(&error))?;
        let scrollback = options::property(&raw, "scrollback")?;
        let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
        if scrollback.as_f64().is_some_and(f64::is_nan) {
            // NaN survives the source clamp and makes row <= end false. Do not
            // reuse ANSI's invalid-array-length error or skip header callbacks.
            empty_rows(columns(&host)?)
        } else {
            let start = options::scrollback_start(max, rows, scrollback)?;
            Range::Integral(CellRange {
                start: Position { x: 0, y: start },
                end: Position {
                    x: columns(&host)?,
                    y: max.saturating_sub(1),
                },
            })
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
    match range {
        Range::Numeric(range) => {
            serialize_html_numeric_rows(&view, range, &config, include_background)
        }
        Range::Integral(range) => serialize_html(
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
        ),
    }
    .map_err(|error| view::error(&error))
}
fn explicit_range(host: &Shared, range: &JsValue) -> Result<Range, JsValue> {
    let col = get(range, "startCol")?;
    get(range, "startLine")?;
    let start = get(range, "startLine")?;
    let end_col = columns(host)?;
    get(range, "endLine")?;
    let end = get(range, "endLine")?;
    if matches!((start.as_f64(), end.as_f64()), (Some(start), Some(end)) if start.is_nan() || end.is_nan() || start > end)
    {
        // A false numeric row comparison skips the cell loop entirely, even for
        // reversed infinite bounds. Preserve the header/footer and do not coerce
        // an unused column. Objects still need late source coercion under task346.
        return Ok(empty_rows(end_col));
    }
    let start_col = if col.is_undefined()
        || col
            .as_f64()
            .is_some_and(|col| col.is_nan() || col >= u32::MAX as f64)
    {
        // No native wasm32 line can have a valid column this large. Like NaN
        // and undefined, it skips the first-row loop even after header resize.
        // Substituting the captured end column would visit newly grown cells.
        usize::MAX
    } else {
        options::index(col)?
    };
    if let (Some(start), Some(end)) = (start.as_f64(), end.as_f64()) {
        if start.is_finite() && end.is_finite() {
            return Ok(Range::Numeric(HtmlNumericRange {
                start_line: start,
                end_line: end,
                start_col,
                end_col,
            }));
        }
    }
    Ok(Range::Integral(CellRange {
        start: Position {
            x: start_col,
            y: options::index(start)?,
        },
        end: Position {
            x: end_col,
            y: options::index(end)?,
        },
    }))
}
fn empty_rows(end_col: usize) -> Range {
    Range::Integral(CellRange {
        start: Position { x: 0, y: 1 },
        end: Position { x: end_col, y: 0 },
    })
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
