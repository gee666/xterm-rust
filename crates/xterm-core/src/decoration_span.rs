use crate::buffer::marker_line::MarkerLine;
use std::fmt;
/// The original numeric loop cannot finish when Number increment stops advancing.
/// Native hosts return this error rather than hanging on such a span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonAdvancingDecorationSpan;
impl fmt::Display for NonAdvancingDecorationSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("decoration line span cannot advance at JavaScript Number precision")
    }
}
impl std::error::Error for NonAdvancingDecorationSpan {}

const UNIT_INCREMENT_LIMIT: f64 = 9_007_199_254_740_992.0;

pub(super) fn end(start: &MarkerLine, height: i64) -> MarkerLine {
    // Both operands are primitives. No host conversion or callback can fail.
    start
        .add_number(height as f64)
        .expect("primitive line plus numeric height cannot fail")
}

pub(super) fn validate(start: &MarkerLine, height: i64) -> Result<(), NonAdvancingDecorationSpan> {
    let end = end(start, height);
    if !start.rel_lt(&end) {
        return Ok(());
    }
    // The first comparison may be String/String and its bucket retains that
    // String key. Post-increment converts it to Number for all later iterations.
    let start = if start.as_string().is_some() {
        start.increment().to_number()
    } else {
        start.to_number()
    };
    let end = end.to_number();
    let next = start + 1.0;
    if start < end
        && (next == start
            || (start < UNIT_INCREMENT_LIMIT && end > UNIT_INCREMENT_LIMIT)
            || (next < end && next + 1.0 == next))
    {
        return Err(NonAdvancingDecorationSpan);
    }
    Ok(())
}

pub(super) fn for_each_line(
    start: &MarkerLine,
    height: i64,
    mut visit: impl FnMut(MarkerLine),
) -> Result<(), NonAdvancingDecorationSpan> {
    validate(start, height)?;
    let end = end(start, height);
    let mut line = start.clone();
    while line.rel_lt(&end) {
        let next = line.increment();
        visit(line);
        line = next;
    }
    Ok(())
}
