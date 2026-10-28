use crate::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
    terminal::TerminalEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderEvent {
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResizeEvent {
    pub cols: usize,
    pub rows: usize,
}

#[derive(Default)]
pub(super) struct Events {
    bell: Emitter<()>,
    binary: Emitter<Vec<u16>>,
    cursor_move: Emitter<()>,
    data: Emitter<Vec<u16>>,
    line_feed: Emitter<()>,
    render: Emitter<RenderEvent>,
    resize: Emitter<ResizeEvent>,
    scroll: Emitter<usize>,
    title: Emitter<Vec<u16>>,
    write_parsed: Emitter<()>,
    suppress_parsed: std::cell::Cell<bool>,
    all: Emitter<TerminalEvent>,
}
impl Events {
    pub fn bell(&self) -> Event<()> {
        self.bell.event()
    }
    pub fn binary(&self) -> Event<Vec<u16>> {
        self.binary.event()
    }
    pub fn cursor_move(&self) -> Event<()> {
        self.cursor_move.event()
    }
    pub fn data(&self) -> Event<Vec<u16>> {
        self.data.event()
    }
    pub fn line_feed(&self) -> Event<()> {
        self.line_feed.event()
    }
    pub fn render(&self) -> Event<RenderEvent> {
        self.render.event()
    }
    pub fn resize(&self) -> Event<ResizeEvent> {
        self.resize.event()
    }
    pub fn scroll(&self) -> Event<usize> {
        self.scroll.event()
    }
    pub fn title(&self) -> Event<Vec<u16>> {
        self.title.event()
    }
    pub fn write_parsed(&self) -> Event<()> {
        self.write_parsed.event()
    }
    pub fn all(&self) -> Event<TerminalEvent> {
        self.all.event()
    }
    pub fn with_queued_parse<T>(&self, action: impl FnOnce() -> T) -> T {
        struct Restore<'a> {
            flag: &'a std::cell::Cell<bool>,
            previous: bool,
        }
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.flag.set(self.previous);
            }
        }
        let _restore = Restore {
            flag: &self.suppress_parsed,
            previous: self.suppress_parsed.replace(true),
        };
        action()
    }
    pub fn dispatch(&self, event: TerminalEvent) -> Result<(), EmitError> {
        // The source WriteBuffer owns parsed notifications after completion
        // callbacks and batching. A queued parse must not invent one per chunk.
        if event == TerminalEvent::WriteParsed && self.suppress_parsed.get() {
            return Ok(());
        }
        match &event {
            TerminalEvent::Bell => self.bell.fire(&())?,
            TerminalEvent::Binary(data) => self.binary.fire(data)?,
            TerminalEvent::CursorMove => self.cursor_move.fire(&())?,
            TerminalEvent::Data(data) => self.data.fire(data)?,
            TerminalEvent::LineFeed => self.line_feed.fire(&())?,
            TerminalEvent::Render { start, end } => self.render.fire(&RenderEvent {
                start: *start,
                end: *end,
            })?,
            TerminalEvent::Resize { cols, rows } => self.resize.fire(&ResizeEvent {
                cols: *cols,
                rows: *rows,
            })?,
            TerminalEvent::Scroll(position) => self.scroll.fire(position)?,
            TerminalEvent::TitleChange(title) => self.title.fire(title)?,
            TerminalEvent::WriteParsed => self.write_parsed.fire(&())?,
            TerminalEvent::AccessibilityChar(_)
            | TerminalEvent::AccessibilityTab(_)
            | TerminalEvent::RequestReset
            | TerminalEvent::SyncScrollBar
            | TerminalEvent::SendFocus
            | TerminalEvent::ColorSchemeQuery
            | TerminalEvent::WindowReport(_)
            | TerminalEvent::Color(_)
            | TerminalEvent::InputScroll(_) => {
                // These internal browser requests have no corresponding public
                // headless event or host listener upstream. The raw observer
                // below still receives them for an attached browser adapter.
            }
        }
        self.all.fire(&event)
    }
    /// CoreTerminal field emitters are registered before constructor services.
    /// Headless-specific bell/cursor/title emitters are registered afterward.
    pub fn dispose_core(&self) {
        self.binary.dispose();
        self.data.dispose();
        self.line_feed.dispose();
        self.render.dispose();
        self.resize.dispose();
        self.scroll.dispose();
        self.write_parsed.dispose();
        self.all.dispose();
    }
    pub fn dispose(&self) {
        self.dispose_core();
        self.bell.dispose();
        self.cursor_move.dispose();
        self.title.dispose();
    }
}
