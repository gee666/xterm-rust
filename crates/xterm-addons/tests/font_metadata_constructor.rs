use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
fn construct(change: &Value) -> Vec<u8> {
    let base = include_bytes!("fixtures/ligature-fonts/UbuntuMono-Regular.ttf");
    let mut records: Vec<([u8; 4], Vec<u8>)> = Vec::new();
    for index in 0..word(base, 4) {
        let at = 12 + index * 16;
        let tag = std::str::from_utf8(&base[at..at + 4]).unwrap();
        if change["tag"].as_str() == Some(tag)
            || change["remove"]
                .as_array()
                .is_some_and(|tags| tags.iter().any(|v| v == tag))
        {
            continue;
        }
        let start = offset(base, at + 8);
        records.push((
            base[at..at + 4].try_into().unwrap(),
            base[start..start + offset(base, at + 12)].to_vec(),
        ));
    }
    if let Some(data) = change["data"].as_array() {
        records.push((
            change["tag"]
                .as_str()
                .unwrap()
                .as_bytes()
                .try_into()
                .unwrap(),
            data.iter()
                .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                .collect(),
        ));
    }
    if let Some(position) = change["duplicateName"].as_str() {
        if position == "first" {
            records.insert(0, (*b"name", Vec::new()));
        } else {
            records.push((*b"name", Vec::new()));
        }
    }
    let mut output = vec![0; 12 + records.len() * 16];
    output[..12].copy_from_slice(&base[..12]);
    output[4..6].copy_from_slice(&u16::try_from(records.len()).unwrap().to_be_bytes());
    for (index, (tag, data)) in records.iter().enumerate() {
        let at = 12 + index * 16;
        output[at..at + 4].copy_from_slice(tag);
        let start = u32::try_from(output.len()).unwrap();
        output[at + 8..at + 12].copy_from_slice(&start.to_be_bytes());
        let length = if index + 1 == records.len() {
            change["directoryLength"]
                .as_u64()
                .map(|v| u32::try_from(v).unwrap())
        } else {
            None
        }
        .unwrap_or(u32::try_from(data.len()).unwrap());
        output[at + 12..at + 16].copy_from_slice(&length.to_be_bytes());
        output.extend_from_slice(data);
    }
    let compressed = change["woff"] == true;
    if compressed {
        output = woff(&output);
    }
    if let Some(position) = change["duplicateName"].as_str() {
        let index = if position == "first" {
            0
        } else {
            records.len() - 1
        };
        let at = if compressed {
            44 + index * 20
        } else {
            12 + index * 16
        };
        let offset = at + if compressed { 4 } else { 8 };
        output[offset..offset + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes());
        output[at + 12..at + 16].copy_from_slice(&128_u32.to_be_bytes());
        if compressed {
            output[at + 8..at + 12].copy_from_slice(&16_u32.to_be_bytes());
        }
    }
    output
}

fn woff(bytes: &[u8]) -> Vec<u8> {
    let count = word(bytes, 4);
    let mut output = vec![0; 44 + count * 20];
    output[..4].copy_from_slice(b"wOFF");
    output[4..8].copy_from_slice(&bytes[..4]);
    output[12..14].copy_from_slice(&u16::try_from(count).unwrap().to_be_bytes());
    for index in 0..count {
        let at = 12 + index * 16;
        let start = offset(bytes, at + 8);
        let data = &bytes[start..start + offset(bytes, at + 12)];
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(data, 6);
        let payload = if compressed.len() < data.len() {
            compressed.as_slice()
        } else {
            data
        };
        let record = 44 + index * 20;
        output[record..record + 4].copy_from_slice(&bytes[at..at + 4]);
        let start = u32::try_from(output.len()).unwrap();
        output[record + 4..record + 8].copy_from_slice(&start.to_be_bytes());
        output[record + 8..record + 12]
            .copy_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
        output[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend_from_slice(payload);
    }
    let size = u32::try_from(output.len()).unwrap();
    output[8..12].copy_from_slice(&size.to_be_bytes());
    output
}

#[test]
fn unchanged_source_metadata_constructor_outcomes() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/font-metadata.json")).unwrap();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let bytes = construct(case);
        let actual = Font::load_buffer(&bytes, FontOptions::default());
        let expected = case["outcome"]["status"] == "ok";
        if actual.is_ok() != expected {
            failures.push(format!(
                "{}: expected {}, got {:?}",
                case["name"],
                case["outcome"],
                actual.err()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn native_metadata_errors_identify_failed_contract() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/font-metadata.json")).unwrap();
    for (name, expected) in [
        (
            "head wrong magic",
            FontError::Invalid("Font header has wrong magic number."),
        ),
        (
            "ltag version 2",
            FontError::Invalid("Unsupported ltag table version."),
        ),
        ("missing name", FontError::MissingTable("name")),
        ("missing hmtx", FontError::MissingTable("hmtx")),
        ("hhea EOF 35", FontError::Bounds),
        (
            "post version 151552 size 35",
            FontError::Type("Cannot read properties of undefined (reading '0')"),
        ),
        (
            "maxp version 0 size 6",
            FontError::Type("Cannot read properties of undefined (reading 'addUnicode')"),
        ),
    ] {
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap();
        assert_eq!(
            Font::load_buffer(&construct(case), FontOptions::default()).err(),
            Some(expected),
            "{name}"
        );
    }
}
