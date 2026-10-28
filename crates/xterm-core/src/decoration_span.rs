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

pub(super) fn validate(start: f64, height: i64) -> Result<(), NonAdvancingDecorationSpan> {
    let end = start + height as f64;
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
    start: f64,
    height: i64,
    mut visit: impl FnMut(f64),
) -> Result<(), NonAdvancingDecorationSpan> {
    validate(start, height)?;
    let end = start + height as f64;
    let mut line = start;
    while line < end {
        let next = line + 1.0;
        if next == line {
            return Err(NonAdvancingDecorationSpan);
        }
        visit(line);
        line = next;
    }
    Ok(())
}
