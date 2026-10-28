// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::{
    buffer::BufferLine,
    search::{
        with_buffer, SearchBuffer, SearchEngine, SearchError, SearchLineCache, SearchOptions,
        SearchPosition, SearchSelection,
    },
    terminal::CoreTerminal,
};

fn find(
    terminal: &CoreTerminal,
    term: &str,
    row: isize,
) -> Result<Option<xterm_core::search::SearchResult>, SearchError> {
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    let mut cache = SearchLineCache::default();
    with_buffer(&buffer, terminal.cols(), |view| {
        SearchEngine::new(view, &mut cache, 0).find(
            &term.encode_utf16().collect::<Vec<_>>(),
            row,
            0,
            &SearchOptions::default(),
        )
    })
}

#[test]
fn one_slot_wrapped_ring_returns_source_range_error_without_native_recursion() {
    let line = BufferLine::new(4, None, true);
    let lookup = |_: isize| Some(&line);
    let view = SearchBuffer::from_ring_lookup(1, 4, 1, &lookup).unwrap();
    let mut cache = SearchLineCache::default();
    let error = SearchEngine::new(view, &mut cache, 0)
        .find(&[97], 0, 0, &SearchOptions::default())
        .unwrap_err();
    assert_eq!(error, SearchError::WrappedLineRecursion);
    assert_eq!(error.to_string(), "Maximum call stack size exceeded");
    // Source reverse traversal skips wrapped lines rather than recursing.
    assert!(SearchEngine::new(view, &mut cache, 0)
        .find_previous(&[97], &SearchOptions::default(), None, None)
        .unwrap()
        .is_none());
    assert_eq!(
        SearchLineCache::translate(view, 0, true),
        Err(SearchError::WrappedTranslationCycle)
    );
}

#[test]
fn wrapped_ring_number_decrement_stalls_return_source_range_error() {
    let line = BufferLine::new(4, None, true);
    let lookup = |_: isize| Some(&line);
    let view = SearchBuffer::from_ring_lookup(2, 4, 2, &lookup).unwrap();
    let mut cache = SearchLineCache::default();
    let expected = if isize::BITS == 64 {
        SearchError::WrappedLineRecursion
    } else {
        SearchError::IndexOverflow
    };
    assert_eq!(
        SearchEngine::new(view, &mut cache, 0).find(
            &[97],
            isize::MIN,
            0,
            &SearchOptions::default()
        ),
        Err(expected)
    );
    if let Ok(row) = isize::try_from(-9_007_199_254_740_992_i64) {
        let mut cache = SearchLineCache::default();
        assert_eq!(
            SearchEngine::new(view, &mut cache, 0).find(&[97], row, 0, &SearchOptions::default()),
            Err(SearchError::WrappedLineRecursion)
        );
        let missing = |_: isize| None;
        let empty = SearchBuffer::from_ring_lookup(1, 4, 3, &missing).unwrap();
        assert_eq!(
            SearchEngine::new(empty, &mut cache, 0).find(
                &[97],
                isize::MIN,
                0,
                &SearchOptions::default()
            ),
            Err(SearchError::RowTraversalCycle)
        );
    }
}

#[test]
fn full_observed_upstream_alternate_ring_regression_seed_baff0099() {
    let mut terminal = CoreTerminal::new(9, 1, 5).unwrap();
    for text in [
        "\x1b[2J",
        "\r\nnext",
        "中",
        "中",
        "\x1b[2J",
        "e\u{301}",
        "e\u{301}",
        "\r\nnext",
        "\r\nline",
        "\x1b[?1049hALT",
    ] {
        terminal.write(text).unwrap();
    }
    terminal.resize(9, 1).unwrap();
    terminal.write("abc").unwrap();
    for term in ["abc", "中", "next", "line", "e\u{301}"] {
        assert_eq!(
            find(&terminal, term, 0),
            Err(SearchError::WrappedLineRecursion)
        );
    }
}

#[test]
fn actual_ring_preserves_beyond_length_and_negative_alias_results() {
    let mut terminal = CoreTerminal::new(4, 2, 1).unwrap();
    terminal.write("aaaa\r\nbbbb\r\ncccc").unwrap();
    let found = find(&terminal, "aaaa", 3).unwrap().unwrap();
    assert_eq!((found.row, found.col, found.size), (3, 0, 4));
    terminal.write("\r\ndddd").unwrap();
    let found = find(&terminal, "dddd", -1).unwrap().unwrap();
    assert_eq!((found.row, found.col, found.size), (-1, 0, 4));
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    let mut cache = SearchLineCache::default();
    let selected = SearchSelection {
        start: SearchPosition { row: 4, col: 0 },
        end: SearchPosition { row: 4, col: 1 },
    };
    let previous = with_buffer(&buffer, terminal.cols(), |view| {
        SearchEngine::new(view, &mut cache, 0).find_previous(
            &[98, 98, 98, 98],
            &SearchOptions::default(),
            Some(&[98, 98, 98, 98]),
            Some(selected),
        )
    })
    .unwrap()
    .unwrap();
    assert_eq!((previous.row, previous.col, previous.size), (3, 0, 4));
}

#[test]
fn wrap_scan_can_find_stale_slots_beyond_the_cleared_viewport_length() {
    let mut terminal = CoreTerminal::new(4, 2, 8).unwrap();
    terminal
        .write("aaaa\r\nbbbb\r\nG\r\ndddd\r\neeee\r\nffff")
        .unwrap();
    terminal.clear().unwrap();
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    assert_eq!(buffer.lines.len(), 2);
    let selected = SearchSelection {
        start: SearchPosition { row: 4, col: 0 },
        end: SearchPosition { row: 4, col: 1 },
    };
    let mut cache = SearchLineCache::default();
    let found = with_buffer(&buffer, terminal.cols(), |view| {
        SearchEngine::new(view, &mut cache, 0).find_next(
            &[71],
            &SearchOptions::default(),
            Some(&[71]),
            Some(selected),
        )
    })
    .unwrap()
    .unwrap();
    assert_eq!((found.row, found.col, found.size), (2, 0, 1));
}

#[test]
fn sparse_large_scrollback_search_borrows_only_materialized_slots() {
    let mut terminal = CoreTerminal::new(8, 2, 1_000_000).unwrap();
    terminal.write("hello").unwrap();
    assert_eq!(find(&terminal, "hello", 0).unwrap().unwrap().size, 5);
}

#[test]
fn search_end_uses_live_base_plus_rows_not_transient_logical_storage_length() {
    let mut terminal = CoreTerminal::new(4, 2, 8).unwrap();
    terminal.write("aaaa\r\nG").unwrap();
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    // Resize/reflow callbacks can observe partial list updates. The source
    // getLine still exposes retained slots while search uses baseY+rows.
    buffer.lines.set_length(1);
    let mut cache = SearchLineCache::default();
    let found = with_buffer(&buffer, terminal.cols(), |view| {
        SearchEngine::new(view, &mut cache, 0).find(&[71], 0, 0, &SearchOptions::default())
    })
    .unwrap()
    .unwrap();
    assert_eq!((found.row, found.col, found.size), (1, 0, 1));
}

#[test]
fn conflicting_live_line_borrow_is_a_recoverable_search_error() {
    let terminal = CoreTerminal::new(8, 2, 1).unwrap();
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    let line = buffer.lines.get(0).unwrap();
    let _write_guard = line.borrow_mut();
    assert_eq!(
        with_buffer(&buffer, 8, |_| Ok(())),
        Err(SearchError::BorrowedLine)
    );
}
