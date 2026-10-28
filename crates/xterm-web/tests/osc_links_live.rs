use xterm_core::buffer::{AttributeData, BufferLine};
use xterm_web::links::{osc_links, try_osc_links, OscBuffer};
#[test]
fn high_safe_integer_url_id_is_not_truncated() {
    let id = 1_u64 << 53;
    let mut line = BufferLine::new(3, None, false);
    let mut attrs = AttributeData::default();
    attrs.extended.set_url_id(id);
    attrs.update_extended();
    line.set_cell_from_codepoint(0, u32::from('a'), 1, &attrs);
    line.set_cell_from_codepoint(1, u32::from('b'), 1, &attrs);
    line.set_cell_from_codepoint(2, u32::from('x'), 1, &AttributeData::default());
    let result = osc_links(
        &[line],
        1,
        |found| {
            assert_eq!(found, id);
            Some(vec![0xd800, 0x61])
        },
        true,
        |_| panic!("bypassed"),
    );
    assert_eq!(result.unwrap()[0].text, vec![0xd800, 0x61]);
}
struct Unavailable;
impl OscBuffer for Unavailable {
    type Error = &'static str;
    fn with_line<R>(
        &self,
        _: usize,
        _: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, Self::Error> {
        Err("borrowed")
    }
}
#[test]
fn live_row_borrow_failures_are_not_missing_rows() {
    assert_eq!(
        try_osc_links(&Unavailable, 1, |_| Ok(None), false, |_| true),
        Err("borrowed")
    );
}
