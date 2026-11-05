// Copyright (c) 2018, 2023 The xterm.js authors. All rights reserved. MIT license.
use super::row_source::{RowSource, TypedRow};
use super::{
    row_colors,
    row_types::*,
    width_cache::{WidthCache, WidthMeasurer},
};
use crate::theme::ColorSet;
use xterm_core::buffer::{AttributeData, BufferLine};

pub struct RowContext<'a> {
    pub options: &'a RowOptions,
    pub colors: &'a ColorSet,
    pub joined_ranges: &'a [[usize; 2]],
    pub decorations: &'a dyn Fn(f64, usize) -> Result<Vec<RowDecoration>, String>,
}
#[derive(Default, Debug)]
pub struct RenderedRow {
    pub spans: Vec<RowSpan>,
    pub has_blinking_cells: bool,
}
#[derive(Default)]
pub struct RowFactory {
    pub selection: Selection,
}
struct MergeState {
    selected: bool,
    bg: u32,
    fg: u32,
    extended: u32,
    link: bool,
    spacing: f64,
}
impl RowFactory {
    pub fn create_row<M: WidthMeasurer>(
        &self,
        line: &BufferLine,
        context: RowContext<'_>,
        widths: &WidthCache<M>,
    ) -> Result<RenderedRow, RowError<M::Error>> {
        let source = TypedRow::new(line, &context, self.selection);
        self.plan_row(source, context, widths)
    }
    pub(super) fn plan_row<M: WidthMeasurer>(
        &self,
        mut source: impl RowSource,
        context: RowContext<'_>,
        widths: &WidthCache<M>,
    ) -> Result<RenderedRow, RowError<M::Error>> {
        let options = context.options;
        let colors = context.colors;
        let mut result = RenderedRow::default();
        let mut length = source.length();
        if options.cursor_row {
            length = length.max(options.cursor_x.saturating_add(1));
        }
        let mut x = 0.0;
        let mut cursor_x = options.cursor_x as f64;
        let mut old = None::<MergeState>;
        let mut mergeable = false;
        let mut classes = Vec::<String>::new();
        while x < length as f64 {
            if !x.is_finite() || x + 1.0 == x {
                return Err(RowError::NonProgressingColumn(x));
            }
            let mut cell = source.cell(x);
            if cell.width == 0.0 {
                x += 1.0;
                continue;
            }
            let mut valid_join = source.valid(x)?;
            let joined_end = source.join(x, &mut cell, &mut valid_join, options, cursor_x)?;
            let joined = joined_end.is_some();
            let last_x = joined_end.unwrap_or(x);
            let selected = source.selected(x, options.row);
            let cursor = options.cursor_row && x == cursor_x;
            let link = options
                .link_range
                .is_some_and(|range| x >= range[0] as f64 && x <= range[1] as f64);
            if cell.attrs.is_blink() != 0 {
                result.has_blinking_cells = true;
                if !options.blink_on {
                    classes.push("xterm-blink-hidden".into());
                }
            }
            let decorations =
                (context.decorations)(x, options.row).map_err(RowError::Decoration)?;
            let mut chars = if cell.chars.is_empty() {
                vec![32]
            } else {
                cell.chars.clone()
            };
            if chars == [32] && (cell.attrs.is_underline() != 0 || cell.attrs.is_overline() != 0) {
                chars[0] = 160;
            }
            let spacing = cell.pixel_width(options.cell_width)?
                - widths.get(
                    &chars,
                    cell.attrs.is_bold() != 0,
                    cell.attrs.is_italic() != 0,
                )?;
            let can_merge = old.as_ref().is_some_and(|old| {
                mergeable
                    && ((selected && old.selected)
                        || (!selected && !old.selected && cell.attrs.bg == old.bg))
                    && ((selected && old.selected && colors.selection_foreground.is_some())
                        || cell.attrs.fg == old.fg)
                    && cell.attrs.extended.ext() == old.extended
                    && link == old.link
                    && spacing == old.spacing
                    && !cursor
                    && !joined
                    && decorations.is_empty()
                    && valid_join
            });
            if can_merge {
                // A previous span exists whenever old/mergeable exist.
                if let Some(span) = result.spans.last_mut() {
                    if cell.attrs.is_invisible() != 0 {
                        span.text.push(32);
                    } else {
                        span.text.extend(chars);
                    }
                }
                x += 1.0;
                continue;
            }
            old = Some(MergeState {
                selected,
                bg: cell.attrs.bg,
                fg: cell.attrs.fg,
                extended: cell.attrs.extended.ext(),
                link,
                spacing,
            });
            let mut span = RowSpan::default();
            // Endpoint getters can change the span after cursor validation. The
            // source moves its local cursor even if a later range revisits cells.
            if joined && cursor_x >= x && cursor_x <= last_x {
                cursor_x = x;
            }
            glyph_classes(&mut classes, &cell, options, cursor);
            span.text = if cell.attrs.is_invisible() != 0 || cell.chars.is_empty() {
                vec![32]
            } else {
                cell.chars.clone()
            };
            if cell.attrs.is_underline() != 0 {
                classes.push(format!(
                    "xterm-underline-{}",
                    cell.attrs.extended.underline_style()
                ));
                if span.text == [32] {
                    span.text[0] = 160;
                }
                if !cell.attrs.is_underline_color_default() {
                    if cell.attrs.is_underline_color_rgb() {
                        let [r, g, b] =
                            AttributeData::to_color_rgb(cell.attrs.get_underline_color() as u32);
                        span.style("text-decoration-color", format!("rgb({r},{g},{b})"));
                    } else {
                        let mut index = cell.attrs.get_underline_color();
                        if options.bold_bright && cell.attrs.is_bold() != 0 && index < 8 {
                            index += 8;
                        }
                        span.style(
                            "text-decoration-color",
                            row_colors::palette::<M::Error>(colors, index)?.css,
                        );
                    }
                }
            }
            if cell.attrs.is_overline() != 0 {
                classes.push("xterm-overline".into());
                if span.text == [32] {
                    span.text[0] = 160;
                }
            }
            if cell.attrs.is_strikethrough() != 0 {
                classes.push("xterm-strikethrough".into());
            }
            if link {
                span.style("text-decoration", "underline".into());
            }
            // Keep class order, including the source's pending blink classes from
            // merged cells, before decoration and foreground/background classes.
            span.classes = std::mem::take(&mut classes);
            row_colors::apply(&mut span, &cell, options, colors, &decorations, selected)?;
            mergeable = !cursor && !joined && decorations.is_empty() && valid_join;
            if spacing != options.default_spacing {
                span.style("letter-spacing", format!("{spacing}px"));
            }
            result.spans.push(span);
            x = last_x + 1.0;
        }
        Ok(result)
    }
}
fn glyph_classes(classes: &mut Vec<String>, cell: &RenderCell, options: &RowOptions, cursor: bool) {
    if !options.cursor_hidden && cursor && options.cursor_initialized {
        classes.push("xterm-cursor".into());
        if options.focused {
            if options.cursor_blink {
                classes.push("xterm-cursor-blink".into());
            }
            let style = match options.cursor_style.as_str() {
                "bar" => "bar",
                "underline" => "underline",
                _ => "block",
            };
            classes.push(format!("xterm-cursor-{style}"));
        } else if ["outline", "block", "bar", "underline"]
            .contains(&options.cursor_inactive_style.as_str())
        {
            classes.push(format!("xterm-cursor-{}", options.cursor_inactive_style));
        }
    }
    for (flag, class) in [
        (cell.attrs.is_bold(), "xterm-bold"),
        (cell.attrs.is_italic(), "xterm-italic"),
        (cell.attrs.is_dim(), "xterm-dim"),
    ] {
        if flag != 0 {
            classes.push(class.into());
        }
    }
}
