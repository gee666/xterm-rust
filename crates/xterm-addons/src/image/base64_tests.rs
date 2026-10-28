// Copyright (c) 2023, 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Exact original WASM payload decoder differential traces.

use super::{DecodeStatus, PayloadDecoder};

fn input(text: &str) -> Vec<u32> {
    text.split(',')
        .filter(|part| !part.is_empty())
        .map(|part| u32::from_str_radix(part, 16).unwrap())
        .collect()
}
fn bytes(text: &str) -> Vec<u8> {
    text.split(',')
        .filter(|part| !part.is_empty())
        .map(|part| u8::from_str_radix(part, 16).unwrap())
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &str, case: usize, step: usize) {
    if let Some(hex) = expected.strip_prefix("h:") {
        assert_eq!(actual, bytes(hex), "bytes case {case} step {step}");
    } else {
        let fields: Vec<_> = expected.split(':').collect();
        assert_eq!(fields[0], "r");
        let count: usize = fields[1].parse().unwrap();
        let pattern = bytes(fields[2]);
        assert_eq!(
            actual.len(),
            count * pattern.len(),
            "length case {case} step {step}"
        );
        for (offset, byte) in actual.iter().enumerate() {
            assert_eq!(
                *byte,
                pattern[offset % pattern.len()],
                "case {case} step {step} byte {offset}"
            );
        }
    }
}

#[test]
fn strict_payload_traces_match_original_decoder_status_and_every_byte() {
    let corpus = include_str!("../../tests/fixtures/image-base64.tsv");
    assert_eq!(corpus.lines().count(), 270);
    for (case, row) in corpus.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        let mut decoder = PayloadDecoder::new(
            fields[0].parse().unwrap(),
            fields[1].parse().unwrap(),
            fields[2].parse().unwrap(),
        )
        .unwrap();
        let expected: Vec<_> = fields[4].split('~').collect();
        let operations: Vec<_> = fields[3].split('|').collect();
        assert_eq!(expected.len(), operations.len());
        for (step, operation) in operations.into_iter().enumerate() {
            let parts: Vec<_> = operation.split(':').collect();
            let status = match parts[0] {
                "i" => {
                    decoder.init(None, None).unwrap();
                    "i".to_owned()
                }
                "r" => {
                    decoder.release();
                    "r".to_owned()
                }
                "e" => decoder.end().unwrap().code().to_string(),
                "p" => decoder.put(&input(parts[1])).unwrap().code().to_string(),
                "q" => {
                    let count: usize = parts[1].parse().unwrap();
                    let pattern = input(parts[2]);
                    let data: Vec<_> = pattern
                        .iter()
                        .copied()
                        .cycle()
                        .take(count * pattern.len())
                        .collect();
                    decoder.put(&data).unwrap().code().to_string()
                }
                other => panic!("unknown fixture operation {other}"),
            };
            let snapshot: Vec<_> = expected[step].split('/').collect();
            assert_eq!(
                status, snapshot[0],
                "status seed 0xb64c0de case {case} step {step}"
            );
            assert_eq!(
                decoder.loaded_bytes().to_string(),
                snapshot[1],
                "loaded case {case} step {step}"
            );
            assert_eq!(
                decoder.free_bytes().to_string(),
                snapshot[2],
                "free case {case} step {step}"
            );
            assert_bytes(decoder.data(), snapshot[3], case, step);
        }
    }
}

#[test]
fn validation_is_deferred_and_codepoints_truncate_like_uint8array() {
    let mut decoder = PayloadDecoder::new(64, 64, 8).unwrap();
    assert_eq!(decoder.end().unwrap(), DecodeStatus::NotInitialized);
    decoder.init(None, None).unwrap();
    assert_eq!(decoder.put(&[0x10051, 0x10051]).unwrap(), DecodeStatus::Ok);
    assert!(decoder.data().is_empty());
    assert_eq!(decoder.end().unwrap(), DecodeStatus::Ok);
    assert_eq!(decoder.data(), [65]);
    assert_eq!(decoder.put(&[65]).unwrap(), DecodeStatus::NotInitialized);
    decoder.init(None, None).unwrap();
    assert_eq!(decoder.put(&[33]).unwrap(), DecodeStatus::Ok);
    assert_eq!(decoder.end().unwrap(), DecodeStatus::DecodeError);
}

#[test]
fn invalid_settings_and_zero_initial_size_are_explicit_safe_boundaries() {
    assert!(PayloadDecoder::new(64, 4, 8).is_err());
    let mut decoder = PayloadDecoder::new(64, 4, 0).unwrap();
    decoder.init(None, None).unwrap();
    assert_eq!(decoder.put(&[81, 81]).unwrap(), DecodeStatus::Ok);
    assert_eq!(decoder.end().unwrap(), DecodeStatus::Ok);
    assert_eq!(decoder.data(), [65]);
}
