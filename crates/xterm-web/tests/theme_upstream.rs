// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Assertions ported from browser/services/ThemeService.test.ts.
use xterm_web::theme::{palette::default_ansi_colors, Theme, ThemeService};

#[test]
fn constructor_fills_all_color_fields_and_ansi() {
    let service = ThemeService::default();
    let colors = service.colors();
    let colors = colors.borrow();
    for color in [
        &colors.foreground,
        &colors.background,
        &colors.cursor,
        &colors.cursor_accent,
        &colors.selection_background_transparent,
        &colors.selection_background_opaque,
        &colors.selection_inactive_background_transparent,
        &colors.selection_inactive_background_opaque,
        &colors.scrollbar_slider_background,
        &colors.scrollbar_slider_hover_background,
        &colors.scrollbar_slider_active_background,
        &colors.overview_ruler_border,
    ] {
        assert!(color.borrow().css.len() >= 7);
    }
    assert_eq!(colors.ansi.len(), 256);
}

#[test]
fn constructor_fills_all_240_explicit_expected_palette_values() {
    let service = ThemeService::default();
    let colors = service.colors();
    let colors = colors.borrow();
    for line in include_str!("fixtures/theme-palette.tsv").lines() {
        let (index, css) = line.split_once('\t').unwrap();
        let index: usize = index.parse().unwrap();
        assert_eq!(
            colors.ansi[index].borrow().css,
            css,
            "palette index {index}"
        );
    }
}

#[test]
fn empty_theme_does_not_throw() {
    ThemeService::default()
        .set_theme(&Theme::default())
        .unwrap();
}

#[test]
fn partial_theme_reverts_unspecified_colors_to_defaults() {
    let service = ThemeService::default();
    let colors = service.colors();
    assert_eq!(colors.borrow().background.borrow().css, "#000000");
    assert_eq!(colors.borrow().foreground.borrow().css, "#ffffff");
    service
        .set_theme(&Theme {
            background: Some("#FF0000".into()),
            foreground: Some("#00FF00".into()),
            ..Theme::default()
        })
        .unwrap();
    assert_eq!(colors.borrow().background.borrow().css, "#FF0000");
    assert_eq!(colors.borrow().foreground.borrow().css, "#00FF00");
    service
        .set_theme(&Theme {
            background: Some("#0000FF".into()),
            ..Theme::default()
        })
        .unwrap();
    assert_eq!(colors.borrow().background.borrow().css, "#0000FF");
    assert_eq!(colors.borrow().foreground.borrow().css, "#ffffff");
}

#[test]
fn all_extended_ansi_colors_can_be_set_in_reverse_order() {
    let service = ThemeService::default();
    let defaults = default_ansi_colors();
    service
        .set_theme(&Theme {
            extended_ansi: defaults.iter().rev().map(|c| Some(c.css.clone())).collect(),
            ..Theme::default()
        })
        .unwrap();
    let colors = service.colors();
    let colors = colors.borrow();
    for index in 16..=255 {
        assert_eq!(
            colors.ansi[index].borrow().css,
            defaults[255 + 16 - index].css
        );
    }
}

#[test]
fn one_extended_ansi_color_does_not_change_the_next_default() {
    let service = ThemeService::default();
    service
        .set_theme(&Theme {
            extended_ansi: vec![Some("#ffffff".into())],
            ..Theme::default()
        })
        .unwrap();
    let colors = service.colors();
    assert_eq!(colors.borrow().ansi[16].borrow().css, "#ffffff");
    assert_eq!(
        colors.borrow().ansi[17].borrow().css,
        default_ansi_colors()[17].css
    );
}

#[test]
fn empty_or_absent_extended_ansi_restores_defaults() {
    let service = ThemeService::default();
    let custom = Theme {
        extended_ansi: vec![Some("#ffffff".into())],
        ..Theme::default()
    };
    service.set_theme(&custom).unwrap();
    let colors = service.colors();
    assert_eq!(colors.borrow().ansi[16].borrow().css, "#ffffff");
    // Rust's empty vector represents both an empty and an absent extended array.
    service
        .set_theme(&Theme {
            extended_ansi: vec![],
            ..Theme::default()
        })
        .unwrap();
    assert_eq!(
        colors.borrow().ansi[16].borrow().css,
        default_ansi_colors()[16].css
    );
    service.set_theme(&custom).unwrap();
    assert_eq!(colors.borrow().ansi[16].borrow().css, "#ffffff");
    service.set_theme(&Theme::default()).unwrap();
    assert_eq!(
        colors.borrow().ansi[16].borrow().css,
        default_ansi_colors()[16].css
    );
}

#[test]
fn shortening_extended_ansi_resets_the_remaining_entries() {
    let service = ThemeService::default();
    service
        .set_theme(&Theme {
            extended_ansi: vec![Some("#ffffff".into()), Some("#000000".into())],
            ..Theme::default()
        })
        .unwrap();
    let colors = service.colors();
    assert_eq!(colors.borrow().ansi[16].borrow().css, "#ffffff");
    assert_eq!(colors.borrow().ansi[17].borrow().css, "#000000");
    service
        .set_theme(&Theme {
            extended_ansi: vec![Some("#ffffff".into())],
            ..Theme::default()
        })
        .unwrap();
    assert_eq!(colors.borrow().ansi[16].borrow().css, "#ffffff");
    assert_eq!(
        colors.borrow().ansi[17].borrow().css,
        default_ansi_colors()[17].css
    );
}
