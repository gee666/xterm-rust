use std::rc::Rc;
use xterm_web::links::{
    Decorations, HoverEffect, HoverState, Link, Position, ProviderReplies, ProviderRequest, Range,
};
fn link(text: &str, start: usize, end: usize) -> Rc<Link> {
    Rc::new(Link {
        text: text.encode_utf16().collect(),
        range: Range {
            start: Position { x: start, y: 1 },
            end: Position { x: end, y: 1 },
        },
    })
}
fn request() -> ProviderRequest {
    ProviderRequest::new(Position { x: 5, y: 1 })
}
#[test]
fn pending_and_empty_reply_delay_lower_priority() {
    let mut replies = ProviderReplies::default();
    let r = request();
    let lower = link("low", 4, 8);
    assert!(replies
        .receive(1, Some(vec![lower.clone()]), &r, 3, 80)
        .is_none());
    assert!(replies.receive(0, Some(vec![]), &r, 3, 80).is_none());
    assert_eq!(replies.receive(2, None, &r, 3, 80), Some(lower));
}
#[test]
fn undefined_higher_reply_allows_early_lower_selection() {
    let mut replies = ProviderReplies::default();
    let r = request();
    assert!(replies.receive(0, None, &r, 3, 80).is_none());
    let lower = link("low", 4, 8);
    assert_eq!(
        replies.receive(1, Some(vec![lower.clone()]), &r, 3, 80),
        Some(lower)
    );
}
#[test]
fn all_replies_choose_provider_order_not_completion_order() {
    let mut replies = ProviderReplies::default();
    let r = request();
    let high = link("high", 4, 8);
    let low = link("low", 4, 8);
    assert!(replies.receive(1, Some(vec![low]), &r, 2, 80).is_none());
    assert_eq!(
        replies.receive(0, Some(vec![high.clone()]), &r, 2, 80),
        Some(high)
    );
    assert!(replies.links(1).unwrap().is_empty());
}
#[test]
fn rejected_overlap_retains_partially_occupied_cells() {
    let mut replies = ProviderReplies::default();
    let r = request();
    replies.receive(0, Some(vec![link("first", 5, 6)]), &r, 2, 80);
    replies.receive(
        1,
        Some(vec![
            link("rejected", 3, 5),
            link("also rejected", 3, 4),
            link("kept", 7, 9),
        ]),
        &r,
        2,
        80,
    );
    let remaining = replies.links(1).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].text, "kept".encode_utf16().collect::<Vec<_>>());
    assert_eq!(replies.clear().len(), 2);
    assert!(replies.links(0).is_none());
}
#[test]
fn hover_effect_order_and_mouseup_text_range_equality() {
    let mut state = HoverState::new(80, 0);
    let original = link("x", 4, 8);
    let effects = state.enter(original.clone(), None);
    assert!(matches!(
        effects[0],
        HoverEffect::Underline { show: true, .. }
    ));
    assert_eq!(effects[1], HoverEffect::Pointer(true));
    assert_eq!(effects[2], HoverEffect::Hover(original));
    state.mouse_down();
    state.enter(link("x", 4, 8), None);
    assert!(matches!(
        state.mouse_up(Some(Position { x: 5, y: 1 })),
        Some(HoverEffect::Activate(_))
    ));
    assert!(state.mouse_up(None).is_none());
    assert!(state.mouse_up(Some(Position { x: 9, y: 1 })).is_none());
    state.enter(link("different", 4, 8), None);
    assert!(state.mouse_up(Some(Position { x: 5, y: 1 })).is_none());
}
#[test]
fn dynamic_decorations_and_viewport_invalidation() {
    let mut state = HoverState::new(80, 0);
    assert_eq!(
        state
            .enter(
                link("x", 4, 8),
                Some(Decorations {
                    underline: false,
                    pointer_cursor: false
                })
            )
            .len(),
        1
    );
    assert!(matches!(
        state.set_underline(true),
        Some(HoverEffect::Underline { show: true, .. })
    ));
    assert!(state.set_underline(true).is_none());
    assert_eq!(
        state.set_pointer_cursor(true),
        Some(HoverEffect::Pointer(true))
    );
    assert!(state.invalidated_by_render(0, 0));
    assert!(!state.invalidated_by_render(1, 5));
    let effects = state.leave();
    assert!(matches!(
        effects[0],
        HoverEffect::Underline { show: false, .. }
    ));
    assert_eq!(effects[1], HoverEffect::Pointer(false));
    assert!(matches!(effects[2], HoverEffect::Leave(_)));
    assert!(state.current().is_none());
    assert!(state.leave().is_empty());
}
