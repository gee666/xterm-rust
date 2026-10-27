// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Assertions ported from TextDecoder.test.ts.
use xterm_core::text_decoder::{
    string_from_code_point, utf32_to_string, StringToUtf32, Utf8ToUtf32,
};

#[test]
fn string_from_code_point_and_utf32_to_string() {
    let data: Vec<u32> = "abcdefg".chars().map(u32::from).collect();
    for &value in &data {
        assert_eq!(string_from_code_point(value), vec![value as u16]);
    }
    assert_eq!(
        utf32_to_string(&data),
        "abcdefg".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn full_codepoint_ranges() {
    // Same 8192-value batches and exclusions as upstream. Include U+10FFFF
    // additionally, which upstream's exclusive upper endpoint does not reach.
    for min in (0..=0x10ffff).step_by(8192) {
        let max = (min + 8192).min(0x110000);
        let expected: Vec<u32> = (min..max)
            .filter(|&cp| !(0xd800..=0xdfff).contains(&cp) && cp != 0xfeff)
            .collect();
        let input = utf32_to_string(&expected);
        let mut output = Vec::new();
        assert_eq!(
            StringToUtf32::new().decode(&input, &mut output),
            expected.len()
        );
        assert_eq!(output, expected);
        assert_eq!(utf32_to_string(&output), input);
        let text = String::from_utf16(&input).unwrap();
        output.clear();
        assert_eq!(
            Utf8ToUtf32::new().decode(text.as_bytes(), &mut output),
            expected.len()
        );
        assert_eq!(output, expected);
        assert_eq!(utf32_to_string(&output), input);
    }
}

#[test]
fn exact_upstream_multilingual_samples() {
    let mut utf8 = Utf8ToUtf32::new();
    let mut utf16 = StringToUtf32::new();
    for text in include_str!("fixtures/text-decoder-strings.txt").lines() {
        let expected: Vec<u32> = text.chars().map(u32::from).collect();
        let mut output = Vec::new();
        assert_eq!(utf8.decode(text.as_bytes(), &mut output), expected.len());
        assert_eq!(output, expected);
        output.clear();
        assert_eq!(
            utf16.decode(&text.encode_utf16().collect::<Vec<_>>(), &mut output),
            expected.len()
        );
        assert_eq!(output, expected);
        utf8.clear();
        utf16.clear();
    }
}

#[test]
fn streaming_upstream_cases() {
    for text in [
        "ÄÖÜßöäü",
        "Ä€Ö€Ü€ß€ö€ä€ü",
        "Ä€𝄞Ö𝄞€Ü𝄞€",
        "\u{feff}\u{feff}",
        "A—BA—BA—BA—BA—B",
        "A𐀀BA𐀀BA𐀀BA𐀀BA𐀀B",
    ] {
        let expected: Vec<u32> = text
            .chars()
            .filter(|&ch| ch != '\u{feff}')
            .map(u32::from)
            .collect();
        for step in 1..=3 {
            let mut decoder = Utf8ToUtf32::new();
            let mut output = Vec::new();
            for chunk in text.as_bytes().chunks(step) {
                decoder.decode(chunk, &mut output);
            }
            assert_eq!(output, expected, "text {text:?}, chunk {step}");
        }
        let mut decoder = StringToUtf32::new();
        let mut output = Vec::new();
        for unit in text.encode_utf16() {
            decoder.decode(&[unit], &mut output);
        }
        assert_eq!(output, expected);
    }
    let mut decoder = Utf8ToUtf32::new();
    let mut output = Vec::new();
    assert_eq!(decoder.decode(&[0xf0, 0xa0, 0x9c], &mut output), 0);
    assert_eq!(decoder.decode(&[0x8e], &mut output), 1);
    assert_eq!(output, vec![0x2070e]);
}

#[test]
fn bom_and_clear() {
    let mut utf8 = Utf8ToUtf32::new();
    let mut output = Vec::new();
    assert_eq!(utf8.decode(&[0xef, 0xbb, 0xbf], &mut output), 0);
    utf8.decode(&[0xf0, 0xa0], &mut output);
    assert_eq!(utf8.decode(&[], &mut output), 0);
    assert_eq!(utf8.interim, [0xf0, 0xa0, 0]);
    utf8.clear();
    utf8.decode(&[0x9c, 0x8e, b'A'], &mut output);
    assert_eq!(output, vec![0x41]);
    let mut utf16 = StringToUtf32::new();
    output.clear();
    assert_eq!(utf16.decode(&[0xfeff], &mut output), 0);
    assert_eq!(utf16.decode(&[0xd800], &mut output), 0);
    assert_eq!(utf16.decode(&[], &mut output), 0);
    utf16.clear();
    utf16.decode(&[0xdc00], &mut output);
    assert_eq!(output, vec![0xdc00]);
}

fn parse_hex(text: &str) -> Vec<u32> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(',')
            .map(|value| u32::from_str_radix(value, 16).unwrap())
            .collect()
    }
}

#[test]
fn js_oracle_chunked_malformed_traces() {
    let fixtures = include_str!("fixtures/text-decoder-traces.tsv");
    for (case, line) in fixtures.lines().enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 4);
        let input: Vec<&str> = fields[1].split(';').collect();
        let expected: Vec<&str> = fields[2].split(';').collect();
        assert_eq!(input.len(), expected.len());
        let mut utf8 = Utf8ToUtf32::new();
        let mut utf16 = StringToUtf32::new();
        let states: Vec<&str> = fields[3].split(';').collect();
        for (chunk, text) in input.iter().enumerate() {
            let values = parse_hex(text);
            let expected = parse_hex(expected[chunk]);
            // Existing output must remain untouched, not count toward written.
            let mut output = vec![0xdeadbeef];
            let written = if fields[0] == "8" {
                let bytes: Vec<u8> = values
                    .iter()
                    .map(|&value| u8::try_from(value).unwrap())
                    .collect();
                let written = utf8.decode(&bytes, &mut output);
                assert_eq!(
                    utf8.interim.map(u32::from).to_vec(),
                    parse_hex(states[chunk]),
                    "case {case}, chunk {chunk}, interim"
                );
                written
            } else {
                let units: Vec<u16> = values
                    .iter()
                    .map(|&value| u16::try_from(value).unwrap())
                    .collect();
                utf16.decode(&units, &mut output)
            };
            assert_eq!(written, expected.len(), "case {case}, chunk {chunk}");
            assert_eq!(&output[1..], expected, "case {case}, chunk {chunk}");
            assert_eq!(output[0], 0xdeadbeef);
        }
    }
}
