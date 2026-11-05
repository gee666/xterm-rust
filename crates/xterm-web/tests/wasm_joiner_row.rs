#![cfg(target_arch = "wasm32")]
use js_sys::{Function, JsString, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_core::{
    buffer::{AttributeData, BufferLine},
    color::Color,
};
use xterm_web::{
    character_joiner::{raw::RawRanges, LineRef},
    renderer::{
        row::{RowContext, RowFactory},
        row_dom::create_spans,
        row_types::{RowError, RowOptions, Selection},
        width_cache::{Font, FontVariant, WidthCache, WidthMeasurer},
    },
    theme::ThemeService,
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/joiner_row.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = joinerRowScenarios)]
    fn scenarios(create: &Function) -> Result<JsString, JsValue>;
}
struct FixedWidth;
impl WidthMeasurer for FixedWidth {
    type Font = Font;
    type Error = JsValue;
    fn configure_variant(&self, _: &Font, _: FontVariant) -> Result<(), JsValue> {
        Ok(())
    }
    fn measure(&self, _: &[u16], _: FontVariant) -> Result<f64, JsValue> {
        Ok(5.0)
    }
}
#[wasm_bindgen]
pub struct RawRowProbe {
    line: LineRef,
    work_cell: RefCell<xterm_core::buffer::CellData>,
    selection: RefCell<Selection>,
}
#[wasm_bindgen]
impl RawRowProbe {
    pub fn set(&self, index: usize, cp: u32, width: i32) {
        self.line
            .borrow_mut()
            .set_cell_from_codepoint(index, cp, width, &AttributeData::default());
    }
    pub fn selection(&self, start: f64, end: f64) {
        *self.selection.borrow_mut() = Selection {
            start: Some([start as i64, 0]),
            end: Some([end as i64, 0]),
            column: false,
        };
    }
    pub fn render(&self, ranges: JsValue, raw: JsValue) -> Result<String, JsValue> {
        let field = |key: &str| Reflect::get(&raw, &key.into());
        let selected = field("selection")?;
        if !selected.is_undefined() {
            self.selection(
                Reflect::get(&selected, &0.into())?.as_f64().unwrap(),
                Reflect::get(&selected, &1.into())?.as_f64().unwrap(),
            );
            self.selection.borrow_mut().column = field("column")?.is_truthy();
        }
        let options = RowOptions {
            cursor_row: field("cursor")?.is_truthy(),
            cursor_x: field("x")?.as_f64().unwrap_or(0.0) as usize,
            cell_width: 5.0,
            ..RowOptions::default()
        };
        let theme = ThemeService::default();
        let colors = theme.colors();
        let mut colors = colors.borrow_mut();
        let white: xterm_web::theme::ColorRef = Color {
            css: "#ffffff".into(),
            rgba: 0xffffffff,
        }
        .into();
        colors.selection_background_opaque = white.clone();
        colors.selection_inactive_background_opaque = white;
        let row = RowFactory::default()
            .create_row_raw(
                &self.line,
                &self.work_cell,
                RawRanges(ranges),
                &|| *self.selection.borrow(),
                RowContext {
                    options: &options,
                    colors: &colors,
                    joined_ranges: &[],
                    decorations: &|_, _| Ok(Vec::new()),
                },
                &WidthCache::new(FixedWidth),
            )
            .map_err(|error| match error {
                RowError::Browser(error) => error,
                other => js_sys::Error::new(&format!("{other:?}")).into(),
            })?;
        let document = web_sys::window().unwrap().document().unwrap();
        let container = document.create_element("div")?;
        for span in create_spans(&document, &row.spans)? {
            container.append_child(&span)?;
        }
        Ok(container.inner_html())
    }
}
fn probe() -> RawRowProbe {
    let probe = RawRowProbe {
        line: Rc::new(RefCell::new(BufferLine::new(6, None, false))),
        selection: RefCell::default(),
        work_cell: RefCell::default(),
    };
    for (i, cp, w) in [
        (0, 97, 1),
        (1, 0x4e2d, 2),
        (2, 0, 0),
        (3, 98, 1),
        (4, 99, 1),
        (5, 100, 1),
    ] {
        probe.set(i, cp, w);
    }
    probe
}
#[wasm_bindgen_test]
fn raw_row_getters_arithmetic_mutation_and_errors_match_unchanged_source() {
    let create =
        Closure::wrap(Box::new(|| -> JsValue { probe().into() }) as Box<dyn Fn() -> JsValue>);
    let actual = scenarios(create.as_ref().unchecked_ref()).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&String::from(actual)).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/joiner-row.json")).unwrap();
    for (name, expected) in expected.as_object().unwrap() {
        assert_eq!(&actual[name], expected, "{name}");
    }
    assert_eq!(
        actual.as_object().unwrap().len(),
        expected.as_object().unwrap().len()
    );
}

#[wasm_bindgen_test]
fn nonprogressing_source_ranges_return_explicit_safety_errors() {
    let options = js_sys::Object::new();
    let ranges = js_sys::Array::new();
    let range = js_sys::Array::new();
    range.push(&1.into());
    range.push(&f64::INFINITY.into());
    ranges.push(&range);
    let error = probe()
        .render(ranges.into(), options.clone().into())
        .unwrap_err();
    assert!(error.is_instance_of::<js_sys::RangeError>());
    let ranges = js_sys::Array::new();
    let range = js_sys::Array::new();
    range.push(&1.into());
    range.push(&f64::NEG_INFINITY.into());
    ranges.push(&range);
    assert!(probe().render(ranges.into(), options.into()).is_err());
}
