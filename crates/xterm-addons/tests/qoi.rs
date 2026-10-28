// Copyright (c) 2023 The xterm.js authors. MIT license.
use serde_json::{json, Value};
use xterm_addons::image::qoi::{QoiDecoder, QoiError};
fn image(width: i32, height: i32, body: &[u8]) -> Vec<u8> {
    let mut data = vec![0; 14 + body.len() + 8];
    data[..4].copy_from_slice(b"qoif");
    data[4..8].copy_from_slice(&width.to_be_bytes());
    data[8..12].copy_from_slice(&height.to_be_bytes());
    data[12] = 4;
    data[14..14 + body.len()].copy_from_slice(body);
    *data.last_mut().unwrap() = 1;
    data
}
fn runs(data: &[u8]) -> Vec<(u32, usize)> {
    let mut result = Vec::new();
    for pixel in data.chunks_exact(4) {
        let word = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
        match result.last_mut() {
            Some((last, count)) if *last == word => *count += 1,
            _ => result.push((word, 1)),
        }
    }
    result
}
#[test]
fn all_source_wasm_pixel_and_memory_observations_match() {
    // Generated source-derived replay intentionally includes malformed input and
    // reused destination memory, not only valid QOI specification examples.
    let fixture: Value = serde_json::from_str(include_str!("fixtures/qoi-source.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    let mut decoder = QoiDecoder::new(4194304.0);
    for (index, case) in cases.iter().enumerate() {
        let operation = &case["operation"];
        let mut pixels = vec![];
        let mut error = None;
        match operation["op"].as_str().unwrap() {
            "new" => decoder = QoiDecoder::new(operation["keep"].as_f64().unwrap()),
            "release" => decoder.release(),
            "decode" => {
                let data = match operation["file"].as_str() {
                    Some("dice.qoi") => include_bytes!("fixtures/image-metrics/dice.qoi").to_vec(),
                    Some("palette.qoi") => include_bytes!("fixtures/qoi/palette.qoi").to_vec(),
                    None => operation["data"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                        .collect(),
                    Some(other) => panic!("unexpected fixture {other}"),
                };
                match decoder.decode(&data) {
                    Ok(decoded) => pixels = runs(&decoded.pixels),
                    Err(e) => error = Some(e.js_name()),
                }
            }
            other => panic!("unexpected operation {other}"),
        }
        assert_eq!(
            json!({"width":decoder.width,"height":decoder.height,"memory":decoder.memory_usage(),"initialized":decoder.is_initialized(),"pixels":pixels,"error":error}),
            case["expected"],
            "source operation {index}"
        );
    }
    assert_eq!(cases.len(), 1329);
}
#[test]
fn source_palette_qoi_decodes_to_the_original_rgba_bytes() {
    let mut decoder = QoiDecoder::new(4194304.0);
    let decoded = decoder
        .decode(include_bytes!("fixtures/qoi/palette.qoi"))
        .unwrap();
    assert_eq!((decoded.width, decoded.height), (640, 80));
    assert_eq!(decoded.pixels, include_bytes!("fixtures/qoi/palette.rgba"));
    let raster = decoded.into_rgba().unwrap();
    assert_eq!((raster.width(), raster.height()), (640, 80));
}
#[test]
fn release_keeps_source_page_boundary_and_signed_dimensions() {
    let encoded = image(2, 1, &[255, 255, 0, 0, 255, 192]);
    let mut kept = QoiDecoder::new(65536.0);
    let first = kept.decode(&encoded).unwrap();
    kept.release();
    assert!(kept.is_initialized());
    let partial = kept.decode(&image(2, 1, &[])).unwrap();
    assert_eq!(partial.pixels, first.pixels);
    let mut released = QoiDecoder::new(65535.0);
    released.decode(&encoded).unwrap();
    released.release();
    assert!(!released.is_initialized());
    assert_eq!(released.memory_usage(), 0);
    assert_eq!((released.width, released.height), (2, 1));
    assert_eq!(
        released.decode(&image(2, 1, &[])).unwrap().pixels,
        vec![0; 8]
    );
}
#[test]
fn oversized_initial_and_growth_arguments_fail_before_allocating() {
    let huge = image(16_777_216, 16_777_216, &[0]);
    let mut decoder = QoiDecoder::new(4_194_304.0);
    assert!(matches!(
        decoder.decode(&huge),
        Err(QoiError::MemoryArgument)
    ));
    assert_eq!(decoder.memory_usage(), 0);
    decoder.decode(&image(1, 1, &[255, 1, 2, 3, 4])).unwrap();
    assert!(matches!(
        decoder.decode(&huge),
        Err(QoiError::MemoryArgument)
    ));
    assert_eq!(decoder.memory_usage(), 65_536);
}

#[test]
fn invalid_initial_pages_are_a_recoverable_type_error_not_a_panic() {
    let mut decoder = QoiDecoder::new(0.0);
    let error = decoder.decode(&image(-100000, 1, &[])).unwrap_err();
    assert!(matches!(error, QoiError::MemoryInitial));
    assert_eq!(error.js_name(), "TypeError");
    assert_eq!((decoder.width, decoder.height), (-100000, 1));
    assert!(!decoder.is_initialized());
    let link = decoder.decode(&image(-2100, 1, &[])).unwrap_err();
    assert!(matches!(link, QoiError::MemoryImport));
    assert_eq!(link.js_name(), "LinkError");
    assert_eq!(
        decoder
            .decode(&image(1, 1, &[255, 1, 2, 3, 4]))
            .unwrap()
            .pixels,
        [1, 2, 3, 4]
    );
}
