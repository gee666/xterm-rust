use super::{ops, view::NumberView, SerializeBackend};
use crate::{
    js_error::to_js_error, selection::service::buffer_dimensions, terminal::Shared,
    utf16::to_js_string,
};
use js_sys::JsString;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::serialize::{serialize_html_range, CellRange, HtmlConfig, HtmlFontSize, Position};

pub(super) fn serialize(addon: &SerializeBackend, options: JsValue) -> Result<JsString, JsValue> {
    let host = addon.current()?;
    let source = host
        .native
        .buffer()
        .map_err(|error| to_js_error(&error))?
        .active()
        .map_err(|error| to_js_error(&error))?;
    // The source handler captures its ANSI array before option/header getters.
    let palette = host
        .services
        .borrow()
        .as_ref()
        .map(|services| services.theme.colors().borrow().ansi.clone());
    let options = if options.is_null() || options.is_undefined() {
        js_sys::Object::new().into()
    } else {
        options
    };
    let only = ops::property(&options, "onlySelection")?;
    let range = ops::property(&options, "range")?;
    let (first, last, column, end_column) = if ops::serialTruthy(&range) {
        let column = ops::property(&range, "startCol")?;
        let first = ops::serialHtmlStart(&range)?;
        let end_column = host.native.cols().map_err(|error| to_js_error(&error))? as f64;
        let last = ops::serialHtmlEnd(&range)?;
        (
            ops::serialNumber(&first)?,
            ops::serialNumber(&last)?,
            ops::serialNumber(&column)?,
            end_column,
        )
    } else if ops::serialTruthy(&only) {
        let selection_host = addon.current()?;
        let Some((first, last, column, end_column)) = selection(&selection_host)? else {
            return Ok(to_js_string(&[]));
        };
        (first, last, column, end_column)
    } else {
        let length = source.len().map_err(|error| to_js_error(&error))? as f64;
        let count = ops::property(&options, "scrollback")?;
        let rows = host.native.rows().map_err(|error| to_js_error(&error))? as f64;
        let retained = ops::serialRetained(&count, rows, length)?;
        let cols = host.native.cols().map_err(|error| to_js_error(&error))? as f64;
        (length - retained, length - 1.0, 0.0, cols)
    };
    let include = ops::serialTruthy(&ops::property(&options, "includeGlobalBackground")?);
    let config = header_config(&host, include, palette)?;
    let row_count = if first.is_nan() || last.is_nan() || first > last {
        0
    } else {
        if !first.is_finite() || !last.is_finite() {
            return Err(js_sys::RangeError::new("Non-finite serialization traversal").into());
        }
        super::index((last - first).floor())?
            .checked_add(1)
            .ok_or_else(|| js_sys::RangeError::new("Invalid array length"))?
    };
    let first_end = if row_count == 1 {
        end_column
    } else {
        source
            .get_line_number(first)
            .map_err(|error| to_js_error(&error))?
            .map(|line| line.borrow().len() as f64)
            .unwrap_or(0.0)
    };
    let columns = NumberView::columns(column, first_end)?;
    let last_count = if row_count == 1 {
        columns.len()
    } else {
        NumberView::columns(0.0, end_column)?.len()
    };
    let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
    let view = NumberView::new(source, first, 0, rows, Some(columns))
        .map_err(|error| super::serialization_error(&error))?;
    let range = (row_count > 0).then(|| CellRange {
        start: Position { x: 0, y: 0 },
        end: Position {
            x: last_count,
            y: row_count - 1,
        },
    });
    let result = serialize_html_range(&view, range, &config, include);
    for (row, col) in view.warnings() {
        warn(row, col)?;
    }
    result
        .map(|output| to_js_string(&output))
        .map_err(|error| super::serialization_error(&error))
}
fn selection(host: &Shared) -> Result<Option<(f64, f64, f64, f64)>, JsValue> {
    if host.dom.borrow().is_none() {
        return Ok(None);
    }
    let buffers = host
        .native
        .with_state(|state| Rc::clone(&state.buffer_service))
        .map_err(|error| to_js_error(&error))?;
    let dimensions = buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?;
    if !host
        .selection
        .has_selection(dimensions)
        .map_err(|error| to_js_error(&error))?
    {
        return Ok(None);
    }
    let start = host.selection.selection_start();
    let end = host
        .selection
        .selection_end(dimensions)
        .map_err(|error| to_js_error(&error))?;
    Ok(start.zip(end).map(|(start, end)| {
        (
            start.row as f64,
            end.row as f64,
            start.column as f64,
            end.column as f64,
        )
    }))
}
fn header_config(
    host: &Shared,
    include: bool,
    palette: Option<Vec<crate::theme::ColorRef>>,
) -> Result<HtmlConfig, JsValue> {
    let options = host.native.options();
    let (foreground, background) = if include {
        let fg = options.get("theme").map_err(|error| to_js_error(&error))?;
        let fg = theme_value(fg, "foreground")?;
        let bg = options.get("theme").map_err(|error| to_js_error(&error))?;
        (fg, theme_value(bg, "background")?)
    } else {
        (None, None)
    };
    let concat = |value: xterm_core::options::OptionValue| {
        value.concat_utf16().map_err(|error| to_js_error(&error))
    };
    let foreground = foreground.map(concat).transpose()?;
    let background = background.map(concat).transpose()?;
    let font_family = concat(
        options
            .get("fontFamily")
            .map_err(|error| to_js_error(&error))?,
    )?;
    let font_size = HtmlFontSize::Text(concat(
        options
            .get("fontSize")
            .map_err(|error| to_js_error(&error))?,
    )?);
    let ansi_colors = palette
        .map(|colors| {
            colors
                .into_iter()
                .map(|color| color.borrow().css.clone())
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| js_sys::Error::new("Theme must contain 256 ANSI colors"))
        })
        .transpose()?;
    Ok(HtmlConfig {
        foreground,
        background,
        font_family,
        font_size,
        ansi_colors,
    })
}
fn theme_value(
    theme: xterm_core::options::OptionValue,
    key: &str,
) -> Result<Option<xterm_core::options::OptionValue>, JsValue> {
    use xterm_core::options::OptionValue as V;
    if matches!(theme, V::Null | V::Undefined) {
        return Ok(None);
    }
    let value = theme
        .get_property(key)
        .map_err(|error| to_js_error(&error))?;
    Ok((!matches!(value, V::Null | V::Undefined)).then_some(value))
}
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function serialWarn(row,col){console.warn(`Can't get cell at row=${row}, col=${col}`);}"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(catch,js_name=serialWarn)]
    fn warn(row: f64, col: f64) -> Result<(), JsValue>;
}
