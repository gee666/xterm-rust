#![cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
use xterm_web::canvas_css::CanvasColorParser;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn canvas_named_hsl_and_fast_colors_match_upstream() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parser = CanvasColorParser::new(&document).unwrap();
    for (css, rgba) in [
        ("red", 0xff0000ff),
        ("blue", 0x0000ffff),
        ("rebeccapurple", 0x663399ff),
        ("hsl(120, 100%, 50%)", 0x00ff00ff),
        ("#AbCdEf", 0xabcdefff),
        ("transparent", 0),
        ("rgba(255, 0, 0, 0.5)", 0xff000080),
    ] {
        let color = parser.to_color(css).unwrap();
        assert_eq!(color.rgba, rgba, "{css}");
        if !css.starts_with("rgba") {
            assert_eq!(color.css, css);
        }
    }
}
#[wasm_bindgen_test]
fn invalid_and_nonopaque_fallbacks_are_rejected_without_stale_pixels() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parser = CanvasColorParser::new(&document).unwrap();
    for css in ["red", "red", "blue", "red"] {
        assert!(parser.to_color(css).is_ok());
    }
    for css in ["not-a-color", "hsl(120 100% 50% / 0.5)", "", "#gggg"] {
        assert!(parser.to_color(css).is_err(), "{css}");
    }
    assert_eq!(parser.to_color("blue").unwrap().rgba, 0x0000ffff);
}
