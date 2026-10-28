use std::rc::Rc;
use xterm_core::buffer::{
    constants::{bg_flags, underline_style},
    BufferLine, CellData, ExtendedAttrs,
};

#[test]
fn original_js_url_ids_survive_32bit_and_safe_integer_boundaries() {
    let fixture = include_str!("fixtures/cell-url.tsv");
    assert_eq!(fixture.lines().count(), 6);
    for row in fixture.lines() {
        let expected: Vec<u64> = row
            .split('\t')
            .map(|value| value.parse().unwrap())
            .collect();
        let mut original = ExtendedAttrs::new(underline_style::CURLY << 26, expected[0]);
        original.set_payload(Some(Rc::new(())));
        let shared = original.share();
        let mut cloned = original.clone();
        let mut actual = vec![
            original.url_id(),
            u64::from(original.ext()),
            u64::from(original.underline_style()),
            cloned.url_id(),
            u64::from(cloned.ext()),
            u64::from(cloned.underline_style()),
            u64::from(Rc::ptr_eq(
                &original.payload().unwrap(),
                &shared.payload().unwrap(),
            )),
            u64::from(cloned.payload().is_none()),
        ];
        cloned.set_url_id(0);
        actual.extend([
            original.url_id(),
            u64::from(cloned.underline_style()),
            u64::from(cloned.ext()),
        ]);
        assert_eq!(actual, expected);

        let mut cell = CellData {
            attributes: xterm_core::buffer::AttributeData {
                extended: original,
                ..Default::default()
            },
            ..Default::default()
        };
        cell.update_extended();
        assert_eq!(cell.has_extended_attrs(), bg_flags::HAS_EXTENDED);
        let line = BufferLine::new(2, Some(&cell), false);
        for copied in [&line, &line.clone()] {
            for col in 0..2 {
                let loaded = copied.get_cell(col).unwrap();
                assert_eq!(loaded.extended.url_id(), expected[0]);
                assert!(loaded.extended.same_identity(&shared));
                assert!(loaded.attributes_equals(&cell));
            }
        }
    }
}
