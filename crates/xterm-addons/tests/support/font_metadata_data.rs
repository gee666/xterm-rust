// Shared fixture reconstruction for native and actual WASM font constructor tests.
use serde_json::Value;

fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
fn duplicate(change: &Value) -> Option<(&str, &str, bool)> {
    let extra = &change["duplicateTable"];
    let position = extra["position"]
        .as_str()
        .or_else(|| change["duplicateName"].as_str())?;
    Some((
        extra["tag"].as_str().unwrap_or("name"),
        position,
        extra["invalidCompression"] == true,
    ))
}
pub fn construct(change: &Value) -> Vec<u8> {
    let base = include_bytes!("../fixtures/ligature-fonts/UbuntuMono-Regular.ttf");
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
    if let Some((tag, position, invalid_compression)) = duplicate(change) {
        let data = if invalid_compression {
            (1..=16).collect()
        } else {
            Vec::new()
        };
        let record = (tag.as_bytes().try_into().unwrap(), data);
        if position == "first" {
            records.insert(0, record);
        } else {
            records.push(record);
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
    if let Some((_, position, invalid_compression)) = duplicate(change) {
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
        if !invalid_compression {
            output[offset..offset + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes());
        }
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
