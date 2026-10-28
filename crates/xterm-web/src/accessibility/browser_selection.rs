// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Accessibility DOM selection sorting and cell mapping.
use super::{
    browser::{js_error, Inner},
    selection::normalize_point,
    selection_range, CellPoint,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Node, Text};

// These shims preserve JavaScript structural method dispatch and parseInt
// coercion, including iframe nodes and the original thrown callback value.
#[wasm_bindgen(
    inline_js = "export function accessibility_attribute(element,name){return element.getAttribute(name);} export function accessibility_position(value){return parseInt(value,10)-1;} export function accessibility_text_length(element){return element.textContent?.length??0;} export function accessibility_less(left,right){return left<right;}"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn accessibility_attribute(element: &Node, name: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn accessibility_position(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch)]
    fn accessibility_text_length(element: &Node) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn accessibility_less(left: &JsValue, right: &JsValue) -> Result<bool, JsValue>;
}

pub(super) fn attribute(element: &Node, name: &str) -> Result<JsValue, JsValue> {
    accessibility_attribute(element, name)
}

struct Point {
    node: Node,
    offset: JsValue,
}
impl Inner {
    pub fn selection_change(&self) -> Result<(), JsValue> {
        if self.rows.borrow().is_empty() {
            return Ok(());
        }
        let Some(selection) = self.document.get_selection()? else {
            return Ok(());
        };
        if selection.is_collapsed() {
            if self
                .row_container
                .contains(selection.anchor_node().as_ref())
            {
                self.host.clear_selection()?;
            }
            return Ok(());
        }
        let (Some(anchor), Some(focus)) = (selection.anchor_node(), selection.focus_node()) else {
            web_sys::console::error_1(&"anchorNode and/or focusNode are null".into());
            return Ok(());
        };
        let mut begin = Point {
            node: anchor,
            offset: JsValue::from_f64(f64::from(selection.anchor_offset())),
        };
        let mut end = Point {
            node: focus,
            offset: JsValue::from_f64(f64::from(selection.focus_offset())),
        };
        if begin.node.compare_document_position(&end.node) & Node::DOCUMENT_POSITION_PRECEDING != 0
            || begin.node == end.node && accessibility_less(&end.offset, &begin.offset)?
        {
            std::mem::swap(&mut begin, &mut end);
        }
        let first = self
            .rows
            .borrow()
            .first()
            .cloned()
            .ok_or_else(|| js_sys::TypeError::new("accessibility first row is missing"))?;
        if begin.node.compare_document_position(&first.element)
            & (Node::DOCUMENT_POSITION_CONTAINED_BY | Node::DOCUMENT_POSITION_FOLLOWING)
            != 0
        {
            // Before the first render a row can have no child Text node. Source
            // contains(undefined) is false, so that selection is left untouched.
            let first = self
                .rows
                .borrow()
                .first()
                .cloned()
                .ok_or_else(|| js_sys::TypeError::new("accessibility first row is missing"))?;
            let Some(node) = first.element.child_nodes().item(0) else {
                return Ok(());
            };
            begin = Point {
                node,
                offset: JsValue::from_f64(0.0),
            };
        }
        if !self.row_container.contains(Some(&begin.node)) {
            return Ok(());
        }
        // Selection/DOM methods above can synchronously resize the terminal.
        // Source captures its last row here, not at handler entry.
        let last = self
            .rows
            .borrow()
            .last()
            .cloned()
            .ok_or_else(|| js_sys::TypeError::new("accessibility last row is missing"))?;
        if end.node.compare_document_position(&last.element)
            & (Node::DOCUMENT_POSITION_CONTAINED_BY | Node::DOCUMENT_POSITION_PRECEDING)
            != 0
        {
            end = Point {
                node: last.element.clone().into(),
                offset: accessibility_text_length(last.element.as_ref())?,
            };
        }
        if !self.row_container.contains(Some(&end.node)) {
            return Ok(());
        }
        let (Some(start), Some(end)) = (self.row_column(begin)?, self.row_column(end)?) else {
            return Ok(());
        };
        let range = selection_range(start, end, self.host.cols()?).map_err(js_error)?;
        self.host.select(range.column, range.row, range.length)
    }
    fn row_column(&self, point: Point) -> Result<Option<CellPoint>, JsValue> {
        // Use the module realm's Text instanceof check, as upstream does. A
        // nodeType-only check would silently alter cross-realm source behavior.
        let node = if point.node.is_instance_of::<Text>() {
            point.node.parent_node()
        } else {
            Some(point.node)
        };
        let Some(node) = node else {
            web_sys::console::warn_1(&"row is invalid. Race condition?".into());
            return Ok(None);
        };
        let row = accessibility_position(&attribute(&node, "aria-posinset")?)?;
        if row.is_nan() {
            web_sys::console::warn_1(&"row is invalid. Race condition?".into());
            return Ok(None);
        }
        let columns = self.column_map.get(node.unchecked_ref());
        let Some(columns) = columns.dyn_ref::<js_sys::Array>() else {
            web_sys::console::warn_1(&"columns is null. Race condition?".into());
            return Ok(None);
        };
        // Keep the optional length value raw until the source's comparison and
        // property-index conversions. Nullish text becomes zero; a throwing
        // length getter or conversion preserves its original thrown value.
        let column = if accessibility_less(
            &point.offset,
            &JsValue::from_f64(f64::from(columns.length())),
        )? {
            js_sys::Reflect::get(columns, &point.offset)?
                .as_f64()
                .unwrap_or(f64::NAN)
        } else {
            columns
                .length()
                .checked_sub(1)
                .and_then(|index| columns.get(index).as_f64())
                .unwrap_or(f64::NAN)
                + 1.0
        };
        Ok(normalize_point(row, column, self.host.cols()?))
    }
}
