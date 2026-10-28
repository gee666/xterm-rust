// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Accessibility row rendering, focus rotation and dimensions.
use super::browser::{set_text, Inner, TreeRow};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{FocusEvent, HtmlElement};

#[derive(Clone, Copy)]
enum Boundary {
    Top,
    Bottom,
}
impl Inner {
    pub fn create_row(&self) -> Result<Rc<TreeRow>, JsValue> {
        // Known-tag construction must not reject another document's realm.
        let element: HtmlElement = self.document.create_element("div")?.unchecked_into();
        element.set_attribute("role", "listitem")?;
        element.set_tab_index(-1);
        element.style().set_property(
            "height",
            &format!("{}px", number(self.host.dimensions()?.cell_height)),
        )?;
        Ok(Rc::new(TreeRow { element }))
    }
    pub fn render_rows(&self, start: f64, end: f64) -> Result<(), JsValue> {
        let buffer = Rc::clone(&self.host).render_buffer()?;
        let set_size = buffer.line_count()?.to_string();
        if start.is_finite()
            && end.is_finite()
            && start >= 0.0
            && start.fract() == 0.0
            && end >= start
        {
            let mut index = start;
            while index <= end {
                let offset = buffer.display_offset()?;
                let absolute = offset
                    .checked_add(index as usize)
                    .ok_or_else(|| js_sys::RangeError::new("accessibility row overflow"))?;
                let data = buffer.row(absolute)?;
                let position = absolute
                    .checked_add(1)
                    .ok_or_else(|| js_sys::RangeError::new("accessibility position overflow"))?
                    .to_string();
                let row = self.rows.borrow().get(index as usize).cloned();
                if let Some(row) = row {
                    let (text, columns) = match data {
                        Some(data) if !data.text.is_empty() => (data.text, data.columns),
                        _ => (vec![0xa0], vec![0, 1]),
                    };
                    set_text(&row.element, &text)?;
                    let mapping = js_sys::Array::new();
                    for column in columns {
                        mapping.push(&JsValue::from_f64(column as f64));
                    }
                    self.column_map.set(row.element.unchecked_ref(), &mapping);
                    row.element.set_attribute("aria-posinset", &position)?;
                    row.element.set_attribute("aria-setsize", &set_size)?;
                    self.align_width(&row)?;
                }
                index += 1.0;
            }
        }
        self.announce()
    }
    pub fn refresh_dimensions(self: &Rc<Self>) -> Result<(), JsValue> {
        let dimensions = self.host.dimensions()?;
        // JavaScript truthiness skips zero and NaN, not negative values.
        if dimensions.cell_height == 0.0 || dimensions.cell_height.is_nan() {
            return Ok(());
        }
        self.container
            .style()
            .set_property("width", &format!("{}px", number(dimensions.canvas_width)))?;
        self.container
            .style()
            .set_property("font-size", &format!("{}px", number(dimensions.font_size)))?;
        let count = self.host.rows()?;
        if self.rows.borrow().len() != count {
            self.resize_rows(count)?;
        }
        for index in 0..count {
            let row = self
                .rows
                .borrow()
                .get(index)
                .cloned()
                .ok_or_else(|| js_sys::Error::new("accessibility row is missing"))?;
            row.element
                .style()
                .set_property("height", &format!("{}px", number(dimensions.cell_height)))?;
            self.align_width(&row)?;
        }
        Ok(())
    }
    fn align_width(&self, row: &TreeRow) -> Result<(), JsValue> {
        row.element.style().set_property("transform", "")?;
        let width = row.element.get_bounding_client_rect().width();
        let mapping = self.column_map.get(row.element.unchecked_ref());
        let last_column = mapping.dyn_ref::<js_sys::Array>().and_then(|columns| {
            columns
                .length()
                .checked_sub(1)
                .and_then(|index| columns.get(index).as_f64())
        });
        if let Some(last_column) = last_column.filter(|last| *last != 0.0 && !last.is_nan()) {
            let target = last_column * self.host.dimensions()?.cell_width;
            row.element
                .style()
                .set_property("transform", &format!("scaleX({})", number(target / width)))?;
        }
        Ok(())
    }
    pub fn detach_boundaries(&self) -> Result<(), JsValue> {
        let (top, bottom) = {
            let rows = self.rows.borrow();
            (rows.first().cloned(), rows.last().cloned())
        };
        if let (Some(top), Some(bottom)) = (top, bottom) {
            self.remove_boundary_listeners(&top.element, &bottom.element)?;
        }
        Ok(())
    }
    fn remove_boundary_listeners(
        &self,
        top: &HtmlElement,
        bottom: &HtmlElement,
    ) -> Result<(), JsValue> {
        let (top_callback, bottom_callback) = self.boundary_callbacks()?;
        top.remove_event_listener_with_callback("focus", top_callback)?;
        bottom.remove_event_listener_with_callback("focus", bottom_callback)
    }
    fn boundary_callback(self: &Rc<Self>, position: Boundary) -> js_sys::Function {
        let weak = Rc::downgrade(self);
        // JS owns the callback as in DomListener. Only a Weak reference points
        // back to the manager, so neither retained rows nor errors form cycles.
        Closure::<dyn Fn(web_sys::Event) -> Result<(), JsValue>>::new(
            move |event: web_sys::Event| {
                if let Some(inner) = weak.upgrade() {
                    inner.boundary_focus(event.unchecked_into(), position)
                } else {
                    Ok(())
                }
            },
        )
        .into_js_value()
        .unchecked_into()
    }
    pub fn initialize_boundaries(self: &Rc<Self>) -> Result<(), JsValue> {
        self.boundaries
            .set((
                self.boundary_callback(Boundary::Top),
                self.boundary_callback(Boundary::Bottom),
            ))
            .map_err(|_| {
                js_sys::Error::new("accessibility boundary callbacks already initialized").into()
            })
    }
    fn boundary_callbacks(&self) -> Result<&(js_sys::Function, js_sys::Function), JsValue> {
        self.boundaries.get().ok_or_else(|| {
            js_sys::Error::new("accessibility boundary callbacks are missing").into()
        })
    }
    pub fn attach_boundaries(self: &Rc<Self>) -> Result<(), JsValue> {
        let (top_callback, bottom_callback) = self.boundary_callbacks()?;
        let top = self
            .rows
            .borrow()
            .first()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("accessibility top row is missing"))?;
        top.element
            .add_event_listener_with_callback("focus", top_callback)?;
        let bottom = self
            .rows
            .borrow()
            .last()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("accessibility bottom row is missing"))?;
        bottom
            .element
            .add_event_listener_with_callback("focus", bottom_callback)
    }
    fn boundary_focus(
        self: &Rc<Self>,
        event: FocusEvent,
        position: Boundary,
    ) -> Result<(), JsValue> {
        let target: HtmlElement = event
            .target()
            .ok_or_else(|| js_sys::Error::new("focus target is missing"))?
            .unchecked_into();
        let adjacent = {
            let rows = self.rows.borrow();
            let index = match position {
                Boundary::Top => Some(1),
                Boundary::Bottom => rows.len().checked_sub(2),
            };
            index.and_then(|index| rows.get(index).cloned())
        };
        let pos_in_set = super::browser_selection::attribute(&target, "aria-posinset")?;
        let limit = match position {
            Boundary::Top => "1".into(),
            Boundary::Bottom => self.host.buffer_length()?.to_string(),
        };
        if pos_in_set == JsValue::from_str(&limit) {
            return Ok(());
        }
        let Some(adjacent) = adjacent else {
            return Ok(());
        };
        let adjacent_target: &web_sys::EventTarget = adjacent.element.as_ref();
        if event.related_target().as_ref() != Some(adjacent_target) {
            return Ok(());
        }
        let removed = match position {
            Boundary::Top => self.rows.borrow_mut().pop(),
            Boundary::Bottom => {
                if self.rows.borrow().is_empty() {
                    None
                } else {
                    Some(self.rows.borrow_mut().remove(0))
                }
            }
        }
        .ok_or_else(|| js_sys::Error::new("accessibility boundary row is missing"))?;
        self.row_container.remove_child(&removed.element)?;
        match position {
            Boundary::Top => self.remove_boundary_listeners(&target, &removed.element)?,
            Boundary::Bottom => self.remove_boundary_listeners(&removed.element, &target)?,
        }
        let row = self.create_row()?;
        match position {
            Boundary::Top => {
                self.rows.borrow_mut().insert(0, Rc::clone(&row));
                self.row_container
                    .insert_before(&row.element, self.row_container.first_child().as_ref())?;
            }
            Boundary::Bottom => {
                self.rows.borrow_mut().push(Rc::clone(&row));
                self.row_container.append_child(&row.element)?;
            }
        }
        self.attach_boundaries()?;
        self.host.scroll_lines(match position {
            Boundary::Top => -1,
            Boundary::Bottom => 1,
        })?;
        let focus = {
            let rows = self.rows.borrow();
            let index = match position {
                Boundary::Top => Some(1),
                Boundary::Bottom => rows.len().checked_sub(2),
            };
            index.and_then(|index| rows.get(index).cloned())
        }
        .ok_or_else(|| js_sys::Error::new("accessibility focus row is missing"))?;
        focus.element.focus()?;
        event.prevent_default();
        event.stop_immediate_propagation();
        Ok(())
    }
    pub fn resize_rows(self: &Rc<Self>, count: usize) -> Result<(), JsValue> {
        if count == 0 {
            return Err(
                js_sys::Error::new("accessibility requires at least one terminal row").into(),
            );
        }
        let (_, bottom_callback) = self.boundary_callbacks()?;
        let bottom = self
            .rows
            .borrow()
            .last()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("accessibility bottom row is missing"))?;
        // Source removes only the bottom listener. The reusable callbacks stay
        // owned even if row creation/removal fails, allowing a later resize.
        bottom
            .element
            .remove_event_listener_with_callback("focus", bottom_callback)?;
        let old_count = self.row_container.children().length() as usize;
        let mut index = old_count;
        while index < self.host.rows()? {
            let row = self.create_row()?;
            {
                let mut rows = self.rows.borrow_mut();
                if index == rows.len() {
                    rows.push(Rc::clone(&row));
                } else if let Some(slot) = rows.get_mut(index) {
                    *slot = Rc::clone(&row);
                } else {
                    return Err(js_sys::Error::new("accessibility row tree has sparse gaps").into());
                }
            }
            self.row_container.append_child(&row.element)?;
            index += 1;
        }
        while self.rows.borrow().len() > count {
            let row = self
                .rows
                .borrow_mut()
                .pop()
                .ok_or_else(|| js_sys::Error::new("accessibility row is missing"))?;
            self.row_container.remove_child(&row.element)?;
        }
        let row = self
            .rows
            .borrow()
            .last()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("accessibility bottom row is missing"))?;
        row.element
            .add_event_listener_with_callback("focus", bottom_callback)?;
        self.refresh_dimensions()
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
