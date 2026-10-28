use serde_json::Value;
use std::rc::Rc;
use xterm_core::color::{css, Color};
use xterm_web::renderer::{theme_css::theme_css, width_cache::Font};
use xterm_web::theme::{ColorRef, ColorSet, Theme, ThemeService};

fn color(value: &Value) -> ColorRef {
    Color {
        css: value["css"].as_str().unwrap().into(),
        rgba: value["rgba"].as_u64().unwrap().try_into().unwrap(),
    }
    .into()
}

fn colors(value: &Value) -> ColorSet {
    let service = ThemeService::new(&Theme::default(), Rc::new(css::to_color));
    let mut colors = service.colors().borrow().clone();
    colors.foreground = color(&value["foreground"]);
    colors.background = color(&value["background"]);
    colors.cursor = color(&value["cursor"]);
    colors.cursor_accent = color(&value["cursorAccent"]);
    colors.selection_background_opaque = color(&value["selectionBackgroundOpaque"]);
    colors.selection_inactive_background_opaque =
        color(&value["selectionInactiveBackgroundOpaque"]);
    colors.ansi = value["ansi"]
        .as_array()
        .unwrap()
        .iter()
        .map(color)
        .collect();
    colors
}

fn number(value: &Value) -> f64 {
    match value.as_str().unwrap() {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        value => value.parse().unwrap(),
    }
}

#[test]
fn complete_css_matches_unchanged_source_on_73_theme_cases() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/theme-css.json")).unwrap();
    assert_eq!(cases.len(), 73);
    for case in cases {
        let font = Font {
            family: case["font"]["family"].as_str().unwrap().into(),
            size: number(&case["font"]["size"]),
            weight: case["font"]["weight"].as_str().unwrap().into(),
            weight_bold: case["font"]["weightBold"].as_str().unwrap().into(),
        };
        let actual = theme_css(
            case["id"].as_str().unwrap().parse().unwrap(),
            &colors(&case["colors"]),
            &font,
            number(&case["cursorWidth"]),
        );
        let expected = case["expectedCss"].as_str().unwrap();
        if actual != expected {
            let index = actual
                .bytes()
                .zip(expected.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(actual.len().min(expected.len()));
            let fragment = |text: &str| {
                String::from_utf8_lossy(
                    &text.as_bytes()[index.saturating_sub(30)..(index + 90).min(text.len())],
                )
                .into_owned()
            };
            panic!(
                "{} full CSS mismatch at byte {index}, lengths {}/{}; actual {:?}; expected {:?}",
                case["name"],
                actual.len(),
                expected.len(),
                fragment(&actual),
                fragment(expected)
            );
        }
    }
}

#[test]
fn selection_palette_and_inverted_default_have_exact_scope() {
    let service = ThemeService::new(&Theme::default(), Rc::new(css::to_color));
    let mut colors = service.colors().borrow().clone();
    colors.ansi.clear();
    let font = Font {
        family: "monospace".into(),
        size: -0.0,
        ..Font::default()
    };
    let css = theme_css(42, &colors, &font, -0.0);
    assert!(css.contains(".xterm-dom-renderer-owner-42.focus .xterm-selection div"));
    assert!(css.contains(".xterm-fg-257 { color: #000000; }"));
    assert!(!css.contains(".xterm-fg-0 {"));
    assert!(css.contains("font-size: 0px;"));
    assert!(css.contains("box-shadow: 0px 0 0"));
    assert!(css.contains("@keyframes blink_block_42"));
    assert!(!css.contains("blink_block_41"));
}
