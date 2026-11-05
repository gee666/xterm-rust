// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
use serde_json::{json, Value};
use xterm_web::webgl::custom_glyphs::{
    draw_custom_glyph, is_block_pattern, CustomGlyphDrawParams, DrawCommand as C, GlyphCanvas,
};

struct Recorder {
    trace: Vec<Value>,
    fill: Value,
    stack: Vec<Value>,
}
impl Recorder {
    fn new() -> Self {
        Self {
            trace: Vec::new(),
            fill: json!("#e06090"),
            stack: Vec::new(),
        }
    }
}
impl GlyphCanvas for Recorder {
    type Error = ();
    fn draw(&mut self, command: C) -> Result<(), ()> {
        let entry = match command {
            C::Save => {
                self.stack.push(self.fill.clone());
                json!(["save"])
            }
            C::Restore => {
                self.fill = self.stack.pop().unwrap();
                json!(["restore"])
            }
            C::Begin => json!(["begin"]),
            C::Close => json!(["close"]),
            C::Move(p) => json!(["move", p[0], p[1]]),
            C::Line(p) => json!(["line", p[0], p[1]]),
            C::Quadratic(p) => json!(["quadratic", p[0], p[1], p[2], p[3]]),
            C::Cubic(p) => json!(["cubic", p[0], p[1], p[2], p[3], p[4], p[5]]),
            C::Arc(p) => json!(["arc", p[0], p[1], p[2], p[3], p[4]]),
            C::Ellipse(p, r) => json!(["ellipse", p[0], p[1], p[2], p[3], p[4], p[5], p[6], r]),
            C::Fill => json!(["fill"]),
            C::Stroke => json!(["stroke"]),
            C::Rect(p) => json!(["rect", p[0], p[1], p[2], p[3]]),
            C::FillRect(p) => json!(["fillRect", p[0], p[1], p[2], p[3]]),
            C::Clip => json!(["clip"]),
            C::ClipRect(p) => json!(["clipRect", p[0], p[1], p[2], p[3]]),
            C::StrokeFromFill => json!(["strokeStyle", self.fill]),
            C::Background(color) => {
                self.fill = json!(color);
                self.trace.push(json!(["fillStyle", color]));
                json!(["strokeStyle", color])
            }
            C::LineWidth(w) => json!(["lineWidth", w]),
            C::SquareCap => json!(["lineCap", "square"]),
            C::Pattern(_, _, v) => {
                self.fill = json!({"variant":v&3});
                json!(["pattern", v & 3])
            }
            C::LogError(message) => json!(["error", message]),
        };
        self.trace.push(entry);
        Ok(())
    }
}

pub fn params(p: &Value) -> CustomGlyphDrawParams<'static> {
    let n = |i| p[i].as_f64().unwrap();
    CustomGlyphDrawParams {
        x_offset: n(0),
        y_offset: n(1),
        device_cell_width: n(2),
        device_cell_height: n(3),
        device_char_width: n(4),
        device_char_height: n(5),
        font_size: n(6),
        device_pixel_ratio: n(7),
        background_color: Some("#112233"),
        variant_offset: 3,
    }
}
fn close(actual: &Value, expected: &Value, location: &str, arc: bool) {
    if let (Some(a), Some(b)) = (actual.as_f64(), expected.as_f64()) {
        // Native libm and JS Math use different implementations of transcendental
        // functions. Canvas pixel comparisons remain exact in the browser suite.
        if arc {
            assert!(
                (a - b).abs() <= 1e-12 * b.abs().max(1.0),
                "{location}: {a} != {b}"
            );
        } else {
            assert_eq!(a, b, "{location}");
        }
    } else if let (Some(a), Some(b)) = (actual.as_array(), expected.as_array()) {
        assert_eq!(a.len(), b.len(), "{location}");
        for (i, (a, b)) in a.iter().zip(b).enumerate() {
            close(
                a,
                b,
                &format!("{location}/{i}"),
                arc || expected[0] == "ellipse",
            );
        }
    } else {
        assert_eq!(actual, expected, "{location}");
    }
}

#[test]
fn all_source_glyph_traces() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/custom-glyphs.json")).unwrap();
    assert_eq!(fixture["glyphs"], 778);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2355);
    for (i, case) in cases.iter().enumerate() {
        let chars: Vec<_> = case["chars"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u16)
            .collect();
        let mut canvas = Recorder::new();
        let result = draw_custom_glyph(&mut canvas, &chars, params(&case["p"])).unwrap();
        assert_eq!(result, case["result"].as_bool().unwrap(), "case{i}");
        close(
            &json!(canvas.trace),
            &case["trace"],
            &format!("case{i} chars{chars:?}"),
            false,
        );
    }
    let patterns: Vec<_> = fixture["patternCodepoints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    for cp in 0..=0x1ffff {
        assert_eq!(is_block_pattern(cp), patterns.contains(&cp), "U+{cp:x}");
    }
}

#[test]
fn canvas_failure_stops_without_restore() {
    struct Fail {
        calls: usize,
    }
    impl GlyphCanvas for Fail {
        type Error = usize;
        fn draw(&mut self, _: C) -> Result<(), usize> {
            self.calls += 1;
            if self.calls == 3 {
                Err(3)
            } else {
                Ok(())
            }
        }
    }
    let mut canvas = Fail { calls: 0 };
    let p = json!([0, 0, 8, 16, 8, 16, 15, 1]);
    assert_eq!(
        draw_custom_glyph(&mut canvas, &[0x2500], params(&p)),
        Err(3)
    );
    assert_eq!(canvas.calls, 3);
}
