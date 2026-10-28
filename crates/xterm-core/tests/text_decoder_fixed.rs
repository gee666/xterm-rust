// Copyright (c) 2019 The xterm.js authors. MIT license.
use xterm_core::text_decoder::{StringToUtf32, Utf8ToUtf32};
const SENTINEL: u32 = 0xdead_beef;

fn hex(text: &str) -> Vec<u32> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(',')
            .map(|v| u32::from_str_radix(v, 16).unwrap())
            .collect()
    }
}

#[test]
fn bounded_targets_match_all_original_chunked_traces() {
    let mut chunks = 0;
    for (case, line) in include_str!("fixtures/text-decoder-traces.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = line.split('\t').collect();
        let input: Vec<_> = fields[1].split(';').collect();
        let outputs: Vec<_> = fields[2].split(';').collect();
        let states: Vec<_> = fields[3].split(';').collect();
        assert_eq!(fields.len(), 4);
        assert_eq!(input.len(), outputs.len());
        if fields[0] == "8" {
            assert_eq!(states.len(), input.len());
        }
        for capacity in [0, 1, 2, 5, 13] {
            let mut utf8 = Utf8ToUtf32::new();
            let mut utf16 = StringToUtf32::new();
            for (step, (input, output)) in input.iter().zip(&outputs).enumerate() {
                let input = hex(input);
                let expected = hex(output);
                let mut target = vec![SENTINEL; capacity];
                let count = if fields[0] == "8" {
                    let input: Vec<u8> = input.iter().map(|v| u8::try_from(*v).unwrap()).collect();
                    let count = utf8.decode_to_slice(&input, &mut target);
                    assert_eq!(
                        utf8.interim.map(u32::from).to_vec(),
                        hex(states[step]),
                        "interim case{case} step{step} cap{capacity}"
                    );
                    count
                } else {
                    let input: Vec<u16> =
                        input.iter().map(|v| u16::try_from(*v).unwrap()).collect();
                    utf16.decode_to_slice(&input, &mut target)
                };
                assert_eq!(
                    count,
                    expected.len(),
                    "logical count case{case} step{step} cap{capacity}"
                );
                for (index, value) in target.iter().enumerate() {
                    assert_eq!(
                        *value,
                        expected.get(index).copied().unwrap_or(SENTINEL),
                        "target case{case} step{step} cap{capacity} index{index}"
                    );
                }
                chunks += 1;
            }
        }
    }
    assert_eq!(chunks, 66_350);
}

#[test]
fn pending_surrogate_overflow_is_counted_without_out_of_bounds_writes() {
    let mut decoder = StringToUtf32::new();
    assert_eq!(decoder.decode_to_slice(&[0xd800], &mut []), 0);
    let mut target = [SENTINEL];
    assert_eq!(decoder.decode_to_slice(&[0x41], &mut target), 2);
    assert_eq!(target, [0xd800]);
    let mut target = [SENTINEL; 3];
    assert_eq!(decoder.decode_to_slice(&[0x42], &mut target), 1);
    assert_eq!(target, [0x42, SENTINEL, SENTINEL]);
    decoder.decode_to_slice(&[0xdbff], &mut []);
    decoder.clear();
    assert_eq!(decoder.decode_to_slice(&[0xdc00], &mut target), 1);
    assert_eq!(target[0], 0xdc00);
}

#[test]
fn public_interim_mutations_preserve_full_fold_sparse_tail_and_retry() {
    let mut decoder = Utf8ToUtf32::new();
    decoder.interim = [0xc2, 0x80, 0x80];
    let mut vector = Vec::new();
    assert_eq!(decoder.decode(b"A", &mut vector), 2);
    assert_eq!(vector, [0x2000, 0x41]);
    decoder.interim = [0xe1, 0, 0x80];
    assert_eq!(decoder.decode_to_slice(&[0x80], &mut []), 0);
    assert_eq!(decoder.interim, [0xe1, 0x80, 0x80]);
    let mut target = [SENTINEL; 3];
    assert_eq!(decoder.decode_to_slice(b"A", &mut target), 2);
    assert_eq!(target, [0x1000, 0x41, SENTINEL]);
    decoder.interim = [0xc0, 0x80, 0];
    let mut target = [SENTINEL; 2];
    assert_eq!(decoder.decode_to_slice(b"abcd", &mut target), 5);
    assert_eq!(target, [0, u32::from(b'a')]);
    decoder.interim = [0xc2, 0x80, 0x80];
    assert_eq!(decoder.decode_to_slice(&[], &mut []), 0);
    assert_eq!(decoder.interim, [0xc2, 0x80, 0x80]);
}

#[test]
fn bounded_utf8_resumes_prefix_and_clear_discards_it() {
    let mut decoder = Utf8ToUtf32::new();
    assert_eq!(decoder.decode_to_slice(&[0xf0, 0xa0, 0x9c], &mut []), 0);
    assert_eq!(decoder.decode_to_slice(&[], &mut []), 0);
    assert_eq!(decoder.decode_to_slice(&[0x8e, b'A'], &mut []), 2);
    assert_eq!(decoder.interim, [0; 3]);
    decoder.decode_to_slice(&[0xe2, 0x82], &mut []);
    decoder.clear();
    let mut target = [SENTINEL; 2];
    assert_eq!(decoder.decode_to_slice(&[0xac, b'B'], &mut target), 1);
    assert_eq!(target, [u32::from(b'B'), SENTINEL]);
}
