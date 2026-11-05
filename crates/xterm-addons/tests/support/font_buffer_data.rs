// Independent reconstruction of the shared source inputs, not copied expected results.
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions, FontWarning};

fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
fn put_word(bytes: &mut [u8], at: usize, value: usize) {
    bytes[at..at + 2].copy_from_slice(&u16::try_from(value).unwrap().to_be_bytes());
}
fn put_offset(bytes: &mut [u8], at: usize, value: usize) {
    bytes[at..at + 4].copy_from_slice(&u32::try_from(value).unwrap().to_be_bytes());
}
fn directory(bytes: &[u8], tag: &str) -> (usize, usize, bool) {
    let woff = &bytes[..4] == b"wOFF";
    let (start, width, count_at) = if woff { (44, 20, 12) } else { (12, 16, 4) };
    for index in 0..word(bytes, count_at) {
        let at = start + index * width;
        if &bytes[at..at + 4] == tag.as_bytes() {
            return (at, offset(bytes, at + if woff { 4 } else { 8 }), woff);
        }
    }
    panic!("Missing fixture table {tag}");
}
pub fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../xterm-font-addons/tests/fixtures/font-buffer-mutations.json"
    ))
    .unwrap()
}
pub fn construct(item: &Value) -> Vec<u8> {
    let base = include_bytes!("../fixtures/ligature-fonts/FiraCode-Regular.otf");
    let count = word(base, 4);
    let end = 12 + count * 16;
    let mut bytes = vec![0; base.len() + 16 + 24];
    bytes[..end].copy_from_slice(&base[..end]);
    bytes[end + 16..base.len() + 16].copy_from_slice(&base[end..]);
    put_word(&mut bytes, 4, count + 1);
    for index in 0..count {
        let at = 12 + index * 16 + 8;
        put_offset(&mut bytes, at, offset(base, at) + 16);
    }
    bytes[end..end + 4].copy_from_slice(b"kern");
    put_offset(&mut bytes, end + 8, base.len() + 16);
    put_offset(&mut bytes, end + 12, 24);
    let kern = &mut bytes[base.len() + 16..];
    put_word(kern, 0, 1);
    put_offset(kern, 4, 2);
    put_word(
        kern,
        12,
        usize::try_from(item["coverage"].as_u64().unwrap()).unwrap(),
    );
    put_word(
        kern,
        16,
        usize::try_from(item["pairs"].as_u64().unwrap()).unwrap(),
    );
    if item["format"] == "SFNT" {
        return bytes;
    }
    let mut output = vec![0; 44 + (count + 1) * 20];
    output[..4].copy_from_slice(b"wOFF");
    output[4..8].copy_from_slice(&bytes[..4]);
    put_word(&mut output, 12, count + 1);
    for index in 0..=count {
        let at = 12 + index * 16;
        let record = 44 + index * 20;
        let start = offset(&bytes, at + 8);
        let length = offset(&bytes, at + 12);
        let data = &bytes[start..start + length];
        let payload = if item["format"] == "WOFF-kern" && &bytes[at..at + 4] == b"kern" {
            miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
        } else {
            data.to_vec()
        };
        assert!(payload.len() <= data.len());
        output[record..record + 4].copy_from_slice(&bytes[at..at + 4]);
        let cursor = output.len();
        put_offset(&mut output, record + 4, cursor);
        put_offset(&mut output, record + 8, payload.len());
        put_offset(&mut output, record + 12, length);
        output.extend_from_slice(&payload);
    }
    output.resize(output.len() + 64, 0);
    let size = output.len();
    put_offset(&mut output, 8, size);
    output
}
pub fn mutate(bytes: &mut [u8], item: &Value) {
    for change in item["writes"].as_array().unwrap() {
        let target = change[0].as_str().unwrap();
        let relative = usize::try_from(change[1].as_u64().unwrap()).unwrap();
        let data: Vec<_> = change[2]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
            .collect();
        if target == "calt" {
            let (_, table, _) = directory(bytes, "GSUB");
            let features = table + word(bytes, table + 6);
            let mut found = false;
            for index in 0..word(bytes, features) {
                let at = features + 2 + index * 6;
                if &bytes[at..at + 4] == b"calt" {
                    bytes[at..at + data.len()].copy_from_slice(&data);
                    found = true;
                }
            }
            assert!(found);
            continue;
        }
        let at = if let Some(tag) = target.strip_prefix("directory:") {
            let (record, _, woff) = directory(bytes, tag);
            record + relative - if woff { 4 } else { 0 }
        } else {
            directory(bytes, target).1 + relative
        };
        bytes[at..at + data.len()].copy_from_slice(&data);
    }
}
pub fn assert_corpus() {
    let fixture = fixture();
    let cases = fixture["constructors"].as_array().unwrap();
    assert_eq!(cases.len(), 39);
    for item in cases {
        let bytes = construct(item);
        let mut current = bytes.clone();
        let mut warnings = 0;
        let actual = Font::load_buffer_with_refresh(&bytes, FontOptions::default(), |warning| {
            assert_eq!(warning, FontWarning::MultipleKernSubtables);
            warnings += 1;
            mutate(&mut current, item);
            if item["throws"] == true {
                Err("original callback error")
            } else {
                Ok(Some(current.clone()))
            }
        });
        assert_eq!(
            warnings,
            item["warnings"].as_u64().unwrap(),
            "{}",
            item["name"]
        );
        match actual {
            Ok(mut font) => {
                assert_eq!(item["outcome"]["status"], "ok", "{}", item["name"]);
                let ranges = font
                    .find_ligature_ranges(&"a===b".encode_utf16().collect::<Vec<_>>())
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&*ranges.borrow()).unwrap(),
                    item["outcome"]["ranges"],
                    "{}",
                    item["name"]
                );
            }
            Err(FontLoadError::Font(error)) => {
                assert_eq!(
                    item["outcome"]["status"], "error",
                    "{}: {error:?}",
                    item["name"]
                );
                assert_eq!(item["outcome"]["name"], "RangeError");
                assert_eq!(
                    item["outcome"]["message"],
                    "Offset is outside the bounds of the DataView"
                );
                assert_eq!(error, FontError::Bounds, "{}", item["name"]);
            }
            Err(FontLoadError::Warning(error)) => {
                assert_eq!(error, "original callback error");
                assert_eq!(item["outcome"]["status"], "warning", "{}", item["name"]);
            }
        }
    }
}
