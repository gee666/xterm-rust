#![cfg(target_arch = "wasm32")]
// Original RowFactory assertions execute in tools/capture_row_factory.mjs.
// Replay every captured call through Rust planning and real browser DOM nodes.
use serde_json::Value;
use std::convert::Infallible;
use wasm_bindgen_test::*;
use xterm_core::color::Color;
#[path = "support/cells.rs"]
mod cells;
use cells::number;
use xterm_web::{
    renderer::{
        row::{RowContext, RowFactory},
        row_dom::create_spans,
        row_types::{RowOptions, Selection},
        width_cache::{Font, FontVariant, WidthCache, WidthMeasurer},
    },
    theme::{ColorRef, ThemeService},
};
wasm_bindgen_test_configure!(run_in_browser);

struct FixtureMeasurer(Value);
impl WidthMeasurer for FixtureMeasurer {
    type Font = Font;
    type Error = Infallible;
    fn configure_variant(&self, _: &Font, _: FontVariant) -> Result<(), Infallible> {
        Ok(())
    }
    fn measure(&self, text: &[u16], variant: FontVariant) -> Result<f64, Infallible> {
        let text = String::from_utf16(text).unwrap();
        Ok(self.0[variant as usize][text].as_f64().unwrap_or(5.0))
    }
}
fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}
fn color(value: &Value) -> ColorRef {
    Color {
        css: value["css"].as_str().unwrap().into(),
        rgba: value["rgba"].as_u64().unwrap() as u32,
    }
    .into()
}
fn position(value: &Value) -> Option<[i64; 2]> {
    value
        .as_array()
        .map(|a| [a[0].as_i64().unwrap(), a[1].as_i64().unwrap()])
}

#[wasm_bindgen_test]
fn replay_every_original_row_assertion_in_real_browser_dom() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/row-factory.json")).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let cases = fixture["calls"].as_array().unwrap();
    assert!(!cases.is_empty());
    for (index, case) in cases.iter().enumerate() {
        let line = cells::line(case["cells"].as_array().unwrap());
        let theme = ThemeService::default();
        let colors = theme.colors();
        {
            let mut colors = colors.borrow_mut();
            let raw = &case["colors"];
            colors.foreground = color(&raw["foreground"]);
            colors.background = color(&raw["background"]);
            colors.selection_background_opaque = color(&raw["selectionBackgroundOpaque"]);
            colors.selection_inactive_background_opaque =
                color(&raw["selectionInactiveBackgroundOpaque"]);
            colors.selection_foreground = raw.get("selectionForeground").map(color);
            colors.ansi = raw["ansi"].as_array().unwrap().iter().map(color).collect();
        }
        let start = case["linkStart"].as_i64().unwrap();
        let end = case["linkEnd"].as_i64().unwrap();
        let options = RowOptions {
            row: number(&case["row"]),
            cursor_row: flag(&case["cursorRow"]),
            cursor_x: number(&case["cursorX"]),
            cursor_style: case["cursorStyle"].as_str().unwrap().into(),
            cursor_inactive_style: case["inactiveStyle"].as_str().unwrap().into(),
            cursor_blink: flag(&case["cursorBlink"]),
            cursor_hidden: flag(&case["cursorHidden"]),
            cursor_initialized: flag(&case["cursorInitialized"]),
            focused: flag(&case["focused"]),
            blink_on: flag(&case["blinkOn"]),
            cell_width: case["cellWidth"].as_f64().unwrap(),
            default_spacing: case["defaultSpacing"].as_f64().unwrap(),
            link_range: if start == -1 || end == -1 {
                None
            } else {
                Some([start, end])
            },
            bold_bright: flag(&case["boldBright"]),
            minimum_contrast_ratio: case["contrast"].as_f64().unwrap_or(1.0),
        };
        let factory = RowFactory {
            selection: Selection {
                start: position(&case["selectionStart"]),
                end: position(&case["selectionEnd"]),
                column: flag(&case["column"]),
            },
        };
        let widths = WidthCache::new(FixtureMeasurer(case["widths"].clone()));
        let colors = colors.borrow();
        let row = factory
            .create_row(
                &line,
                RowContext {
                    options: &options,
                    colors: &colors,
                    joined_ranges: &[],
                    decorations: &|_, _| Ok(Vec::new()),
                },
                &widths,
            )
            .unwrap();
        let container = document.create_element("div").unwrap();
        for span in create_spans(&document, &row.spans).unwrap() {
            container.append_child(&span).unwrap();
        }
        assert_eq!(
            container.inner_html(),
            case["html"].as_str().unwrap(),
            "upstream captured call {index}"
        );
    }
}
