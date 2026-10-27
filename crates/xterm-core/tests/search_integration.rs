// Copyright (c) 2026 xterm-rust contributors. MIT license.
use xterm_core::{
    buffer::BufferLine,
    search::{
        LineCacheEntry, SearchBuffer, SearchEngine, SearchError, SearchLineCache, SearchOptions,
        SearchPosition, SearchSelection,
    },
    terminal::CoreTerminal,
};

fn search(
    terminal: &CoreTerminal,
    cache: &mut SearchLineCache,
    term: &str,
    selection: Option<SearchSelection>,
) -> xterm_core::search::SearchResult {
    let active = terminal.state().active_buffer();
    let active = active.borrow();
    let lines: Vec<_> = (0..active.lines.len())
        .map(|i| active.lines.get(i).unwrap())
        .collect();
    let guards: Vec<_> = lines.iter().map(|line| line.borrow()).collect();
    let rows: Vec<_> = guards.iter().map(|line| &**line).collect();
    let view = SearchBuffer::from_rows(&rows, terminal.cols()).unwrap();
    let term: Vec<_> = term.encode_utf16().collect();
    SearchEngine::new(view, cache, 0)
        .find_next(&term, &SearchOptions::default(), Some(&term), selection)
        .unwrap()
        .unwrap()
}

#[test]
fn terminal_parser_cells_and_search_share_live_ring_rows() {
    let mut terminal = CoreTerminal::new(20, 3, 10).unwrap();
    terminal.write("red \x1b[31mtarget\x1b[0m 中").unwrap();
    let mut cache = SearchLineCache::default();
    let first = search(&terminal, &mut cache, "target", None);
    assert_eq!((first.row, first.col, first.size), (0, 4, 6));
    terminal.write("\r\n中target").unwrap();
    // Task59 owns automatic event wiring; the portable cache has an explicit
    // invalidation boundary rather than observing mutable global terminal state.
    cache.invalidate();
    let selection = SearchSelection {
        start: SearchPosition { row: 0, col: 4 },
        end: SearchPosition { row: 0, col: 10 },
    };
    let second = search(&terminal, &mut cache, "target", Some(selection));
    assert_eq!((second.row, second.col, second.size), (1, 2, 6));
}

#[test]
fn terminal_wide_character_wrap_and_cross_row_cell_span() {
    let mut terminal = CoreTerminal::new(8, 3, 10).unwrap();
    terminal.write("1234567中target").unwrap();
    let mut cache = SearchLineCache::default();
    let target = search(&terminal, &mut cache, "target", None);
    assert_eq!((target.row, target.col, target.size), (1, 2, 6));
    let span = search(&terminal, &mut cache, "7中", None);
    assert_eq!((span.row, span.col, span.size), (0, 6, 4));
}

#[test]
fn invalid_dimensions_and_nonuniform_row_widths_are_errors() {
    let lines = [BufferLine::new(2, None, false)];
    assert!(matches!(
        SearchBuffer::new(&lines, 0),
        Err(SearchError::InvalidDimensions)
    ));
    assert!(matches!(
        SearchBuffer::new(&lines, 3),
        Err(SearchError::InvalidDimensions)
    ));
    assert!(matches!(
        SearchBuffer::new(&[], usize::MAX),
        Err(SearchError::InvalidDimensions)
    ));
}

#[test]
fn malformed_cache_offsets_are_recoverable_errors() {
    for offsets in [vec![], vec![1], vec![0, 2, 1], vec![0, 4]] {
        let lines = [BufferLine::new(4, None, false)];
        let buffer = SearchBuffer::new(&lines, 4).unwrap();
        let mut cache = SearchLineCache::default();
        cache.init(0);
        cache.set(
            0,
            LineCacheEntry {
                text: vec![97, 98, 99],
                offsets,
            },
        );
        assert_eq!(
            SearchEngine::new(buffer, &mut cache, 0).find(&[97], 0, 0, &SearchOptions::default()),
            Err(SearchError::InvalidCacheEntry)
        );
    }
}

#[test]
fn extreme_negative_starts_skip_nonexistent_rows_without_iteration() {
    let lines = [BufferLine::new(4, None, false)];
    let buffer = SearchBuffer::new(&lines, 4).unwrap();
    let mut cache = SearchLineCache::default();
    assert_eq!(
        SearchEngine::new(buffer, &mut cache, 0)
            .find(&[97], isize::MIN, isize::MIN, &SearchOptions::default())
            .unwrap(),
        None
    );
}
