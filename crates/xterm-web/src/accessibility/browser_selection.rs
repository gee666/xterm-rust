// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Accessibility DOM selection sorting and cell mapping.
use super::{
    browser::{get_text, js_error, Inner},
    map_point, selection_range, CellPoint,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Node, Text};

// These shims preserve JavaScript structural method dispatch and parseInt
// coercion, including iframe nodes and the original thrown callback value.
#[wasm_bindgen(
    inline_js = "export function accessibility_attribute(element,name){return element.getAttribute(name);} export function accessibility_position(value){return parseInt(value,10)-1;}"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn accessibility_attribute(element: &Node, name: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn accessibility_position(value: &JsValue) -> Result<f64, JsValue>;
}

pub(super) fn attribute(element: &Node, name: &str) -> Result<JsValue, JsValue> {
    accessibility_attribute(element, name)
}

struct Point {
    node: Node,
    offset: usize,
}
impl Inner {
    pub fn selection_change(&self) -> Result<(), JsValue> {
        let (first, last) = {
            let rows = self.rows.borrow();
            (rows.first().cloned(), rows.last().cloned())
        };
        let (Some(first), Some(last)) = (first, last) else {
            return Ok(());
        };
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
            offset: selection.anchor_offset() as usize,
        };
        let mut end = Point {
            node: focus,
            offset: selection.focus_offset() as usize,
        };
        if begin.node.compare_document_position(&end.node) & Node::DOCUMENT_POSITION_PRECEDING != 0
            || begin.node.is_same_node(Some(&end.node)) && begin.offset > end.offset
        {
            std::mem::swap(&mut begin, &mut end);
        }
        if begin.node.compare_document_position(&first.element)
            & (Node::DOCUMENT_POSITION_CONTAINED_BY | Node::DOCUMENT_POSITION_FOLLOWING)
            != 0
        {
            // Before the first render a row can have no child Text node. Source
            // contains(undefined) is false, so that selection is left untouched.
            let Some(node) = first.element.child_nodes().item(0) else {
                return Ok(());
            };
            begin = Point { node, offset: 0 };
        }
        if !self.row_container.contains(Some(&begin.node)) {
            return Ok(());
        }
        if end.node.compare_document_position(&last.element)
            & (Node::DOCUMENT_POSITION_CONTAINED_BY | Node::DOCUMENT_POSITION_PRECEDING)
            != 0
        {
            end = Point {
                node: last.element.clone().into(),
                offset: get_text(&last.element)?.len(),
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
        let mut mapping = Vec::new();
        mapping
            .try_reserve(columns.length() as usize)
            .map_err(js_error)?;
        for value in columns.iter() {
            let column = value
                .as_f64()
                .ok_or_else(|| js_sys::TypeError::new("accessibility column is not numeric"))?;
            // Column maps originate from Rust buffer cell indices in this WASM
            // instance, so all entries fit usize and are exact JS integers.
            if !column.is_finite()
                || column < 0.0
                || column.fract() != 0.0
                || column > usize::MAX as f64
            {
                return Err(js_sys::RangeError::new(
                    "accessibility column is outside cell-index bounds",
                )
                .into());
            }
            mapping.push(column as usize);
        }
        Ok(map_point(row, point.offset, &mapping, self.host.cols()?))
    }
}
