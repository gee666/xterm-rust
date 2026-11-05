// Shared native/WASM regression for warning writes to the final META table.
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions};

pub fn load(case: &Value) -> Result<Font, FontError> {
    let mut bytes = super::corpus::construct(case);
    let Some(version) = case["metaMutation"].as_u64() else {
        return Font::load_buffer(&bytes, FontOptions::default());
    };
    let snapshot = bytes.clone();
    let mut calls = 0;
    let result = Font::load_buffer_with_refresh(&snapshot, FontOptions::default(), |_| {
        calls += 1;
        let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
        for index in 0..usize::from(count) {
            let record = 12 + index * 16;
            if &bytes[record..record + 4] == b"meta" {
                let start = usize::try_from(u32::from_be_bytes(
                    bytes[record + 8..record + 12].try_into().unwrap(),
                ))
                .unwrap();
                bytes[start..start + 4]
                    .copy_from_slice(&u32::try_from(version).unwrap().to_be_bytes());
            }
        }
        Ok::<_, std::convert::Infallible>(Some(bytes.clone()))
    });
    assert_eq!(calls, 1, "{}", case["name"]);
    match result {
        Ok(font) => Ok(font),
        Err(FontLoadError::Font(error)) => Err(error),
        Err(FontLoadError::Warning(never)) => match never {},
    }
}
