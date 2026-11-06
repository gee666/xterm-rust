// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Live dimensions are read at the source's DOM assignment boundaries.
use super::browser::{Inner, TreeRow};
use crate::scroll::fast_dom_node::{assign_style, begin_style};
use std::rc::Rc;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::HtmlElement;

// Object.assign supplies only the source DOM operation, including native errors.
// Rust resolves its receiver and both values first and owns all row traversal.
#[wasm_bindgen(
    inline_js = "export function accessibilityAssignDimensions(style,width,fontSize){Object.assign(style,{width,fontSize});}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = accessibilityAssignDimensions)]
    fn assign_dimensions(style: &JsValue, width: &str, font_size: &str) -> Result<(), JsValue>;
}
impl Inner {
    pub fn refresh_dimensions(self: &Rc<Self>) -> Result<(), JsValue> {
        let height = self.host.cell_height()?;
        if height == 0.0 || height.is_nan() {
            return Ok(());
        }
        let style = begin_style(&self.container)?;
        let width = format!("{}px", number(self.host.canvas_width()?));
        let font_size = format!("{}px", number(self.host.font_size()?));
        assign_dimensions(&style, &width, &font_size)?;
        let row_count = self.rows.borrow().len();
        if row_count != self.host.rows()? {
            self.resize_rows(self.host.rows()?)?;
        }
        let mut index = 0;
        while index < self.host.rows()? {
            let row = self.dimension_row(index)?;
            self.refresh_row_dimensions(&row.element)?;
            // A height setter can resize/replace the row array. Source selects
            // the alignment receiver only after that setter has returned.
            self.align_width(&self.dimension_row(index)?)?;
            index += 1;
        }
        Ok(())
    }
    fn dimension_row(&self, index: usize) -> Result<Rc<TreeRow>, JsValue> {
        self.rows
            .borrow()
            .get(index)
            .cloned()
            .ok_or_else(|| js_sys::Error::new("accessibility row is missing").into())
    }
    pub(super) fn refresh_row_dimensions(&self, row: &HtmlElement) -> Result<(), JsValue> {
        let style = begin_style(row)?;
        let height = format!("{}px", number(self.host.cell_height()?));
        assign_style(&style, "height", &height.into())
    }
    pub(super) fn align_width(&self, row: &TreeRow) -> Result<(), JsValue> {
        assign_style(&begin_style(&row.element)?, "transform", &"".into())?;
        let width = row.element.get_bounding_client_rect().width();
        let mapping = self.column_map.get(row.element.unchecked_ref());
        let last_column = mapping.dyn_ref::<js_sys::Array>().and_then(|columns| {
            columns
                .length()
                .checked_sub(1)
                .and_then(|index| columns.get(index).as_f64())
        });
        if let Some(last_column) = last_column.filter(|last| *last != 0.0 && !last.is_nan()) {
            let target = last_column * self.host.cell_width()?;
            let transform = format!("scaleX({})", number(target / width));
            assign_style(&begin_style(&row.element)?, "transform", &transform.into())?;
        }
        Ok(())
    }
}
fn number(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else if value == f64::INFINITY {
        "Infinity".into()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".into()
    } else if value == 0.0 {
        "0".into()
    } else {
        value.to_string()
    }
}
