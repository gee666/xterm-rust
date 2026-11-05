use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions};

fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
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
pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff-header.json")).unwrap()
}
fn expected(case: &Value) -> Option<FontError> {
    if case["outcome"]["status"] == "ok" {
        return None;
    }
    let message = case["outcome"]["message"].as_str().unwrap();
    Some(
        if let Some(version) = message.strip_prefix("Unsupported CFF table version ") {
            let (major, minor) = version.split_once('.').unwrap();
            FontError::CffVersion {
                major: major.parse().unwrap(),
                minor: minor.parse().unwrap(),
            }
        } else if message == "Font doesn't contain TrueType, CFF or CFF2 outlines." {
            FontError::Invalid("Font doesn't contain TrueType, CFF or CFF2 outlines.")
        } else if message == "Data error" {
            FontError::Compression
        } else if message == "Cannot read properties of undefined (reading 'compression')" {
            FontError::MissingTable("name")
        } else {
            assert!(
                message == "Offset is outside the bounds of the DataView"
                    || message.starts_with("Invalid typed array length:"),
                "{message}"
            );
            FontError::Bounds
        },
    )
}
fn mutate(bytes: &mut [u8], case: &Value) {
    let woff = case["woff"] == true;
    let (start, width, count, field) = if woff {
        (44, 20, word(bytes, 12), 4)
    } else {
        (12, 16, word(bytes, 4), 8)
    };
    let at = (0..count)
        .map(|i| start + i * width)
        .filter(|at| &bytes[*at..*at + 4] == case["tag"].as_str().unwrap().as_bytes())
        .last()
        .unwrap();
    let off = offset(bytes, at + field);
    match case["mutation"].as_str().unwrap() {
        "descriptor" => {
            bytes[at + field..at + field + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes())
        }
        "major" => bytes[off] = 3,
        "repair" => bytes[off] = 1,
        "wrapper" => bytes[off] = 255,
        "throw" => {}
        value => panic!("Unknown mutation {value}"),
    }
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let mut bytes = construct(case, &fixture);
        let hash = bytes.iter().fold(2166136261_u32, |h, b| {
            (h ^ u32::from(*b)).wrapping_mul(16777619)
        });
        assert_eq!(
            u64::from(hash),
            case["inputFnv32"].as_u64().unwrap(),
            "input {}",
            case["name"]
        );
        let mut warnings = Vec::new();
        let initial = bytes.clone();
        let actual =
            Font::load_buffer_with_refresh(&initial, FontOptions::default(), |diagnostic| {
                warnings.push(serde_json::json!(["warn", diagnostic.message()]));
                let changed = case["mutation"].is_string();
                if changed {
                    mutate(&mut bytes, case);
                }
                if case["throws"].is_string() {
                    Err(())
                } else {
                    Ok(changed.then(|| bytes.clone()))
                }
            })
            .err();
        let expected = if case["outcome"]["status"] == "warning" {
            Some(FontLoadError::Warning(()))
        } else {
            expected(case).map(FontLoadError::Font)
        };
        if actual != expected || serde_json::json!(warnings) != case["warnings"] {
            failures.push(format!(
                "{} expected {expected:?} {}, got {actual:?} {warnings:?}",
                case["name"], case["warnings"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
