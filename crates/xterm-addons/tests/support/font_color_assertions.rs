#[path = "font_metadata_data.rs"]
mod corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions};

fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
fn layout(bytes: &[u8]) -> (usize, usize, usize, usize) {
    if &bytes[..4] == b"wOFF" {
        (44, 20, word(bytes, 12), 4)
    } else {
        (12, 16, word(bytes, 4), 8)
    }
}
fn reorder(bytes: &mut [u8], case: &Value) {
    if let Some(tags) = case["frontTags"].as_array() {
        let (start, width, count, _) = layout(bytes);
        let mut records: Vec<_> = bytes[start..start + count * width]
            .chunks_exact(width)
            .map(<[u8]>::to_vec)
            .collect();
        records.sort_by_key(|r| {
            !tags
                .iter()
                .any(|tag| tag.as_str().unwrap().as_bytes() == &r[..4])
        });
        for (index, record) in records.iter().enumerate() {
            bytes[start + index * width..start + (index + 1) * width].copy_from_slice(record);
        }
    }
}
fn mutate(bytes: &mut [u8], case: &Value) {
    let action = case["mutation"].as_str().unwrap();
    let color = case["mutationTag"].as_str().unwrap();
    let target = match action {
        "payload" | "descriptor" => color,
        "next" if color == "CPAL" => "COLR",
        "next" => "CPAL",
        "head" | "headDescriptor" => "head",
        _ => "cmap",
    };
    let (start, width, count, field) = layout(bytes);
    let record = (0..count)
        .map(|i| start + i * width)
        .find(|at| &bytes[*at..*at + 4] == target.as_bytes())
        .unwrap();
    if matches!(action, "descriptor" | "headDescriptor") {
        bytes[record + field..record + field + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes());
    } else {
        let at = offset(bytes, record + field);
        if action == "head" {
            bytes[at + 12..at + 16].fill(0);
        } else {
            let at = at
                + if action == "payload" {
                    if color == "CPAL" {
                        4
                    } else {
                        2
                    }
                } else {
                    0
                };
            let value: u16 = if action == "repairCmap" { 0 } else { 65535 };
            bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
        }
    }
}
fn expected_error(case: &Value) -> Option<FontLoadError<()>> {
    match case["outcome"]["status"].as_str().unwrap() {
        "ok" => None,
        "warning" => Some(FontLoadError::Warning(())),
        _ => Some(FontLoadError::Font(
            match case["outcome"]["message"].as_str().unwrap() {
                "Invalid array length" => FontError::ArrayLength,
                "cmap table version should be 0." => {
                    FontError::Invalid("cmap table version should be 0.")
                }
                "Font header has wrong magic number." => {
                    FontError::Invalid("Font header has wrong magic number.")
                }
                _ => corpus::constructor_error(case).unwrap(),
            },
        )),
    }
}
pub fn assert_corpus() {
    let fixture: Value = serde_json::from_str(include_str!("../fixtures/font-color.json")).unwrap();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let mut bytes = if case["woff"] == true {
            corpus::construct_with_woff_payloads(case, Some(&fixture["woffTemplate"]))
        } else {
            corpus::construct(case)
        };
        reorder(&mut bytes, case);
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
        let mut mutated = false;
        let actual =
            Font::load_buffer_with_refresh(&bytes.clone(), FontOptions::default(), |diagnostic| {
                warnings.push(diagnostic.message().to_string());
                if case["mutation"].is_string() && !mutated {
                    mutated = true;
                    mutate(&mut bytes, case);
                }
                if case["throws"].is_string() {
                    Err(())
                } else {
                    Ok(Some(bytes.clone()))
                }
            })
            .err();
        let expected = expected_error(case);
        let expected_warnings: Vec<_> = case["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        if actual != expected || warnings != expected_warnings {
            failures.push(format!(
                "{}: expected {expected:?} {expected_warnings:?}, got {actual:?} {warnings:?}",
                case["name"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
