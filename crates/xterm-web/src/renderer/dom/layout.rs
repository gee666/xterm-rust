use super::*;
use crate::renderer::selection_model::SelectionRenderModel;
impl DomRenderer {
    pub(super) fn refresh_row_elements(&self, rows: usize) -> Result<(), JsValue> {
        let mut elements = self.rows.borrow_mut();
        let mut blinking = self.row_blink.borrow_mut();
        for _ in elements.len()..=rows {
            let row = element(&self.document, "div", "")?;
            self.row_container.append_child(&row)?;
            elements.push(row);
            blinking.push(false);
        }
        while elements.len() > rows {
            if let Some(row) = elements.pop() {
                row.remove();
            }
            blinking.pop();
        }
        Ok(())
    }
    pub(super) fn update_dimensions(&self, cols: usize, rows: usize) -> Result<(), JsValue> {
        let options = self.options.borrow();
        let dimensions = super::super::geometry::dimensions(
            self.metrics.state.size(),
            self.window.device_pixel_ratio(),
            cols,
            rows,
            options.letter_spacing,
            options.line_height,
        );
        self.dimensions.set(dimensions);
        for row in self.rows.borrow().iter() {
            row.style().set_property(
                "width",
                &format!("{}px", css_number(dimensions.css.canvas.width)),
            )?;
            let height = format!("{}px", css_number(dimensions.css.cell.height));
            row.style().set_property("height", &height)?;
            row.style().set_property("line-height", &height)?;
            row.style().set_property("overflow", "hidden")?;
        }
        self.dimensions_style.set_text_content(Some(&format!(
            ".{} .xterm-rows span {{ display: inline-block; height: 100%; vertical-align: top;}}",
            self.owner_class()
        )));
        self.selection_container.style().set_property(
            "height",
            &self.viewport.style().get_property_value("height")?,
        )?;
        self.screen.style().set_property(
            "width",
            &format!("{}px", css_number(dimensions.css.canvas.width)),
        )?;
        self.screen.style().set_property(
            "height",
            &format!("{}px", css_number(dimensions.css.canvas.height)),
        )?;
        Ok(())
    }
    pub(super) fn set_default_spacing(&self) -> Result<(), JsValue> {
        let spacing = self.dimensions.get().css.cell.width
            - self
                .widths
                .borrow_mut()
                .get(&[u16::from(b'W')], false, false)
                .map_err(width_error)?;
        self.spacing.set(spacing);
        self.row_container
            .style()
            .set_property("letter-spacing", &format!("{}px", css_number(spacing)))
    }
    pub fn handle_resize(&self, cols: usize, rows: usize) -> Result<(), JsValue> {
        self.refresh_row_elements(rows)?;
        self.update_dimensions(cols, rows)?;
        let selection = self.selection_model.get().selection;
        self.handle_selection_changed(selection.start, selection.end, selection.column)
    }
    pub fn handle_char_size_changed(&self) -> Result<(), JsValue> {
        let cols = self
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let rows = self
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.update_dimensions(cols, rows)?;
        self.widths.borrow_mut().clear();
        self.set_default_spacing()
    }
    pub fn handle_device_pixel_ratio_change(&self) -> Result<(), JsValue> {
        self.metrics.measure()?;
        self.handle_char_size_changed()
    }
    pub fn handle_focus(&self) -> Result<(), JsValue> {
        self.focused.set(true);
        self.row_container.class_list().add_1("xterm-focus")?;
        let blink = self
            .cursor_blink
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("DOM renderer has been disposed"))?;
        blink.resume()?;
        let y = self
            .terminal
            .with_state(|state| state.active_buffer().borrow().y)
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.render_rows(y, y)
    }
    pub fn handle_blur(&self) -> Result<(), JsValue> {
        self.focused.set(false);
        self.row_container.class_list().remove_1("xterm-focus")?;
        let blink = self
            .cursor_blink
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("DOM renderer has been disposed"))?;
        blink.pause();
        let rows = self
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.render_rows(0, rows - 1)
    }
    pub fn handle_selection_changed(
        &self,
        start: Option<[i64; 2]>,
        end: Option<[i64; 2]>,
        column: bool,
    ) -> Result<(), JsValue> {
        let selection = Selection { start, end, column };
        *self.selection.borrow_mut() = selection;
        self.selection_container.set_text_content(None);
        let (rows, cols, viewport_y, cursor_y) = self
            .terminal
            .with_state(|state| {
                let active = state.active_buffer();
                let buffer = active.borrow();
                (state.rows(), state.cols(), buffer.ydisp, buffer.y)
            })
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let cols = i64::try_from(cols)
            .map_err(|_| js_sys::RangeError::new("selection columns overflow"))?;
        let old = SelectionRenderModel::update(rows, viewport_y, self.last_selection.get());
        let (Some(start), Some(end)) = (start, end) else {
            return Ok(());
        };
        let new = SelectionRenderModel::update(rows, viewport_y, selection);
        self.selection_model.set(new);
        if new.has_selection {
            let fragment = self.document.create_document_fragment();
            if column {
                let overlay = self.selection_element(
                    new.capped_start,
                    start[0].min(end[0]),
                    start[0].max(end[0]),
                    new.capped_end - new.capped_start + 1,
                )?;
                fragment.append_child(&overlay)?;
            } else {
                let first_start = if new.viewport_start == new.capped_start {
                    start[0]
                } else {
                    0
                };
                let first_end = if new.capped_start == new.viewport_end {
                    end[0]
                } else {
                    cols
                };
                let first = self.selection_element(new.capped_start, first_start, first_end, 1)?;
                fragment.append_child(&first)?;
                let middle = self.selection_element(
                    new.capped_start + 1,
                    0,
                    cols,
                    new.capped_end - new.capped_start - 1,
                )?;
                fragment.append_child(&middle)?;
                if new.capped_start != new.capped_end {
                    let last_end = if new.viewport_end == new.capped_end {
                        end[0]
                    } else {
                        cols
                    };
                    let last = self.selection_element(new.capped_end, 0, last_end, 1)?;
                    fragment.append_child(&last)?;
                }
            }
            self.selection_container.append_child(&fragment)?;
        }
        let old_start = if old.has_selection {
            old.capped_start
        } else {
            0
        };
        let old_end = if old.has_selection {
            old.capped_end
        } else {
            -1
        };
        let new_start = if new.has_selection {
            new.capped_start
        } else {
            0
        };
        let new_end = if new.has_selection {
            new.capped_end
        } else {
            -1
        };
        let mut render_start = old_start.min(new_start).max(0);
        let mut render_end = old_end.max(new_end).min(rows as i128 - 1);
        if render_end >= 0 {
            if new.has_selection && cursor_y < rows {
                render_start = render_start.min(cursor_y as i128);
                render_end = render_end.max(cursor_y as i128);
            }
            self.render_rows(
                usize::try_from(render_start)
                    .map_err(|_| js_sys::RangeError::new("selection row overflow"))?,
                usize::try_from(render_end)
                    .map_err(|_| js_sys::RangeError::new("selection row overflow"))?,
            )?;
        }
        self.last_selection.set(selection);
        Ok(())
    }
    fn selection_element(
        &self,
        row: i128,
        col_start: i64,
        col_end: i64,
        row_count: i128,
    ) -> Result<HtmlElement, JsValue> {
        let element = element(&self.document, "div", "")?;
        let dimensions = self.dimensions.get();
        let left = col_start as f64 * dimensions.css.cell.width;
        let mut width = dimensions.css.cell.width * (col_end as f64 - col_start as f64);
        if left + width > dimensions.css.canvas.width {
            width = dimensions.css.canvas.width - left;
        }
        for (key, value) in [
            ("height", row_count as f64 * dimensions.css.cell.height),
            ("top", row as f64 * dimensions.css.cell.height),
            ("left", left),
            ("width", width),
        ] {
            element
                .style()
                .set_property(key, &format!("{}px", css_number(value)))?;
        }
        Ok(element)
    }
}
