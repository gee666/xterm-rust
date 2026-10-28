//! Read-only live-buffer snapshots for URL computation without engine borrows
//! held across host RegExp calls or hover/activation callbacks.
use crate::web_links::{LinkBuffer, LinkCell, LinkError, LinkLine};
use xterm_core::terminal::{CoreTerminal, TerminalState};

struct SnapshotLine {
    wrapped: bool,
    text: Vec<u16>,
    cells: Vec<LinkCell>,
}
impl LinkLine for SnapshotLine {
    fn is_wrapped(&self) -> bool {
        self.wrapped
    }
    fn translate_trimmed(&self) -> Vec<u16> {
        self.text.clone()
    }
    fn len(&self) -> usize {
        self.cells.len()
    }
    fn cell(&self, index: usize) -> Option<LinkCell> {
        self.cells.get(index).cloned()
    }
}
pub struct LinkSnapshot(Vec<Option<SnapshotLine>>);
impl LinkBuffer for LinkSnapshot {
    fn line(&self, index: usize) -> Option<&dyn LinkLine> {
        self.0
            .get(index)?
            .as_ref()
            .map(|line| line as &dyn LinkLine)
    }
}
impl LinkSnapshot {
    pub fn from_engine(engine: &CoreTerminal) -> Result<Self, LinkError> {
        Self::from_state(&engine.state())
    }
    pub fn from_state(state: &TerminalState) -> Result<Self, LinkError> {
        let active = state.active_buffer();
        let buffer = active
            .try_borrow()
            .map_err(|_| LinkError::Message("active buffer is borrowed".into()))?;
        let mut lines = Vec::with_capacity(buffer.lines.len());
        for row in 0..buffer.lines.len() {
            let Some(line) = buffer.lines.get(row) else {
                lines.push(None);
                continue;
            };
            let line = line
                .try_borrow()
                .map_err(|_| LinkError::Message("buffer line is borrowed".into()))?;
            let mut cells = Vec::with_capacity(line.len());
            for column in 0..line.len() {
                let cell = line.get_cell(column).ok_or_else(|| {
                    LinkError::Message("buffer cell is absent within line".into())
                })?;
                cells.push(LinkCell {
                    chars: cell.get_chars(),
                    width: u8::try_from(cell.get_width())
                        .map_err(|_| LinkError::Message("negative cell width".into()))?,
                });
            }
            lines.push(Some(SnapshotLine {
                wrapped: line.is_wrapped,
                text: line.translate_to_string(true, None, None, None),
                cells,
            }));
        }
        Ok(Self(lines))
    }
}
