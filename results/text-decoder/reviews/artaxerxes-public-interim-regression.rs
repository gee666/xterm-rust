#[path = "../crates/xterm-core/src/input/text_decoder.rs"]
pub mod decoder;
#[test]
fn public_interim_keeps_all_populated_prefix_bytes() {
    let mut decoder = decoder::Utf8ToUtf32::new();
    decoder.interim = [0xc2, 0x80, 0x80];
    let mut actual = Vec::new();
    assert_eq!(decoder.decode(&[b'A'], &mut actual), 2);
    assert_eq!(actual, [0x2000, 0x41]);
}
#[test]
fn fixed_target_keeps_same_source_prefix_and_logical_count() {
    let mut decoder = decoder::Utf8ToUtf32::new();
    decoder.interim = [0xc2, 0x80, 0x80];
    let mut actual = [0xdead_beef];
    assert_eq!(decoder.decode_to_slice(&[b'A'], &mut actual), 2);
    assert_eq!(actual, [0x2000]);
}
