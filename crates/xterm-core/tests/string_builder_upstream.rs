// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
// Faithful assertions from source_repo/src/common/StringBuilder.test.ts.
use xterm_core::string_builder::{LimitedStringBuilder, PayloadLimit, StringBuilder};

#[test]
fn starts_empty() {
    let b = StringBuilder::new();
    assert_eq!(b.len(), 0);
    assert_eq!(b.to_utf8().unwrap(), "");
}

#[test]
fn appends_single_chunk() {
    let mut b = StringBuilder::new();
    b.append("hello");
    assert_eq!(b.len(), 5);
    assert_eq!(b.to_utf8().unwrap(), "hello");
}

#[test]
fn joins_multiple_chunks_in_order() {
    let mut b = StringBuilder::new();
    for chunk in ["foo", "bar", "baz"] {
        b.append(chunk);
    }
    assert_eq!(b.len(), 9);
    assert_eq!(b.to_utf8().unwrap(), "foobarbaz");
}

#[test]
fn handles_empty_chunks() {
    let mut b = StringBuilder::new();
    for chunk in ["", "a", ""] {
        b.append(chunk);
    }
    assert_eq!(b.len(), 1);
    assert_eq!(b.to_utf8().unwrap(), "a");
}

#[test]
fn resets_accumulated_data() {
    let mut b = StringBuilder::new();
    b.append("hello");
    b.reset();
    assert_eq!(b.len(), 0);
    assert_eq!(b.to_utf8().unwrap(), "");
}

#[test]
fn appends_after_reset() {
    let mut b = StringBuilder::new();
    b.append("old");
    b.reset();
    b.append("new");
    assert_eq!(b.to_utf8().unwrap(), "new");
}

#[test]
fn accumulates_many_small_chunks() {
    let mut b = StringBuilder::new();
    for _ in 0..10_000 {
        b.append("x");
    }
    assert_eq!(b.len(), 10_000);
    assert_eq!(b.to_utf8().unwrap(), "x".repeat(10_000));
}

#[test]
fn exposes_limit() {
    assert_eq!(
        LimitedStringBuilder::new(42).limit(),
        PayloadLimit::Units(42)
    );
}

#[test]
fn limited_starts_empty() {
    let b = LimitedStringBuilder::new(10);
    assert_eq!(b.len(), 0);
    assert_eq!(b.to_utf8().unwrap(), "");
}

#[test]
fn accepts_up_to_limit() {
    let mut b = LimitedStringBuilder::new(10);
    assert!(!b.append("12345"));
    assert!(!b.append("67890"));
    assert_eq!(b.len(), 10);
    assert_eq!(b.to_utf8().unwrap(), "1234567890");
}

#[test]
fn accepts_single_exact_chunk() {
    let mut b = LimitedStringBuilder::new(5);
    assert!(!b.append("abcde"));
    assert_eq!(b.len(), 5);
    assert_eq!(b.to_utf8().unwrap(), "abcde");
}

#[test]
fn rejects_overflow_and_clears() {
    let mut b = LimitedStringBuilder::new(5);
    b.append("abc");
    assert!(b.append("def"));
    assert_eq!(b.len(), 0);
    assert_eq!(b.to_utf8().unwrap(), "");
}

#[test]
fn rejects_single_large_chunk() {
    let mut b = LimitedStringBuilder::new(3);
    assert!(b.append("toolong"));
    assert_eq!(b.len(), 0);
    assert_eq!(b.to_utf8().unwrap(), "");
}

#[test]
fn appends_after_breach_and_reset() {
    let mut b = LimitedStringBuilder::new(3);
    assert!(b.append("abcd"));
    b.reset();
    assert!(!b.append("ab"));
    assert_eq!(b.to_utf8().unwrap(), "ab");
}

#[test]
fn accumulates_to_limit_then_rejects() {
    let mut b = LimitedStringBuilder::new(100);
    for _ in 0..100 {
        assert!(!b.append("A"));
    }
    assert_eq!(b.to_utf8().unwrap(), "A".repeat(100));
    assert!(b.append("B"));
    assert_eq!(b.to_utf8().unwrap(), "");
}

#[test]
fn zero_limit_rejects_nonempty_chunk() {
    let mut b = LimitedStringBuilder::new(0);
    assert!(b.append("a"));
    assert_eq!(b.len(), 0);
}

#[test]
fn zero_limit_accepts_empty_chunk() {
    let mut b = LimitedStringBuilder::new(0);
    assert!(!b.append(""));
    assert_eq!(b.len(), 0);
    assert_eq!(b.to_utf8().unwrap(), "");
}
