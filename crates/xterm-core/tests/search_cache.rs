// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
//! Direct upstream cache assertions not exercised by translation fixtures.
//! The three terminal event subscription assertions remain addon integration.
use xterm_core::search::{
    LineCacheEntry, SearchLineCache, SearchOptions, SearchState, CACHE_TTL_MS,
};

fn entry(text: &str) -> LineCacheEntry {
    LineCacheEntry {
        text: text.encode_utf16().collect(),
        offsets: vec![0],
    }
}

#[test]
fn should_create_a_search_line_cache_instance() {
    let cache = SearchLineCache::default();
    assert!(cache.get(0).is_none());
}
#[test]
fn should_start_with_no_cache_initialized() {
    let cache = SearchLineCache::default();
    assert!(cache.get(0).is_none());
    assert!(cache.deadline_ms().is_none());
}
#[test]
fn should_initialize_the_lines_cache_array() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    assert!(cache.get(0).is_none());
}
#[test]
fn should_not_reinitialize_if_cache_already_exists() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(0, entry("test"));
    cache.init(1);
    assert_eq!(cache.get(0), Some(&entry("test")));
}
#[test]
fn should_set_up_ttl_timeout() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(0, entry("test"));
    assert_eq!(cache.get(0), Some(&entry("test")));
    assert_eq!(cache.deadline_ms(), Some(CACHE_TTL_MS));
}
#[test]
fn should_return_undefined_when_cache_is_not_initialized() {
    let cache = SearchLineCache::default();
    assert!(cache.get(0).is_none());
    assert!(cache.get(10).is_none());
}
#[test]
fn should_return_undefined_for_unset_entries_when_initialized() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    assert!(cache.get(0).is_none());
    assert!(cache.get(50).is_none());
}
#[test]
fn should_return_cached_entries() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(5, entry("test content"));
    assert_eq!(cache.get(5), Some(&entry("test content")));
}
#[test]
fn should_not_set_entries_when_not_initialized() {
    let mut cache = SearchLineCache::default();
    cache.set(0, entry("test content"));
    assert!(cache.get(0).is_none());
}
#[test]
fn should_set_entries_when_initialized() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(10, entry("test content"));
    assert_eq!(cache.get(10), Some(&entry("test content")));
}
#[test]
fn should_overwrite_existing_entries() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(0, entry("first content"));
    assert_eq!(cache.get(0), Some(&entry("first content")));
    cache.set(0, entry("second content"));
    assert_eq!(cache.get(0), Some(&entry("second content")));
}
#[test]
fn should_clean_up_resources_on_dispose() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(0, entry("test"));
    assert_eq!(cache.get(0), Some(&entry("test")));
    cache.invalidate();
    assert!(cache.get(0).is_none());
    assert!(cache.deadline_ms().is_none());
}
#[test]
fn should_be_safe_to_dispose_multiple_times() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.invalidate();
    cache.invalidate();
    assert!(cache.get(0).is_none());
}
#[test]
fn should_handle_complex_line_offsets() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    let entry = LineCacheEntry {
        text: "A very long line that wraps multiple times across several terminal lines"
            .encode_utf16()
            .collect(),
        offsets: vec![0, 20, 40, 60],
    };
    cache.set(0, entry.clone());
    assert_eq!(cache.get(0), Some(&entry));
    assert_eq!(cache.get(0).unwrap().text.len(), 72);
    assert_eq!(cache.get(0).unwrap().offsets.len(), 4);
}
#[test]
fn should_handle_unicode_characters_in_cache_entries() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    let entry = entry("Hello 世界 🌍 测试");
    cache.set(0, entry.clone());
    assert_eq!(cache.get(0), Some(&entry));
}
#[test]
fn ttl_boundary_refresh_and_repeated_expiry() {
    let mut cache = SearchLineCache::default();
    cache.init(100);
    cache.set(0, entry("first"));
    cache.expire(15_099);
    assert!(cache.get(0).is_some());
    cache.init(15_099);
    assert_eq!(cache.deadline_ms(), Some(30_099));
    cache.expire(15_100);
    assert!(cache.get(0).is_some());
    cache.expire(30_099);
    assert!(cache.get(0).is_none());
    cache.expire(30_100);
    assert!(cache.get(0).is_none());
}
#[test]
fn initialized_cache_invalidation_and_negative_property_keys() {
    let mut cache = SearchLineCache::default();
    cache.init(0);
    cache.set(-1, entry("property"));
    assert_eq!(cache.get(-1), Some(&entry("property")));
    cache.invalidate();
    cache.set(0, entry("ignored"));
    assert!(cache.get(-1).is_none());
    assert!(cache.get(0).is_none());
    cache.init(1);
    assert!(cache.get(0).is_none());
}
#[test]
fn clearing_cached_term_retains_options_and_reset_clears_both() {
    let mut state = SearchState {
        cached_search_term: Some(vec![0xd800]),
        last_search_options: Some(SearchOptions::default()),
    };
    state.clear_cached_term();
    assert!(state.cached_search_term.is_none());
    assert!(state.last_search_options.is_some());
    state.reset();
    assert!(state.cached_search_term.is_none());
    assert!(state.last_search_options.is_none());
}
