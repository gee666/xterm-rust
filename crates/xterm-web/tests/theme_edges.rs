use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    color::{channels, Color},
    lifecycle::Disposable,
};
use xterm_web::{
    contrast_cache::ColorContrastCache,
    theme::{ColorRef, ColorSlot, Theme, ThemeService},
};

// Original ColorContrastCache.test.ts cases, preserving missing/null/value states.
#[test]
fn cache_saves_and_gets_color_values() {
    let mut cache = ColorContrastCache::new();
    assert!(cache.get_color(1, 0).is_none());
    cache.set_color(1, 1, None);
    assert_eq!(cache.get_color(1, 1), Some(&None));
    let color: ColorRef = Color {
        css: "#030303".into(),
        rgba: 0x030303ff,
    }
    .into();
    cache.set_color(1, 2, Some(color.clone()));
    let cached = cache.get_color(1, 2).unwrap().as_ref().unwrap();
    assert_eq!(
        *cached.borrow(),
        Color {
            css: "#030303".into(),
            rgba: 0x030303ff
        }
    );
    assert!(cached.same_object(&color));
    color.borrow_mut().rgba = 42;
    assert_eq!(cached.borrow().rgba, 42);
}
#[test]
fn cache_saves_and_gets_css_values() {
    let mut cache = ColorContrastCache::new();
    assert!(cache.get_css(1, 0).is_none());
    cache.set_css(1, 1, None);
    assert_eq!(cache.get_css(1, 1), Some(&None));
    cache.set_css(1, 2, Some("#030303".into()));
    assert_eq!(cache.get_css(1, 2).unwrap().as_deref(), Some("#030303"));
}
#[test]
fn cache_clear_removes_both_stores_and_preserves_tuple_order() {
    let mut cache = ColorContrastCache::new();
    cache.set_color(1, 1, None);
    cache.set_color(1, 2, Some(channels::to_color(3, 3, 3, None).into()));
    cache.set_css(1, 1, None);
    cache.set_css(1, 2, Some("#030303".into()));
    assert!(cache.get_css(2, 1).is_none());
    cache.clear();
    assert!(cache.get_color(1, 1).is_none());
    assert!(cache.get_color(1, 2).is_none());
    assert!(cache.get_css(1, 1).is_none());
    assert!(cache.get_css(1, 2).is_none());
}
#[test]
fn opaque_selections_are_blended_before_transparent_copy_adjustment() {
    let service = ThemeService::default();
    service
        .set_theme(&Theme {
            background: Some("#102030".into()),
            selection_background: Some("#ff0000".into()),
            ..Theme::default()
        })
        .unwrap();
    let colors = service.colors();
    let colors = colors.borrow();
    assert_eq!(
        colors.selection_background_transparent.borrow().rgba,
        0xff00004d
    );
    assert_eq!(colors.selection_background_opaque.borrow().rgba, 0xff0000ff);
    assert_eq!(
        colors
            .selection_inactive_background_transparent
            .borrow()
            .rgba,
        0xff00004d
    );
    assert_eq!(
        colors.selection_inactive_background_opaque.borrow().rgba,
        0xff0000ff
    );
}
#[test]
fn invalid_and_transparent_foregrounds_remain_distinct() {
    let service = ThemeService::default();
    let colors = service.colors();
    service
        .set_theme(&Theme {
            selection_foreground: Some("invalid".into()),
            foreground: Some("invalid".into()),
            ..Theme::default()
        })
        .unwrap();
    assert!(colors.borrow().selection_foreground.is_none());
    assert_eq!(colors.borrow().foreground.borrow().rgba, 0xffffffff);
    service
        .set_theme(&Theme {
            selection_foreground: Some("#00000000".into()),
            ..Theme::default()
        })
        .unwrap();
    assert_eq!(
        colors
            .borrow()
            .selection_foreground
            .as_ref()
            .unwrap()
            .borrow()
            .rgba,
        0
    );
}
#[test]
fn cache_invalidation_matches_the_exact_source_option_path() {
    let service = ThemeService::default();
    let colors = service.colors();
    let full = Rc::clone(&colors.borrow().contrast_cache);
    let half = Rc::clone(&colors.borrow().half_contrast_cache);
    full.borrow_mut().set_color(1, 1, None);
    half.borrow_mut().set_color(1, 1, None);
    service.minimum_contrast_ratio_changed();
    assert!(full.borrow().get_color(1, 1).is_none());
    assert_eq!(half.borrow().get_color(1, 1), Some(&None));
    service.set_theme(&Theme::default()).unwrap();
    assert!(full.borrow().get_color(1, 1).is_none());
    assert!(half.borrow().get_color(1, 1).is_none());
    assert!(Rc::ptr_eq(&full, &colors.borrow().contrast_cache));
    assert!(Rc::ptr_eq(&half, &colors.borrow().half_contrast_cache));
}
#[test]
fn restore_retains_color_identity_and_not_temporary_replacements() {
    let service = ThemeService::default();
    let colors = service.colors();
    let original = colors.borrow().foreground.clone();
    original.borrow_mut().css = "original-mutated".into();
    service
        .modify_colors(|colors| {
            colors.borrow_mut().foreground = channels::to_color(1, 2, 3, None).into();
            colors.borrow_mut().ansi[0] = channels::to_color(9, 8, 7, None).into();
        })
        .unwrap();
    service.restore_color(None).unwrap();
    assert_eq!(colors.borrow().foreground.borrow().css, "#010203");
    assert_eq!(colors.borrow().ansi[0].borrow().css, "#2e3436");
    service.restore_color(Some(ColorSlot::Foreground)).unwrap();
    assert!(colors.borrow().foreground.same_object(&original));
    assert_eq!(colors.borrow().foreground.borrow().css, "original-mutated");
}
#[test]
fn color_events_commit_state_before_reentrant_listeners() {
    let service = Rc::new(ThemeService::default());
    let weak = Rc::downgrade(&service);
    let calls = Rc::new(Cell::new(0));
    let log = Rc::new(RefCell::new(Vec::new()));
    let calls_ref = Rc::clone(&calls);
    let log_ref = Rc::clone(&log);
    let live = service.colors();
    let expected_live = Rc::clone(&live);
    let handle = service.on_change_colors().subscribe(move |colors| {
        assert!(Rc::ptr_eq(colors, &expected_live));
        log_ref
            .borrow_mut()
            .push(colors.borrow().background.borrow().css.clone());
        calls_ref.set(calls_ref.get() + 1);
        if calls_ref.get() == 1 {
            weak.upgrade()
                .unwrap()
                .restore_color(Some(ColorSlot::Background))
                .unwrap();
        }
    });
    service
        .set_theme(&Theme {
            background: Some("#123456".into()),
            ..Theme::default()
        })
        .unwrap();
    assert_eq!(*log.borrow(), ["#123456", "#123456"]);
    handle.dispose();
    service.restore_color(Some(ColorSlot::Background)).unwrap();
    assert_eq!(calls.get(), 2);
    service.dispose();
}
