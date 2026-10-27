use super::*;

fn textarea(value: &str) -> Textarea {
    Textarea {
        value: value.encode_utf16().collect(),
        ..Textarea::default()
    }
}
fn output(data: Option<Vec<u16>>) -> String {
    String::from_utf16(&data.unwrap()).unwrap()
}
fn update(state: &mut Composition, value: &str) -> Textarea {
    let action = state.update(&value.encode_utf16().collect::<Vec<_>>());
    let area = textarea(value);
    assert_eq!(state.run_deferred(action, &area), None);
    area
}
fn commit(state: &mut Composition, area: &Textarea) -> String {
    let action = state.end();
    output(state.run_deferred(action, area))
}

#[test]
fn upstream_simple_characters() {
    let mut state = Composition::default();
    state.start(&textarea(""));
    let area = update(&mut state, "ㅇ");
    let mut handled = commit(&mut state, &area);
    state.start(&area);
    let area = update(&mut state, "ㅇㅇ");
    handled.push_str(&commit(&mut state, &area));
    assert_eq!(handled, "ㅇㅇ");
}

#[test]
fn upstream_complex_characters() {
    let mut state = Composition::default();
    state.start(&textarea(""));
    update(&mut state, "ㅇ");
    update(&mut state, "아");
    let area = update(&mut state, "앙");
    let mut handled = commit(&mut state, &area);
    assert_eq!(handled, "앙");
    state.start(&area);
    update(&mut state, "앙ㅇ");
    update(&mut state, "앙아");
    let area = update(&mut state, "앙앙");
    handled.push_str(&commit(&mut state, &area));
    assert_eq!(handled, "앙앙");
}

#[test]
fn upstream_complex_changes_with_following_character() {
    let mut state = Composition::default();
    state.start(&textarea(""));
    update(&mut state, "ㅇ");
    update(&mut state, "아");
    let area = update(&mut state, "앙");
    let send_first = state.end();
    state.start(&area);
    let update_second = state.update(&"아".encode_utf16().collect::<Vec<_>>());
    let area = textarea("아아");
    let mut handled = output(state.run_deferred(send_first, &area));
    state.run_deferred(update_second, &area);
    handled.push_str(&commit(&mut state, &area));
    assert_eq!(handled, "아아");
}

#[test]
fn upstream_multicharacters() {
    assert_composition(&["d", "だ", "だあ"], "だあ");
}
#[test]
fn upstream_same_length_conversion() {
    assert_composition(&["d", "だ", "だー", "ダー"], "ダー");
}
#[test]
fn upstream_different_length_conversion() {
    assert_composition(&["い", "いm", "いま", "今"], "今");
}
fn assert_composition(updates: &[&str], expected: &str) {
    let mut state = Composition::default();
    state.start(&textarea(""));
    let mut area = Textarea::default();
    for value in updates {
        area = update(&mut state, value);
    }
    assert_eq!(commit(&mut state, &area), expected);
}

#[test]
fn upstream_input_immediately_after_composition() {
    let mut state = Composition::default();
    state.start(&textarea(""));
    update(&mut state, "ㅇ");
    let action = state.end();
    assert_eq!(output(state.run_deferred(action, &textarea("ㅇ1"))), "ㅇ1");
}

#[test]
fn upstream_middle_insertion_does_not_resend_suffix() {
    let mut state = Composition::default();
    let mut area = textarea("一二");
    area.selection_start = Some(1);
    area.selection_end = Some(1);
    state.start(&area);
    let action = state.update(&"一".encode_utf16().collect::<Vec<_>>());
    area.value = "一一二".encode_utf16().collect();
    area.selection_start = Some(2);
    area.selection_end = Some(2);
    state.run_deferred(action, &area);
    let send = state.end();
    area.value = "一一1二".encode_utf16().collect();
    assert_eq!(output(state.run_deferred(send, &area)), "一1");
}

#[test]
fn modifiers_suppress_and_immediate_enter_cancels_deferred_send() {
    let mut state = Composition::default();
    state.start(&textarea(""));
    let area = update(&mut state, "今");
    for code in [16, 17, 18, 20, 229] {
        let result = state.keydown(code, &area);
        assert!(!result.continue_processing);
        assert!(result.data.is_none());
    }
    let send = state.end();
    let result = state.keydown(13, &area);
    assert!(result.continue_processing);
    assert_eq!(output(result.data), "今");
    assert!(state.run_deferred(send, &area).is_none());
}

#[test]
fn standalone_229_diff_coalesces_and_handles_replace_and_delete() {
    for (old, new, expected) in [("a", "ab", "b"), ("ab", "a", "\x7f"), ("ab", "cd", "cd")] {
        let mut state = Composition::default();
        let result = state.keydown(229, &textarea(old));
        assert!(!result.continue_processing);
        assert!(state.keydown(229, &textarea(old)).deferred.is_none());
        assert_eq!(
            output(state.run_deferred(result.deferred.unwrap(), &textarea(new))),
            expected
        );
    }
}

#[test]
fn new_composition_cancels_pending_textarea_diff() {
    let mut state = Composition::default();
    let deferred = state.keydown(229, &textarea("a")).deferred.unwrap();
    state.start(&textarea("a"));
    assert!(state.run_deferred(deferred, &textarea("ab")).is_none());
}

#[test]
fn utf16_selection_preserves_unpaired_surrogates_and_reversed_range() {
    let mut state = Composition::default();
    let area = Textarea {
        value: vec![0xd800, 0xdc00, 0xd801],
        selection_start: Some(3),
        selection_end: Some(2),
    };
    state.start(&area);
    assert_eq!(state.keydown(13, &area).data, Some(vec![0xd801]));
    let data = vec![0xd800, 0xdc00, 0xd801];
    let _ = state.update(&data);
    assert_eq!(state.overlay_text, [0x200e, 0xd800, 0xdc00, 0xd801, 0x200e]);
}
