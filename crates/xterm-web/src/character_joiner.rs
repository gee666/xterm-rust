// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/services/CharacterJoinerService.ts.
mod ranges;
#[cfg(target_arch = "wasm32")]
pub mod raw;
mod storage;
pub use ranges::{merge_ranges, string_ranges_to_cell_ranges};
use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::Rc,
};
pub use storage::RangeStorage;
use xterm_core::buffer::{constants::content, AttributeData, BufferLine, CellData, CharData};

pub type JoinRange = [usize; 2];
pub type JoinerId = f64;
pub type JoinerHandler<R = Vec<JoinRange>> =
    Rc<dyn Fn(&[u16]) -> Result<R, <R as RangeStorage>::HandlerError>>;
pub type LineRef = Rc<RefCell<BufferLine>>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JoinerError(pub String);
impl fmt::Display for JoinerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for JoinerError {}
struct Joiner<R: RangeStorage> {
    id: JoinerId,
    handler: JoinerHandler<R>,
    #[cfg(target_arch = "wasm32")]
    record: Option<wasm_bindgen::JsValue>,
}

type ErrorReporter<R> = Rc<
    dyn Fn(&<R as RangeStorage>::HandlerError) -> Result<(), <R as RangeStorage>::ConversionError>,
>;
/// Handler failures are reported to the host logger and subsequent handlers run.
/// Registration uses live array traversal, including callback-driven mutation.
pub struct CharacterJoinerService<R: RangeStorage = Vec<JoinRange>> {
    handlers: RefCell<Vec<Joiner<R>>>,
    next_id: Cell<f64>,
    work_cell: RefCell<CellData>,
    report_error: ErrorReporter<R>,
}
impl<R: RangeStorage> CharacterJoinerService<R> {
    pub fn new(report_error: impl Fn(&R::HandlerError) + 'static) -> Self {
        Self::with_error_reporter(move |error| {
            report_error(error);
            Ok(())
        })
    }
    pub fn with_error_reporter(
        report_error: impl Fn(&R::HandlerError) -> Result<(), R::ConversionError> + 'static,
    ) -> Self {
        Self {
            handlers: RefCell::new(Vec::new()),
            next_id: Cell::new(0.0),
            work_cell: RefCell::new(CellData::default()),
            report_error: Rc::new(report_error),
        }
    }
    pub fn register(&self, handler: JoinerHandler<R>) -> JoinerId {
        let id = self.next_id.get();
        self.next_id.set(id + 1.0);
        self.handlers.borrow_mut().push(Joiner {
            id,
            handler,
            #[cfg(target_arch = "wasm32")]
            record: None,
        });
        id
    }
    pub fn deregister(&self, id: JoinerId) -> bool {
        let mut handlers = self.handlers.borrow_mut();
        if let Some(index) = handlers.iter().position(|handler| handler.id == id) {
            handlers.remove(index);
            true
        } else {
            false
        }
    }
    fn handler(&self, index: usize) -> Option<JoinerHandler<R>> {
        self.handlers
            .borrow()
            .get(index)
            .map(|entry| Rc::clone(&entry.handler))
    }
    pub fn try_get_joined_characters(
        &self,
        line: Option<LineRef>,
    ) -> Result<R, R::ConversionError> {
        if self.handlers.borrow().is_empty() {
            return Ok(R::empty());
        }
        let Some(line) = line else {
            return Ok(R::empty());
        };
        if line.borrow().is_empty() {
            return Ok(R::empty());
        }
        let text = line.borrow().translate_to_string(true, None, None, None);
        let trimmed_length = line.borrow().get_trimmed_length();
        let mut ranges = R::empty();
        let (mut start_column, mut string_index, mut start_string) = (0, 0, 0);
        let mut fg = line.borrow().get_fg(0).unwrap_or(0);
        let mut bg = line.borrow().get_bg(0).unwrap_or(0);
        for x in 0..trimmed_length {
            *self.work_cell.borrow_mut() = line.borrow().get_cell(x).unwrap_or_default();
            if self.work_cell.borrow().get_width() == 0 {
                continue;
            }
            let changed = {
                let cell = self.work_cell.borrow();
                cell.fg != fg || cell.bg != bg
            };
            if changed {
                if x - start_column > 1 {
                    ranges.append(self.join_ranges(
                        &text,
                        start_string,
                        string_index,
                        &line,
                        start_column,
                    )?)?;
                }
                start_column = x;
                start_string = string_index;
                let cell = self.work_cell.borrow();
                fg = cell.fg;
                bg = cell.bg;
            }
            string_index += self.work_cell.borrow().get_chars().len().max(1);
        }
        if trimmed_length - start_column > 1 {
            ranges.append(self.join_ranges(
                &text,
                start_string,
                string_index,
                &line,
                start_column,
            )?)?;
        }
        Ok(ranges)
    }
    fn join_ranges(
        &self,
        text: &[u16],
        start: usize,
        end: usize,
        line: &LineRef,
        start_column: usize,
    ) -> Result<R, R::ConversionError> {
        let segment = &text[start.min(text.len())..end.min(text.len())];
        let mut result = R::empty();
        match self
            .handler(0)
            .map_or_else(|| R::missing_handler(segment), |handler| handler(segment))
        {
            Ok(ranges) => result = ranges,
            Err(error) => (self.report_error)(&error)?,
        }
        let mut index = 1;
        while let Some(handler) = self.handler(index) {
            match handler(segment) {
                Ok(ranges) => {
                    if let Err(error) = result.merge(ranges) {
                        (self.report_error)(&error)?;
                    }
                }
                Err(error) => (self.report_error)(&error)?,
            }
            index += 1;
        }
        result.convert(line, start_column)?;
        Ok(result)
    }
}
impl CharacterJoinerService {
    pub fn get_joined_characters(&self, line: Option<LineRef>) -> Vec<JoinRange> {
        match self.try_get_joined_characters(line) {
            Ok(ranges) => ranges,
            Err(never) => match never {},
        }
    }
}

/// Joined cells deliberately do not retain the original extended attributes.
#[derive(Clone, Debug)]
pub struct JoinedCellData {
    pub content: u32,
    pub attributes: AttributeData,
    pub combined_data: Vec<u16>,
    width: i32,
}
impl JoinedCellData {
    pub fn new(first: &CellData, text: Vec<u16>, width: i32) -> Self {
        Self {
            content: 0,
            attributes: AttributeData {
                fg: first.fg,
                bg: first.bg,
                ..AttributeData::default()
            },
            combined_data: text,
            width,
        }
    }
    pub fn is_combined(&self) -> u32 {
        content::IS_COMBINED_MASK
    }
    pub fn get_width(&self) -> i32 {
        self.width
    }
    pub fn get_chars(&self) -> &[u16] {
        &self.combined_data
    }
    pub fn get_code(&self) -> u32 {
        0x1fffff
    }
    pub fn get_as_char_data(&self) -> CharData {
        CharData {
            attr: self.attributes.fg,
            chars: self.combined_data.clone(),
            width: self.width,
            code: Some(self.get_code()),
        }
    }
    pub fn set_from_char_data(&mut self, _: &CharData) -> Result<(), JoinerError> {
        Err(JoinerError("not implemented".into()))
    }
}
