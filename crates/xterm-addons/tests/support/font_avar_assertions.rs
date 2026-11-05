#[path = "font_metadata_data.rs"]
mod corpus;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions};

fn mutate(bytes: &mut [u8], item: &serde_json::Value) {
    let Some(tag) = item["tag"].as_str() else {
        return;
    };
    let woff = &bytes[..4] == b"wOFF";
    let (start, width, count_at, offset_at) = if woff {
        (44, 20, 12, 4)
    } else {
        (12, 16, 4, 8)
    };
    let count = usize::from(u16::from_be_bytes(
        bytes[count_at..count_at + 2].try_into().unwrap(),
    ));
    for index in 0..count {
        let at = start + index * width;
        if &bytes[at..at + 4] != tag.as_bytes() {
            continue;
        }
        let offset = at + offset_at;
        if item["descriptor"] == true {
            bytes[offset..offset + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes());
        } else {
            let payload = usize::try_from(u32::from_be_bytes(
                bytes[offset..offset + 4].try_into().unwrap(),
            ))
            .unwrap();
            let at = payload + usize::try_from(item["at"].as_u64().unwrap()).unwrap();
            bytes[at..at + 2].copy_from_slice(
                &u16::try_from(item["value"].as_u64().unwrap())
                    .unwrap()
                    .to_be_bytes(),
            );
        }
    }
}
fn expected_error(case: &serde_json::Value) -> Option<FontError> {
    match case["outcome"]["message"].as_str() {
        Some("Data error") => Some(FontError::Compression),
        Some("avar axis count must correspond to fvar axis count") => Some(FontError::Invalid(
            "avar axis count must correspond to fvar axis count",
        )),
        Some("Cannot read properties of undefined (reading 'axes')") => Some(FontError::Type(
            "Cannot read properties of undefined (reading 'axes')",
        )),
        Some("Unsupported fvar table version.") => {
            Some(FontError::Invalid("Unsupported fvar table version."))
        }
        _ => corpus::constructor_error(case),
    }
}
pub fn assert_corpus() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/font-avar.json")).unwrap();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let bytes = if case["woff"] == true {
            corpus::construct_with_woff_payloads(case, Some(&fixture["woffTemplate"]))
        } else {
            corpus::construct(case)
        };
        let hash = bytes.iter().fold(0x811c9dc5_u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(0x01000193)
        });
        assert_eq!(
            u64::from(hash),
            case["inputHash"].as_u64().unwrap(),
            "{}",
            case["name"]
        );
        let mut live = bytes.clone();
        let mut warnings = Vec::new();
        let actual = Font::load_buffer_with_refresh(&bytes, FontOptions::default(), |warning| {
            warnings.push(warning.message().into_owned());
            mutate(&mut live, &case["mutation"]);
            if case["throws"].is_string() {
                Err(())
            } else {
                Ok(case["mutation"].is_object().then(|| live.clone()))
            }
        })
        .err();
        let expected = if case["outcome"]["status"] == "warning" {
            Some(FontLoadError::Warning(()))
        } else {
            expected_error(case).map(FontLoadError::Font)
        };
        let expected_warnings: Vec<&str> = case["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        if actual != expected || warnings != expected_warnings {
            failures.push(format!(
                "{}: expected {expected:?}/{expected_warnings:?}, got {actual:?}/{warnings:?}",
                case["name"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
