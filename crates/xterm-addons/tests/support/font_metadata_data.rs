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
    construct_with_woff_payloads(change, None)
}

// Shared encoded payloads keep source/Rust diagnostic offsets byte-identical.
// Existing corpora still use their original independently compressed inputs.
pub fn construct_with_woff_payloads(change: &Value, payloads: Option<&Value>) -> Vec<u8> {
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
        let mut data = base[start..start + offset(base, at + 12)].to_vec();
        if tag == "hhea" {
            if let Some(metrics) = change["metrics"].as_u64() {
                data[34..36].copy_from_slice(&u16::try_from(metrics).unwrap().to_be_bytes());
            }
        }
        if tag == "maxp" {
            if let Some(count) = change["glyphCount"].as_u64() {
                data[4..6].copy_from_slice(&u16::try_from(count).unwrap().to_be_bytes());
            }
        }
        records.push((base[at..at + 4].try_into().unwrap(), data));
    }
    if change["metaMutation"].as_u64().is_some() {
        let mut data = vec![0; 18];
        data[..2].copy_from_slice(&1_u16.to_be_bytes());
        data[4..8].copy_from_slice(&2_u32.to_be_bytes());
        records.push((*b"kern", data));
    }
    if let Some(tables) = change["extraTables"].as_array() {
        for table in tables {
            records.push((
                table["tag"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
                    .try_into()
                    .unwrap(),
                table["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                    .collect(),
            ));
        }
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
        output = woff(&output, change, payloads);
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
    if let Some(length) = change["hmtxDirectoryLength"].as_u64() {
        let at = if compressed {
            44 + (records.len() - 1) * 20
        } else {
            12 + (records.len() - 1) * 16
        };
        output[at + 12..at + 16].copy_from_slice(&u32::try_from(length).unwrap().to_be_bytes());
    }
    if change["invalidHmtxOffset"] == true {
        let at = if compressed {
            44 + (records.len() - 1) * 20
        } else {
            12 + (records.len() - 1) * 16
        };
        let offset = at + if compressed { 4 } else { 8 };
        output[offset..offset + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes());
        if change["compressedHmtx"] == true {
            output[at + 8..at + 12].copy_from_slice(&16_u32.to_be_bytes());
            output[at + 12..at + 16].copy_from_slice(&128_u32.to_be_bytes());
        }
    }
    output
}

fn woff(bytes: &[u8], change: &Value, payloads: Option<&Value>) -> Vec<u8> {
    let count = word(bytes, 4);
    let mut output = vec![0; 44 + count * 20];
    output[..4].copy_from_slice(b"wOFF");
    output[4..8].copy_from_slice(&bytes[..4]);
    output[12..14].copy_from_slice(&u16::try_from(count).unwrap().to_be_bytes());
    for index in 0..count {
        let at = 12 + index * 16;
        let start = offset(bytes, at + 8);
        let data = &bytes[start..start + offset(bytes, at + 12)];
        let tag = std::str::from_utf8(&bytes[at..at + 4]).unwrap();
        let encoded = change["woffPayloads"]
            .get(tag)
            .or_else(|| payloads.and_then(|v| v.get(tag)));
        let compressed = encoded.map_or_else(
            || miniz_oxide::deflate::compress_to_vec_zlib(data, 6),
            |value| {
                value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                    .collect()
            },
        );
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

// Native error types deliberately represent source DataView and missing-table
// errors. Compare the actual failure cause, not merely accepted versus rejected.
pub fn constructor_error(case: &Value) -> Option<xterm_addons::ligatures::font::FontError> {
    use xterm_addons::ligatures::font::FontError;
    if case["outcome"]["status"] == "ok" {
        return None;
    }
    // The native WOFF boundary uses checked zlib decompression. The source's
    // permissive inflater turns this corrupt selected payload into a bad META
    // version instead. Both reject loading, but the typed failure differs.
    if case["name"] == "selected meta invalid compressed data" {
        return Some(FontError::Compression);
    }
    let message = case["outcome"]["message"].as_str().unwrap();
    Some(match message {
        "Offset is outside the bounds of the DataView" => FontError::Bounds,
        "Unsupported META table version." => FontError::Invalid("Unsupported META table version."),
        "Unsupported kern sub-table version." => {
            FontError::Invalid("Unsupported kern sub-table version.")
        }
        "Cannot read properties of undefined (reading 'addUnicode')" => {
            FontError::Type("Cannot read properties of undefined (reading 'addUnicode')")
        }
        "Cannot read properties of undefined (reading 'compression')" => {
            FontError::MissingTable(if case["remove"][0] == "name" {
                "name"
            } else {
                "hmtx"
            })
        }
        value if value.starts_with("Invalid typed array length:") => FontError::Bounds,
        value if value.starts_with("Unsupported kern table version (") => FontError::KernVersion(
            value
                .strip_prefix("Unsupported kern table version (")
                .unwrap()
                .strip_suffix(").")
                .unwrap()
                .parse()
                .unwrap(),
        ),
        value => panic!("Unmapped source constructor error: {value}"),
    })
}
