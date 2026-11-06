// Shared test-only SFNT/WOFF writer. Original constructor assertions stay in their suites.
use serde_json::Value;

pub(super) fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
pub(super) fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
fn array(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
        .collect()
}
struct Record {
    tag: String,
    data: Vec<u8>,
    bad_offset: bool,
}
fn record(tag: &str, data: Vec<u8>, bad_offset: bool) -> Record {
    Record {
        tag: tag.into(),
        data,
        bad_offset,
    }
}
pub fn construct(case: &Value, fixture: &Value) -> Vec<u8> {
    let base = if case["base"] == "ttf" {
        include_bytes!("../fixtures/ligature-fonts/UbuntuMono-Regular.ttf").as_slice()
    } else {
        include_bytes!("../fixtures/ligature-fonts/FiraCode-Regular.otf").as_slice()
    };
    let mut original = None;
    let mut records = Vec::new();
    for i in 0..word(base, 4) {
        let at = 12 + i * 16;
        let tag = std::str::from_utf8(&base[at..at + 4]).unwrap();
        let off = offset(base, at + 8);
        let data = base[off..off + offset(base, at + 12)].to_vec();
        if tag == "CFF " {
            original = Some(data.clone());
        }
        if matches!(tag, "CFF " | "CFF2")
            || case["remove"]
                .as_array()
                .is_some_and(|a| a.iter().any(|t| t == tag))
        {
            continue;
        }
        records.push(record(tag, data, false));
    }
    let tag = case["tag"].as_str().unwrap();
    let mut payload = if case["data"].is_array() {
        array(&case["data"])
    } else {
        original.unwrap()
    };
    if case["header"].is_array() {
        let header = array(&case["header"]);
        payload[..header.len()].copy_from_slice(&header);
    }
    if case["mutation"].is_string() {
        let mut color = vec![0; 14];
        color[1] = 1;
        records.insert(0, record("CPAL", color, false));
    }
    if case["duplicate"] == "first" {
        records.insert(0, record(tag, payload.clone(), true));
    }
    if case["other"].is_object() {
        let other = &case["other"];
        records.push(record(
            other["tag"].as_str().unwrap(),
            array(&other["data"]),
            other["badOffset"] == true,
        ));
    }
    records.push(record(tag, payload.clone(), case["badOffset"] == true));
    if case["duplicate"] == "last" {
        records.push(record(tag, payload, true));
    }
    if case["noCff"] == true {
        records.retain(|r| !matches!(r.tag.as_str(), "CFF " | "CFF2"));
    }
    let woff = case["woff"] == true;
    let (start, width) = if woff { (44, 20) } else { (12, 16) };
    let mut bytes = vec![0; start + records.len() * width];
    bytes[..4].copy_from_slice(if woff { b"wOFF" } else { &base[..4] });
    if woff {
        bytes[4..8].copy_from_slice(&base[..4]);
    }
    let count_at = if woff { 12 } else { 4 };
    bytes[count_at..count_at + 2]
        .copy_from_slice(&u16::try_from(records.len()).unwrap().to_be_bytes());
    for (i, r) in records.iter().enumerate() {
        let at = start + i * width;
        let base_key = if case["base"] == "ttf" { "ttf" } else { "otf" };
        let encoded = case["encoded"]
            .get(&r.tag)
            .or_else(|| fixture["woffTemplates"][base_key].get(&r.tag))
            .map(array);
        let payload = encoded
            .as_deref()
            .filter(|p| woff && p.len() < r.data.len())
            .unwrap_or(&r.data);
        bytes[at..at + 4].copy_from_slice(r.tag.as_bytes());
        let off = if r.bad_offset {
            0xfffffff0
        } else {
            u32::try_from(bytes.len()).unwrap()
        };
        let field = at + if woff { 4 } else { 8 };
        bytes[field..field + 4].copy_from_slice(&off.to_be_bytes());
        if woff {
            bytes[at + 8..at + 12]
                .copy_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
        }
        let length = if i + 1 == records.len() {
            case["directoryLength"]
                .as_u64()
                .map(|n| u32::try_from(n).unwrap())
        } else {
            None
        }
        .unwrap_or(u32::try_from(r.data.len()).unwrap());
        bytes[at + 12..at + 16].copy_from_slice(&length.to_be_bytes());
        bytes.extend_from_slice(payload);
    }
    if woff {
        let length = u32::try_from(bytes.len()).unwrap();
        bytes[8..12].copy_from_slice(&length.to_be_bytes());
    }
    bytes
}
