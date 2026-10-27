// Copyright (c) 2018-2024 The xterm.js authors. All rights reserved. MIT license.
use xterm_addons::{
    ligatures::{parse::parse_families, FallbackLigatures},
    web_fonts::{create_family, quote, relayout_plan, split_family, unquote, FontDescriptor},
};

fn utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

fn decode_hex(value: &str) -> Vec<u16> {
    if value == "-" {
        return Vec::new();
    }
    value
        .as_bytes()
        .chunks_exact(4)
        .map(|chunk| u16::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn complete_source_parser_assertions_and_seeded_utf16_traces() {
    let corpus = include_str!("fixtures/font-parse.tsv");
    let mut rows = 0;
    for (index, row) in corpus.lines().enumerate() {
        let (input, expected) = row.split_once('\t').unwrap();
        let input = decode_hex(input);
        let actual = parse_families(&input);
        if let Some(error) = expected.strip_prefix("E:") {
            assert_eq!(
                actual.unwrap_err().to_string(),
                error,
                "row {index}: {input:?}"
            );
        } else {
            let value = expected.strip_prefix("V:").unwrap();
            let families = if value.is_empty() {
                Vec::new()
            } else {
                value.split(',').map(decode_hex).collect()
            };
            assert_eq!(actual.unwrap(), families, "row {index}: {input:?}");
        }
        rows += 1;
    }
    assert_eq!(rows, 8224);
}

#[test]
fn source_fallback_ranges_match_seeded_default_and_custom_patterns() {
    let corpus = include_str!("fixtures/font-fallback.tsv");
    let mut rows = 0;
    for (index, row) in corpus.lines().enumerate() {
        let columns: Vec<_> = row.split('\t').collect();
        assert_eq!(columns.len(), 3);
        let text = decode_hex(columns[0]);
        let patterns = if columns[1].is_empty() {
            Vec::new()
        } else {
            columns[1].split(',').map(decode_hex).collect()
        };
        let ranges: Vec<[usize; 2]> = if columns[2].is_empty() {
            Vec::new()
        } else {
            columns[2]
                .split(',')
                .map(|range| {
                    let (start, end) = range.split_once(':').unwrap();
                    [start.parse().unwrap(), end.parse().unwrap()]
                })
                .collect()
        };
        let matcher = FallbackLigatures::new(patterns).unwrap();
        assert_eq!(matcher.ranges(&text), ranges, "row {index}: {text:?}");
        if index < 60 {
            assert_eq!(FallbackLigatures::default().ranges(&text), ranges);
        }
        rows += 1;
    }
    assert_eq!(rows, 4156);
}

#[test]
fn fallback_is_longest_first_nonoverlapping_and_utf16_indexed() {
    let matcher = FallbackLigatures::default();
    assert_eq!(
        matcher.ranges(&utf16("😀<---->!===>")),
        vec![[2, 8], [8, 12]]
    );
    let matcher = FallbackLigatures::new(vec![utf16("a"), utf16("ab"), utf16("bc")]).unwrap();
    assert_eq!(matcher.ranges(&utf16("abcab")), vec![[0, 2], [3, 5]]);
    assert!(FallbackLigatures::new(vec![Vec::new()]).is_err());
    assert!(FallbackLigatures::new(Vec::new())
        .unwrap()
        .ranges(&utf16("=="))
        .is_empty());
    assert_eq!(
        FallbackLigatures::new(vec![vec![0xD800]])
            .unwrap()
            .ranges(&[0xD800, 0xDC00]),
        vec![[0, 1]]
    );
}

#[test]
fn web_family_splitting_deliberately_differs_from_css_parser() {
    assert_eq!(
        split_family(&utf16("  'A B' , C,\u{feff}\"D\" ,")),
        vec![utf16("A B"), utf16("C"), utf16("D"), vec![]]
    );
    assert!(split_family(&[]).is_empty());
    assert_eq!(
        split_family(&utf16("'A,B'")),
        vec![utf16("'A"), utf16("B'")]
    );
    assert_eq!(unquote(&[0x22, 0xD800, 0x22]), [0xD800]);
    assert_eq!(unquote(&[0x22]), [0x22]);
}

#[test]
fn family_quote_rules_preserve_source_identifiers_and_escaping() {
    for name in ["monospace", "-mono", "_mono", "é", "😀"] {
        assert_eq!(quote(&utf16(name)), utf16(name));
    }
    for name in ["", "7mono", "-7mono", "--mono", "a b", "a.b", "a\n2"] {
        assert_eq!(quote(&utf16(name)), utf16(&format!("\"{name}\"")));
    }
    assert_eq!(quote(&utf16("a\"b\\c")), utf16("\"a\\\"b\\c\""));
    assert_eq!(
        create_family(&[utf16("A B"), utf16("serif")]),
        utf16("\"A B\", serif")
    );
}

#[test]
fn relayout_loads_dirty_in_order_then_falls_back_and_restores_exact_input() {
    let original = utf16(" 'Web', Local, Web, \"A B\"");
    let registered = vec![utf16("\"Web\""), utf16("unused")];
    let plan = relayout_plan(&original, &registered).unwrap();
    assert_eq!(plan.load_families, vec![utf16("Web"), utf16("Web")]);
    assert_eq!(plan.temporary_family, utf16("Local, \"A B\""));
    assert_eq!(plan.restore_family, original);
    assert_eq!(
        relayout_plan(&utf16("Web"), &registered)
            .unwrap()
            .temporary_family,
        utf16("monospace")
    );
    assert!(relayout_plan(&utf16("web"), &registered).is_none());
    assert!(relayout_plan(&[], &registered).is_none());
}

#[test]
fn face_dedup_uses_only_unquoted_family_stretch_style_range_and_weight() {
    let face = FontDescriptor {
        family: utf16("'Web'"),
        stretch: utf16("normal"),
        style: utf16("normal"),
        unicode_range: utf16("U+0-10FFFF"),
        weight: utf16("400"),
    };
    let mut other = face.clone();
    other.family = utf16("Web");
    assert_eq!(face.clone().normalized(), other.clone().normalized());
    for property in 0..5 {
        let mut changed = other.clone();
        let field = match property {
            0 => &mut changed.family,
            1 => &mut changed.stretch,
            2 => &mut changed.style,
            3 => &mut changed.unicode_range,
            _ => &mut changed.weight,
        };
        field.push(0x20);
        assert_ne!(face.clone().normalized(), changed.normalized());
    }
    other.family = utf16("web");
    assert_ne!(face.normalized(), other.normalized());
}
