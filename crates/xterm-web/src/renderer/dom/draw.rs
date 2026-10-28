use super::*;
use crate::renderer::{
    row::RowContext,
    row_dom::create_spans,
    row_types::{RowDecoration, RowError, RowOptions},
    width_cache::WidthError,
};
use xterm_core::{core_service::CursorStyle, decoration::DecorationLayer};
struct LinkPaint {
    enabled: bool,
    start: usize,
    end: usize,
    x: i64,
    x2: i64,
    cols: usize,
}
impl DomRenderer {
    pub fn render_rows(&self, start: usize, end: usize) -> Result<(), JsValue> {
        self.render_rows_inner(start, end, None)
    }
    /// JavaScript loop semantics survive NaN/fractional refresh ranges. The
    /// service clamps upper bounds; undefined negative row slots have no effect.
    pub fn render_rows_js(&self, start: f64, end: f64) -> Result<(), JsValue> {
        if start.is_nan() || end.is_nan() || start.fract() != 0.0 || end < start || end < 0.0 {
            return self.update_text_blink();
        }
        if !start.is_finite() || !end.is_finite() {
            return Err(js_sys::RangeError::new(
                "renderer row range must be finite after service clamping",
            )
            .into());
        }
        let count = self.rows.borrow().len();
        if count == 0 || start >= count as f64 {
            return self.update_text_blink();
        }
        self.render_rows_inner(
            start.max(0.0) as usize,
            end.floor().min((count - 1) as f64) as usize,
            None,
        )
    }
    pub fn handle_link_underline(
        &self,
        enabled: bool,
        event: crate::links::UnderlineEvent,
    ) -> Result<(), JsValue> {
        let rows = self
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let cols = self
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let max = i64::try_from(rows - 1)
            .map_err(|_| js_sys::RangeError::new("link viewport row overflow"))?;
        let start = event.y1.clamp(0, max) as usize;
        let end = event.y2.clamp(0, max) as usize;
        let x = if event.y1 < 0 { 0 } else { event.x1 };
        let x2 = if event.y2 < 0 {
            0
        } else {
            i64::try_from(event.x2).map_err(|_| js_sys::RangeError::new("link column overflow"))?
        };
        self.render_rows_inner(
            start,
            end,
            Some(LinkPaint {
                enabled,
                start,
                end,
                x,
                x2,
                cols: event.cols.min(cols),
            }),
        )
    }
    fn render_rows_inner(
        &self,
        start: usize,
        end: usize,
        link: Option<LinkPaint>,
    ) -> Result<(), JsValue> {
        if self.disposed.get() {
            return Err(js_sys::Error::new("DOM renderer has been disposed").into());
        }
        let (buffer, core) = self
            .terminal
            .with_state(|state| (state.active_buffer(), Rc::clone(&state.core)))
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let cols = self
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let (cursor_y, cursor_x, display) = {
            let buffer = buffer.borrow();
            (
                buffer.ybase + buffer.y,
                buffer.x.min(cols - 1),
                buffer.ydisp,
            )
        };
        let options = self.options.borrow().clone();
        let modes = core.dec_private_modes.borrow().clone();
        for y in start..=end {
            let Some(element) = self.rows.borrow().get(y).cloned() else {
                continue;
            };
            let absolute = y + display;
            let line = buffer.borrow().lines.get(absolute);
            let Some(line) = line else {
                replace_children(&element, &[])?;
                self.row_blink.borrow_mut()[y] = false;
                continue;
            };
            // User joiners run before borrowing theme, width caches or row data.
            let joins = self.joiners.get_joined_characters(Some(Rc::clone(&line)));
            let row_options = RowOptions {
                row: absolute,
                cursor_row: absolute == cursor_y && link.as_ref().is_none_or(|link| link.cols > 0),
                cursor_x: link
                    .as_ref()
                    .map_or(cursor_x, |link| cursor_x.min(link.cols.saturating_sub(1))),
                cursor_style: match if link.is_some() {
                    None
                } else {
                    modes.cursor_style
                } {
                    Some(CursorStyle::Block) => "block".into(),
                    Some(CursorStyle::Bar) => "bar".into(),
                    Some(CursorStyle::Underline) => "underline".into(),
                    None => options.cursor_style.clone(),
                },
                cursor_inactive_style: options.cursor_inactive_style.clone(),
                cursor_blink: if link.is_some() {
                    options.cursor_blink
                } else {
                    modes.cursor_blink.unwrap_or(options.cursor_blink)
                },
                cursor_hidden: core.is_cursor_hidden.get(),
                cursor_initialized: core.is_cursor_initialized.get(),
                focused: self.browser_focused()?,
                blink_on: self.is_text_blink_on(),
                cell_width: self.dimensions.get().css.cell.width,
                default_spacing: self.spacing.get(),
                link_range: link.as_ref().filter(|link| link.enabled).and_then(|link| {
                    let left = if y == link.start { link.x } else { 0 };
                    let right = if y == link.end {
                        link.x2
                    } else {
                        link.cols as i64
                    } - 1;
                    if left == -1 || right == -1 {
                        None
                    } else {
                        Some([left, right])
                    }
                }),
                bold_bright: options.bold_bright,
                minimum_contrast_ratio: options.minimum_contrast_ratio,
            };
            let colors = self.theme.colors();
            let colors = colors.borrow();
            let factory = RowFactory {
                selection: *self.selection.borrow(),
            };
            let decorations = |x: usize, row: usize| -> Result<Vec<RowDecoration>, String> {
                let row = i64::try_from(row)
                    .map_err(|_| "decoration row exceeds signed bounds".to_owned())?;
                self.decoration
                    .get_decorations_at_cell(x as f64, row, None)
                    .map(|decoration| {
                        let fallback = |input: &str| {
                            self.theme
                                .parse_color(input)
                                .ok()
                                .map(|color| color.rgba.to_be_bytes())
                        };
                        let background = decoration
                            .background_color(fallback)
                            .map_err(|e| e.to_string())?
                            .map(|color| color.borrow().clone());
                        let foreground = decoration
                            .foreground_color(fallback)
                            .map_err(|e| e.to_string())?
                            .map(|color| color.borrow().clone());
                        let top = decoration.options().borrow().layer == Some(DecorationLayer::Top);
                        Ok(RowDecoration {
                            foreground,
                            background,
                            top,
                        })
                    })
                    .collect()
            };
            let rendered = factory
                .create_row(
                    &line.borrow(),
                    RowContext {
                        options: &row_options,
                        colors: &colors,
                        joined_ranges: &joins,
                        decorations: &decorations,
                    },
                    self.widths()?,
                )
                .map_err(row_error)?;
            let spans = create_spans(&self.document, &rendered.spans)?;
            replace_children(&element, &spans)?;
            self.row_blink.borrow_mut()[y] = rendered.has_blinking_cells;
        }
        self.update_text_blink()
    }
}
fn replace_children(element: &HtmlElement, children: &[HtmlElement]) -> Result<(), JsValue> {
    let nodes = js_sys::Array::new();
    for node in children {
        nodes.push(node.as_ref());
    }
    let replace: js_sys::Function =
        js_sys::Reflect::get(element.as_ref(), &JsValue::from_str("replaceChildren"))?
            .dyn_into()?;
    replace.apply(element.as_ref(), &nodes)?;
    Ok(())
}
fn row_error(error: RowError<JsValue>) -> JsValue {
    match error {
        RowError::Width(WidthError::Measure(error)) => error,
        RowError::Width(WidthError::Disposed) => {
            js_sys::TypeError::new("width measurement canvas has been disposed").into()
        }
        RowError::Palette(index) => {
            js_sys::TypeError::new(&format!("palette index {index} is missing")).into()
        }
        RowError::JoinRange(range) => {
            js_sys::RangeError::new(&format!("invalid join range {range:?}")).into()
        }
        RowError::Decoration(error) => js_sys::Error::new(&error).into(),
    }
}
