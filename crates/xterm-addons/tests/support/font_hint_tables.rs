// Independent Rust reconstruction checks the shared JavaScript input builder.
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-hint-tables.json")).unwrap()
}
fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
struct Record {
    tag: [u8; 4],
    data: Vec<u8>,
    offset: Option<u32>,
    length: u32,
    compressed_length: u32,
}
fn payload(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
        .collect()
}
fn number(value: &Value, fallback: usize) -> u32 {
    value
        .as_u64()
        .map(|v| u32::try_from(v).unwrap())
        .unwrap_or(u32::try_from(fallback).unwrap())
}
pub fn construct(change: &Value) -> Vec<u8> {
    let base = include_bytes!("../fixtures/ligature-fonts/UbuntuMono-Regular.ttf");
    let mut records = Vec::new();
    for index in 0..word(base, 4) {
        let at = 12 + index * 16;
        let tag: [u8; 4] = base[at..at + 4].try_into().unwrap();
        if tag.as_slice() == change["tag"].as_str().unwrap().as_bytes() {
            continue;
        }
        let start = offset(base, at + 8);
        let length = offset(base, at + 12);
        let mut data = base[start..start + length].to_vec();
        if &tag == b"head" && change["badHead"] == true {
            data[12] ^= 1;
        }
        records.push(Record {
            tag,
            data,
            offset: None,
            length: u32::try_from(length).unwrap(),
            compressed_length: u32::try_from(length).unwrap(),
        });
    }
    for extra in change["entries"].as_array().unwrap() {
        let data = payload(&extra["data"]);
        let record = Record {
            tag: change["tag"]
                .as_str()
                .unwrap()
                .as_bytes()
                .try_into()
                .unwrap(),
            offset: extra["offset"].as_u64().map(|v| u32::try_from(v).unwrap()),
            length: number(&extra["length"], data.len()),
            compressed_length: number(&extra["compressedLength"], data.len()),
            data,
        };
        if extra["first"] == true {
            records.insert(0, record);
        } else {
            records.push(record);
        }
    }
    if !change["tail"].is_null() {
        let data = payload(&change["tail"]);
        let length = u32::try_from(data.len()).unwrap();
        records.push(Record {
            tag: *b"zzzz",
            data,
            offset: None,
            length,
            compressed_length: length,
        });
    }
    let woff = change["woff"] == true;
    let (header, width) = if woff { (44, 20) } else { (12, 16) };
    let mut output = vec![0; header + records.len() * width];
    if woff {
        output[..4].copy_from_slice(b"wOFF");
        output[4..8].copy_from_slice(&base[..4]);
    } else {
        output[..12].copy_from_slice(&base[..12]);
    }
    let count_at = if woff { 12 } else { 4 };
    output[count_at..count_at + 2]
        .copy_from_slice(&u16::try_from(records.len()).unwrap().to_be_bytes());
    for (index, record) in records.iter().enumerate() {
        let at = header + index * width;
        output[at..at + 4].copy_from_slice(&record.tag);
        let offset = record
            .offset
            .unwrap_or(u32::try_from(output.len()).unwrap());
        let offset_at = at + if woff { 4 } else { 8 };
        output[offset_at..offset_at + 4].copy_from_slice(&offset.to_be_bytes());
        if woff {
            output[at + 8..at + 12].copy_from_slice(&record.compressed_length.to_be_bytes());
        }
        output[at + 12..at + 16].copy_from_slice(&record.length.to_be_bytes());
        output.extend_from_slice(&record.data);
    }
    if woff {
        let length = u32::try_from(output.len()).unwrap();
        output[8..12].copy_from_slice(&length.to_be_bytes());
    }
    output
}
pub fn assert_corpus() {
    let fixture = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let expected = if case["outcome"]["status"] == "ok" {
            None
        } else {
            let message = case["outcome"]["message"].as_str().unwrap();
            Some(match message {
                "Invalid array length" => FontError::ArrayLength,
                "Font header has wrong magic number." => {
                    FontError::Invalid("Font header has wrong magic number.")
                }
                "Data error" => FontError::Compression,
                "Offset is outside the bounds of the DataView" => FontError::Bounds,
                value if value.starts_with("Invalid typed array length:") => FontError::Bounds,
                value => panic!("Unmapped source error {value}"),
            })
        };
        let bytes = construct(case);
        let actual = Font::load_buffer(&bytes, FontOptions::default());
        assert_eq!(actual.err(), expected, "{}", case["name"]);
    }
    // Kept separate from finite source outcomes: the unchanged source loops
    // on empty dynamic Huffman trees, while Rust must terminate safely.
    let nontermination = construct(&fixture["nontermination"]);
    assert_eq!(
        Font::load_buffer(&nontermination, FontOptions::default()).err(),
        Some(FontError::Compression)
    );
}
