// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use serde_json::{json, Value};
use xterm_core::{
    buffer::{AttributeData, BufferLine, CellData, ExtendedAttrs},
    color::channels,
};
use xterm_web::{
    theme::{ColorRef, ColorSet, ThemeService},
    webgl::{
        cell_colors::{CellColorContext, CellColorResolver},
        glyphs::GlyphCell,
        model::RenderModel,
        renderer_frame::FrameHost,
        renderer_types::{GlyphColorNumbers, ModelCell, UpdateContext, UpdateHost},
    },
};

pub fn word(v: &Value) -> u32 {
    u32::try_from(v.as_u64().unwrap()).unwrap()
}
pub fn usize_value(v: &Value) -> usize {
    usize::try_from(v.as_u64().unwrap()).unwrap()
}
pub fn color(v: u32) -> ColorRef {
    channels::to_color(
        (v >> 24).into(),
        ((v >> 16) & 255).into(),
        ((v >> 8) & 255).into(),
        Some((v & 255).into()),
    )
    .into()
}
pub struct Host {
    pub trace: Vec<Value>,
    pub lines: Vec<Option<BufferLine>>,
    pub joins: Vec<Vec<[usize; 2]>>,
    pub context: UpdateContext,
    pub colors: ColorSet,
    pub frames: std::collections::VecDeque<bool>,
    pub attached: bool,
    pub ready: bool,
    pub atlas: bool,
    fail: Value,
    fail_count: usize,
}
impl Host {
    pub fn new(context: UpdateContext) -> Self {
        let theme = ThemeService::default();
        let mut colors = theme.colors().borrow().clone();
        colors.foreground = color(0xa1b2c3ff);
        colors.background = color(0x112233ff);
        colors.cursor = color(0x445566ff);
        colors.cursor_accent = color(0x778899ff);
        colors.selection_background_opaque = color(0x334455ff);
        colors.selection_inactive_background_opaque = color(0x556677ff);
        colors.selection_foreground = None;
        colors.ansi = (0..256).map(|i| color(((i * 97531) << 8) | 255)).collect();
        Self {
            trace: vec![],
            lines: vec![],
            joins: vec![],
            context,
            colors,
            frames: Default::default(),
            attached: true,
            ready: true,
            atlas: true,
            fail: Value::Null,
            fail_count: 0,
        }
    }
    pub fn step(&mut self, step: &Value, context: UpdateContext) {
        self.trace.clear();
        self.context = context;
        self.fail = step["fail"].clone();
        self.fail_count = 0;
        self.attached = step["attached"].as_bool().unwrap();
        self.ready = step["ready"].as_bool().unwrap();
        self.atlas = step["atlas"].as_bool().unwrap();
        self.frames = step["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_bool().unwrap())
            .collect();
        self.joins = serde_json::from_value(step["joins"].clone()).unwrap();
        self.lines = step["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                row.as_array().map(|row| {
                    let mut line = BufferLine::new(context.cols, None, false);
                    for (x, c) in row.iter().enumerate() {
                        line.set_cell(
                            x,
                            &CellData {
                                content: word(&c["content"]),
                                attributes: AttributeData {
                                    fg: word(&c["fg"]),
                                    bg: word(&c["bg"]),
                                    extended: ExtendedAttrs::new(word(&c["ext"]), 0),
                                },
                                combined_data: serde_json::from_value(c["chars"].clone()).unwrap(),
                            },
                        );
                    }
                    line
                })
            })
            .collect();
    }
    fn record(&mut self, name: &str, event: Value) -> Result<(), &'static str> {
        self.trace.push(event);
        if self.fail["name"].as_str() == Some(name) {
            self.fail_count += 1;
            if self.fail_count == usize_value(&self.fail["at"]) {
                return Err("injected");
            }
        }
        Ok(())
    }
}
impl UpdateHost for Host {
    type Error = &'static str;
    fn has_line(&mut self, row: usize) -> Result<bool, Self::Error> {
        self.record("line", json!(["line", row]))?;
        Ok(self
            .lines
            .get(row - self.context.viewport_y)
            .is_some_and(Option::is_some))
    }
    fn joined_characters(&mut self, row: usize) -> Result<Vec<[usize; 2]>, Self::Error> {
        self.record("joins", json!(["joins", row]))?;
        Ok(self.joins[row - self.context.viewport_y].clone())
    }
    fn load_cell(&mut self, row: usize, x: usize, cell: &mut CellData) -> Result<(), Self::Error> {
        self.record("load", json!(["load", x]))?;
        self.lines[row - self.context.viewport_y]
            .as_ref()
            .unwrap()
            .load_cell(x, cell)
            .unwrap();
        Ok(())
    }
    fn joined_text(&mut self, row: usize, range: [usize; 2]) -> Result<Vec<u16>, Self::Error> {
        self.record("translate", json!(["translate", range[0], range[1]]))?;
        Ok(self.lines[row - self.context.viewport_y]
            .as_ref()
            .unwrap()
            .translate_to_string(true, Some(range[0]), Some(range[1]), None))
    }
    fn resolve_colors(
        &mut self,
        cell: &ModelCell,
        x: usize,
        row: usize,
        selected: bool,
        resolver: &mut CellColorResolver,
    ) -> Result<(), Self::Error> {
        self.record("resolve", json!(["resolve", x, row, cell.code]))?;
        resolver
            .resolve(
                &cell.attributes,
                cell.code.unwrap_or(0),
                CellColorContext {
                    x: x as f64,
                    y: row as f64,
                    device_cell_width: 8.25,
                    device_cell_height: 17.5,
                    font_size: 15.0,
                    device_pixel_ratio: 1.25,
                    selected,
                    focused: self.context.focused,
                },
                &self.colors,
                &[],
            )
            .map_err(|_| "palette")
    }
    fn cursor_colors(&mut self) -> Result<[u32; 2], Self::Error> {
        Ok([
            self.colors.cursor_accent.borrow().rgba,
            self.colors.cursor.borrow().rgba,
        ])
    }
    fn update_glyph(
        &mut self,
        c: GlyphCell<'_>,
        numbers: GlyphColorNumbers,
    ) -> Result<(), Self::Error> {
        self.record(
            "glyph",
            json!([
                "glyph",
                c.x,
                c.y,
                c.code,
                numbers.current[0],
                numbers.current[1],
                numbers.current[2],
                c.chars,
                c.width as usize,
                numbers.previous_background
            ]),
        )
    }
    fn update_backgrounds(&mut self, _: &RenderModel) -> Result<(), Self::Error> {
        self.record("backgrounds", json!(["backgrounds"]))
    }
    fn update_cursor(&mut self, _: &RenderModel) -> Result<(), Self::Error> {
        self.record("cursor", json!(["cursor"]))
    }
    fn needs_text_blink(&mut self, v: bool) -> Result<(), Self::Error> {
        self.record("blink", json!(["blink", v]))
    }
}
impl FrameHost for Host {
    fn ensure_attached(&mut self) -> Result<bool, Self::Error> {
        Ok(self.attached)
    }
    fn grid_changed(&mut self, start: i64, end: i64) -> Result<(), Self::Error> {
        self.record("grid", json!(["grid", start, end]))
    }
    fn renderers_ready(&self) -> bool {
        self.ready
    }
    fn begin_frame(&mut self) -> Result<bool, Self::Error> {
        let v = self.frames.pop_front().unwrap_or(false);
        self.record("begin", json!(["begin", v]))?;
        Ok(v)
    }
    fn has_atlas(&self) -> bool {
        self.atlas
    }
    fn clear_glyphs(&mut self) -> Result<(), Self::Error> {
        self.record("clear", json!(["clear"]))
    }
    fn invalidate_atlas_textures(&mut self) -> Result<(), Self::Error> {
        self.record("invalidate", json!(["invalidate"]))
    }
    fn render_backgrounds(&mut self) -> Result<(), Self::Error> {
        self.record("draw-backgrounds", json!(["draw-backgrounds"]))
    }
    fn render_glyphs(&mut self, _: &RenderModel) -> Result<(), Self::Error> {
        self.record("draw-glyphs", json!(["draw-glyphs"]))
    }
    fn cursor_visible(&self) -> bool {
        self.context.cursor_blink_visible
    }
    fn render_cursor(&mut self) -> Result<(), Self::Error> {
        self.record("draw-cursor", json!(["draw-cursor"]))
    }
}
