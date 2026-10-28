// Copyright (c) 2021 Joerg Breitbart; xterm.js authors. MIT license.
use super::*;
use serde_json::{json, Value};

fn number(value: &Value) -> f64 {
    match value.as_str().unwrap() {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        text => text.parse().unwrap(),
    }
}

fn properties(decoder: &SixelDecoder) -> Value {
    let p = decoder.properties().unwrap();
    json!({ "width": p.width, "height": p.height, "mode": p.mode as u8,
        "level": p.level as u8, "truncate": p.truncate, "paletteLimit": p.palette_limit,
        "fillColor": p.fill_color, "memUsage": p.memory_usage,
        "rasterAttributes": { "numerator": p.raster.numerator,
            "denominator": p.raster.denominator, "width": p.raster.width, "height": p.raster.height } })
}

fn error(error: DecodeError, phase: &str) -> Value {
    json!({ "phase": phase, "name": error.js_name(), "message": error.to_string() })
}

fn runs(pixels: &[u32]) -> Value {
    let mut result: Vec<(u32, usize)> = Vec::new();
    for &pixel in pixels {
        if let Some(last) = result.last_mut() {
            if last.0 == pixel {
                last.1 += 1;
                continue;
            }
        }
        result.push((pixel, 1));
    }
    serde_json::to_value(result).unwrap()
}

fn sparse_palette(value: &Value) -> Vec<u32> {
    let mut result = vec![0; PALETTE_SIZE];
    for pair in value.as_array().unwrap() {
        let index = usize::try_from(pair[0].as_u64().unwrap()).unwrap();
        result[index] = u32::try_from(pair[1].as_u64().unwrap()).unwrap();
    }
    result
}

#[test]
fn actual_dependency_pixels_properties_and_palette_match() {
    let mut cases = 0;
    let mut snapshots = 0;
    for line in include_str!("../../../tests/fixtures/sixel-oracle.ndjson").lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        let name = row["name"].as_str().unwrap();
        let config = &row["config"];
        let options = DecoderOptions {
            memory_limit: if config["memoryLimit"].is_null() {
                None
            } else {
                Some(number(&config["memoryLimit"]))
            },
            palette_limit: number(
                config
                    .get("constructorPaletteLimit")
                    .unwrap_or(&config["paletteLimit"]),
            ),
            palette: if config["addonPalette"].as_bool().unwrap() {
                ADDON_PALETTE.to_vec()
            } else {
                super::colors::VT340.to_vec()
            },
            ..DecoderOptions::default()
        };
        let actual = SixelDecoder::new(options);
        if !row["constructorError"].is_null() {
            let Err(failure) = actual else {
                panic!("constructor accepted {name}")
            };
            assert_eq!(
                error(failure, "constructor"),
                row["constructorError"],
                "{name}"
            );
            cases += 1;
            continue;
        }
        let mut decoder = actual.unwrap_or_else(|e| panic!("constructor {name}: {e}"));
        decoder.init(
            u32::try_from(config["fill"].as_u64().unwrap()).unwrap(),
            None,
            number(&config["paletteLimit"]),
            config["truncate"].as_bool().unwrap(),
        );
        let palette_basis = sparse_palette(&row["initialPalette"]);
        assert_eq!(decoder.palette, palette_basis, "initial palette {name}");
        let expected = row["snapshots"].as_array().unwrap();
        let mut next = 0;
        for (index, input) in row["chunks"].as_array().unwrap().iter().enumerate() {
            let input: Vec<u32> = input
                .as_array()
                .unwrap()
                .iter()
                .map(|v| u32::try_from(v.as_u64().unwrap()).unwrap())
                .collect();
            let mut failure = Value::Null;
            if let Err(e) = decoder.decode(&input) {
                failure = error(e, "decode");
                decoder.release();
            }
            if next >= expected.len() || expected[next]["chunk"].as_u64().unwrap() as usize != index
            {
                assert!(
                    failure.is_null(),
                    "unexpected unobserved error {name} chunk {index}: {failure}"
                );
                continue;
            }
            let before = properties(&decoder);
            let mut pixels = Vec::new();
            if failure.is_null() {
                match decoder.pixels() {
                    Ok(data) => pixels = data,
                    Err(e) => {
                        failure = error(e, "pixels");
                        decoder.release();
                    }
                }
            }
            let expected_frame = &expected[next];
            assert_eq!(
                failure, expected_frame["error"],
                "{name} chunk {index}, input {input:?}"
            );
            assert_eq!(
                before, expected_frame["before"],
                "before {name} chunk {index}"
            );
            let after = if expected_frame["after"].is_null() {
                &expected_frame["before"]
            } else {
                &expected_frame["after"]
            };
            assert_eq!(properties(&decoder), *after, "after {name} chunk {index}");
            assert_eq!(
                runs(&pixels),
                expected_frame["pixels"],
                "RGBA words {name} chunk {index}"
            );
            let mut palette = palette_basis.clone();
            for pair in expected_frame["palette"].as_array().unwrap() {
                palette[usize::try_from(pair[0].as_u64().unwrap()).unwrap()] =
                    u32::try_from(pair[1].as_u64().unwrap()).unwrap();
            }
            assert_eq!(
                decoder.palette, palette,
                "all4096 palette slots {name} chunk {index}"
            );
            assert_eq!(
                decoder.palette().len() as u64,
                expected_frame["paletteLength"].as_u64().unwrap(),
                "palette length {name} chunk {index}"
            );
            snapshots += 1;
            next += 1;
            if !failure.is_null() {
                break;
            }
        }
        assert_eq!(next, expected.len(), "missed snapshots {name}");
        cases += 1;
    }
    assert_eq!(cases, 676);
    assert_eq!(snapshots, 7015);
}
