use std::rc::Rc;
use xterm_core::{
    headless::Terminal,
    options::{OptionValue, OptionsService},
};

pub fn terminal(cols: usize, rows: usize) -> Terminal {
    Terminal::with_options(Rc::new(OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(cols as f64)),
            ("rows", OptionValue::Number(rows as f64)),
            ("allowProposedApi", OptionValue::Bool(true)),
        ],
    )))
    .unwrap()
}
pub fn line(term: &Terminal, index: usize, trim: bool) -> String {
    view_line(&term.buffer().unwrap().active().unwrap(), index, trim)
}
pub fn view_line(
    view: &xterm_core::public_api::buffer::BufferApiView,
    index: usize,
    trim: bool,
) -> String {
    let line = view.get_line(index).unwrap().unwrap();
    let units = line.borrow().translate_to_string(trim, None, None, None);
    String::from_utf16(&units).unwrap()
}
pub fn line_equals(term: &Terminal, index: usize, expected: &str) {
    assert_eq!(line(term, index, true), expected);
}
