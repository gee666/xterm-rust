// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
//! Replays actual-cell inputs observed while unchanged upstream assertions ran,
//! followed by a seeded corpus. Expected outputs come from the source engine.
use xterm_core::{
    buffer::{BufferLine, CellData, CharData},
    search::{
        SearchBuffer, SearchEngine, SearchError, SearchLineCache, SearchOptions, SearchPosition,
        SearchSelection, SearchState,
    },
};

fn units(value: &str) -> Vec<u16> {
    if value == "-" {
        return vec![];
    }
    value
        .split(',')
        .map(|s| u16::from_str_radix(s, 16).unwrap())
        .collect()
}
fn hex(value: &[u16]) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .iter()
        .map(|u| format!("{u:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn options(value: &str) -> SearchOptions {
    let flags: Vec<_> = value
        .bytes()
        .map(|b| match b {
            b'u' => None,
            b't' => Some(true),
            b'f' => Some(false),
            _ => panic!("invalid fixture boolean"),
        })
        .collect();
    SearchOptions {
        case_sensitive: flags[0],
        regex: flags[1],
        whole_word: flags[2],
        ..SearchOptions::default()
    }
}
fn lines(value: &str, cols: usize) -> Vec<BufferLine> {
    value
        .split('|')
        .map(|row| {
            let (wrap, cells) = row.split_once('~').unwrap();
            let mut line = BufferLine::new(cols, None, wrap == "1");
            if cells != "-" {
                for (col, encoded) in cells.split(';').enumerate() {
                    let (chars, width) = encoded.split_once(':').unwrap();
                    line.set_cell(
                        col,
                        &CellData::from_char_data(&CharData {
                            attr: 0,
                            chars: units(chars),
                            width: width.parse().unwrap(),
                            code: None,
                        }),
                    );
                }
            }
            line
        })
        .collect()
}
fn selection(value: &str) -> Option<SearchSelection> {
    if value == "-" {
        return None;
    }
    let positions: Vec<isize> = value.split(',').map(|s| s.parse().unwrap()).collect();
    Some(SearchSelection {
        start: SearchPosition {
            row: positions[0],
            col: positions[1],
        },
        end: SearchPosition {
            row: positions[2],
            col: positions[3],
        },
    })
}

#[test]
fn unchanged_upstream_calls_and_seeded_engine_corpus() {
    let mut count = 0;
    for (index, row) in include_str!("fixtures/search-engine.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        let cols = fields[0].parse().unwrap();
        let lines = lines(fields[1], cols);
        let buffer = SearchBuffer::new(&lines, cols).unwrap();
        let term = units(fields[2]);
        let options = options(fields[6]);
        let cached = (fields[7] != "~").then(|| units(fields[7]));
        let selection = selection(fields[8]);
        let mut cache = SearchLineCache::default();
        let mut engine = SearchEngine::new(buffer, &mut cache, 0);
        let result = match fields[3] {
            "find" => engine.find(
                &term,
                fields[4].parse().unwrap(),
                fields[5].parse().unwrap(),
                &options,
            ),
            "findNextWithSelection" => {
                engine.find_next(&term, &options, cached.as_deref(), selection)
            }
            "findPreviousWithSelection" => {
                engine.find_previous(&term, &options, cached.as_deref(), selection)
            }
            _ => panic!("invalid fixture method"),
        };
        let actual = match result {
            Ok(None) => "none".into(),
            Ok(Some(found)) => format!(
                "{}/{}/{}/{}",
                hex(&found.term),
                found.row,
                found.col,
                found.size
            ),
            Err(SearchError::InvalidColumn { .. }) => "error-column".into(),
            Err(SearchError::InvalidRegex(_)) => "error-regex".into(),
            Err(e) => panic!("unexpected error: {e}"),
        };
        assert_eq!(
            actual,
            fields[9],
            "engine fixture {}: {}",
            index + 1,
            fields[10]
        );
        count += 1;
    }
    assert_eq!(count, 2200);
}

#[test]
fn actual_cell_wrapped_translation_corpus() {
    let mut count = 0;
    for (index, row) in include_str!("fixtures/search-translation.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        let cols = fields[0].parse().unwrap();
        let lines = lines(fields[1], cols);
        let buffer = SearchBuffer::new(&lines, cols).unwrap();
        let result =
            SearchLineCache::translate(buffer, fields[2].parse().unwrap(), fields[3] == "1")
                .unwrap();
        assert_eq!(
            hex(&result.text),
            fields[4],
            "translation fixture {}: {}",
            index + 1,
            fields[6]
        );
        let offsets = result
            .offsets
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(offsets, fields[5], "translation offsets {}", index + 1);
        count += 1;
    }
    assert_eq!(count, 6116);
}

#[test]
fn optional_options_and_highlighting_state_corpus() {
    let mut count = 0;
    for (index, row) in include_str!("fixtures/search-state.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        let state = SearchState {
            last_search_options: (fields[0] != "-").then(|| options(fields[0])),
            cached_search_term: (fields[3] != "~").then(|| units(fields[3])),
        };
        let next = (fields[1] != "-").then(|| SearchOptions {
            decorations: fields[2] == "1",
            ..options(fields[1])
        });
        let term = units(fields[4]);
        assert_eq!(
            state.is_valid_search_term(&term),
            fields[5] == "1",
            "valid term {index}"
        );
        assert_eq!(
            state.did_options_change(next.as_ref()),
            fields[6] == "1",
            "changed options {index}"
        );
        assert_eq!(
            state.should_update_highlighting(&term, next.as_ref()),
            fields[7] == "1",
            "highlighting {index}"
        );
        count += 1;
    }
    assert_eq!(count, 1024);
}
