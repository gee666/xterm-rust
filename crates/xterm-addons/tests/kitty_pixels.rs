// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Compare original Chromium ImageData inputs, before bitmap premultiplication.
use serde_json::Value;
use xterm_addons::image::kitty::pixels::{raw_pixels, PixelError, RawFormat};
fn f(value: &Value) -> f64 {
    f64::from_bits(u64::from_str_radix(value.as_str().unwrap(), 16).unwrap())
}

#[test]
fn raw_rgb_rgba_and_numeric_boundaries_match_original_chromium_inputs() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/kitty-pixels.json")).unwrap();
    assert_eq!(rows.len(), 182);
    for (index, row) in rows.iter().enumerate() {
        let format = if f(&row["format"]) == 32.0 {
            RawFormat::Rgba
        } else {
            RawFormat::Rgb
        };
        let input: Vec<_> = row["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
            .collect();
        match raw_pixels(&input, f(&row["width"]), f(&row["height"]), format) {
            Ok(image) => {
                assert!(
                    row["accepted"].as_bool().unwrap(),
                    "unexpected acceptance {index}"
                );
                assert_eq!(
                    u64::from(image.width()),
                    row["pixels"]["width"].as_u64().unwrap(),
                    "width {index}"
                );
                assert_eq!(
                    u64::from(image.height()),
                    row["pixels"]["height"].as_u64().unwrap(),
                    "height {index}"
                );
                let expected: Vec<_> = row["pixels"]["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
                    .collect();
                assert_eq!(image.pixels(), expected, "pixels {index}");
            }
            Err(error) => {
                assert!(
                    !row["accepted"].as_bool().unwrap(),
                    "unexpected rejection {index}: {error}"
                );
                match error {
                    PixelError::MissingDimensions | PixelError::InsufficientData => assert_eq!(
                        error.to_string(),
                        row["error"]["message"].as_str().unwrap(),
                        "source error {index}"
                    ),
                    PixelError::InvalidDimensions => assert_ne!(
                        row["error"]["name"], "Error",
                        "typed WebIDL/ImageData rejection {index}"
                    ),
                    other => panic!("unexpected native resource error {index}: {other}"),
                }
            }
        }
    }
}
