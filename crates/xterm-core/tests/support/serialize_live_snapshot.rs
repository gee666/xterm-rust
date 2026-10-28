use xterm_core::{
    buffer::{AttributeData, Buffer, CellData},
    headless::{ModesSnapshot, Terminal},
};
#[derive(Debug, PartialEq, Eq)]
pub struct Attributes {
    fg: u32,
    bg: u32,
    underline_style: u32,
    underline_color: i32,
    underline_color_mode: u32,
}
fn attributes(a: &AttributeData) -> Attributes {
    Attributes {
        fg: a.fg,
        bg: a.bg,
        underline_style: a.get_underline_style(),
        underline_color: a.get_underline_color(),
        underline_color_mode: a.get_underline_color_mode(),
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Cell {
    chars: Vec<u16>,
    width: i32,
    code: Option<u32>,
    attrs: Attributes,
}
fn cell(c: CellData) -> Cell {
    Cell {
        chars: c.get_chars(),
        width: c.get_width(),
        code: c.get_code(),
        attrs: attributes(&c),
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Row {
    wrapped: bool,
    cells: Vec<Cell>,
}
#[derive(Debug, PartialEq, Eq)]
struct BufferSnapshot {
    x: usize,
    y: usize,
    base: usize,
    display: usize,
    top: usize,
    bottom: usize,
    rows: Vec<Row>,
}
fn buffer(b: &Buffer) -> BufferSnapshot {
    BufferSnapshot {
        x: b.x,
        y: b.y,
        base: b.ybase,
        display: b.ydisp,
        top: b.scroll_top,
        bottom: b.scroll_bottom,
        rows: (0..b.lines.len())
            .map(|index| {
                let row = b.lines.get(index).unwrap();
                let row = row.borrow();
                Row {
                    wrapped: row.is_wrapped,
                    cells: (0..row.len())
                        .map(|col| cell(row.get_cell(col).unwrap()))
                        .collect(),
                }
            })
            .collect(),
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct TerminalSnapshot {
    cols: usize,
    rows: usize,
    normal: BufferSnapshot,
    alternate: Option<BufferSnapshot>,
    active_alternate: bool,
    modes: ModesSnapshot,
    current: Attributes,
}
impl TerminalSnapshot {
    pub fn normal_cursor(&self) -> [usize; 2] {
        [self.normal.x, self.normal.y]
    }
    pub fn with_normal_cursor(mut self, x: usize, y: usize) -> Self {
        self.normal.x = x;
        self.normal.y = y;
        self
    }
}
pub fn terminal(t: &Terminal) -> TerminalSnapshot {
    let modes = t.modes().unwrap();
    t.with_state(|s| {
        let normal = s.buffers.normal();
        let alternate = s.buffers.alt();
        let active_alternate = std::rc::Rc::ptr_eq(&s.buffers.active(), &alternate);
        let normal = normal.borrow();
        let alternate = alternate.borrow();
        TerminalSnapshot {
            cols: s.cols(),
            rows: s.rows(),
            normal: buffer(&normal),
            alternate: (!alternate.lines.is_empty()).then(|| buffer(&alternate)),
            active_alternate,
            modes,
            current: attributes(&s.attributes),
        }
    })
    .unwrap()
}
