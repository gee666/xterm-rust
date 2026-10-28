use xterm_core::buffer::AttributeData;
use xterm_core::parser::params::{ParamValue, Params};
use xterm_core::terminal::sgr::{apply, KittyControl, SgrAction, SgrCursor};

fn params(values: &[i32]) -> Params {
    Params::from_array(
        &values
            .iter()
            .copied()
            .map(ParamValue::Param)
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

#[test]
fn diagnostics_pause_before_following_attributes_and_allow_reentrant_changes() {
    let params = params(&[1, 999, 2]);
    let mut cursor = SgrCursor::new(&params);
    let mut attr = AttributeData::default();
    assert_eq!(cursor.next(&mut attr), SgrAction::Unknown(999));
    let prefix = [attr.fg, attr.bg];
    // A logger failure stops here. No faint attribute has been applied yet.
    assert_eq!(prefix, [0x08000000, 0]);
    apply(&mut attr, &self::params(&[0, 3]));
    assert_eq!(cursor.next(&mut attr), SgrAction::Done);
    assert_eq!([attr.fg, attr.bg], [0, 0x0c000000]);
    assert_eq!(cursor.next(&mut attr), SgrAction::Done);
}

#[test]
fn kitty_option_boundary_is_reached_only_for_kitty_parameters() {
    let params = params(&[1, 999, 221, 2, 222, 4]);
    let mut cursor = SgrCursor::new(&params);
    let mut attr = AttributeData::default();
    assert_eq!(cursor.next(&mut attr), SgrAction::Unknown(999));
    assert_eq!(cursor.next(&mut attr), SgrAction::Kitty(KittyControl::Bold));
    // A getter failure stops here, retaining the prefix state.
    assert_eq!([attr.fg, attr.bg], [0x08000000, 0]);
    assert_eq!(KittyControl::Bold.apply(&mut attr, false), Some(221));
    assert_eq!(
        cursor.next(&mut attr),
        SgrAction::Kitty(KittyControl::Faint)
    );
    assert_eq!([attr.fg, attr.bg], [0x08000000, 0x08000000]);
    assert_eq!(KittyControl::Faint.apply(&mut attr, true), None);
    assert_eq!(cursor.next(&mut attr), SgrAction::Done);
    assert_eq!([attr.fg, attr.bg], [0x18000000, 0x10000000]);
}

#[test]
fn resumed_processing_can_retain_the_detached_original_attribute_object() {
    let params = params(&[1, 999, 2]);
    let mut cursor = SgrCursor::new(&params);
    let mut original = AttributeData::default();
    assert_eq!(cursor.next(&mut original), SgrAction::Unknown(999));
    apply(&mut original, &self::params(&[3, 7]));
    let replacement = AttributeData::default();
    assert_eq!(cursor.next(&mut original), SgrAction::Done);
    assert_eq!([original.fg, original.bg], [0x0c000000, 0x0c000000]);
    assert_eq!([replacement.fg, replacement.bg], [0, 0]);
}
