//! Replay original assertion inputs plus seeded cross-font calls and identity.
use super::*;
use serde_json::{json, Value};

pub(super) fn font_bytes(name: &str) -> &'static [u8] {
    match name {
        "FiraCode-Regular.otf" => {
            include_bytes!("../../tests/fixtures/ligature-fonts/FiraCode-Regular.otf")
        }
        "iosevka-regular.ttf" => {
            include_bytes!("../../tests/fixtures/ligature-fonts/iosevka-regular.ttf")
        }
        "Monoid-Regular.ttf" => {
            include_bytes!("../../tests/fixtures/ligature-fonts/Monoid-Regular.ttf")
        }
        "UbuntuMono-Regular.ttf" => {
            include_bytes!("../../tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
        }
        other => panic!("unknown fixture font {other}"),
    }
}
pub(super) fn array(value: &Value) -> &Vec<Value> {
    value.as_array().unwrap()
}
pub(super) fn integer(value: &Value) -> usize {
    usize::try_from(value.as_u64().unwrap()).unwrap()
}
pub(super) fn glyph(value: GlyphValue) -> Value {
    match value {
        GlyphValue::Number(value) => json!(value),
        GlyphValue::NaN => json!("NaN"),
        GlyphValue::Undefined => json!("undefined"),
    }
}
#[derive(Clone)]
enum Handle {
    Data(LigatureResult),
    Glyphs(GlyphArray),
    Ranges(RangeArray),
}
impl Handle {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Data(a), Self::Data(b)) => Rc::ptr_eq(a, b),
            (Self::Glyphs(a), Self::Glyphs(b)) => Rc::ptr_eq(a, b),
            (Self::Ranges(a), Self::Ranges(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
fn handle_id(handles: &mut Vec<Handle>, value: Handle) -> usize {
    if let Some(index) = handles.iter().position(|existing| existing.same(&value)) {
        return index;
    }
    handles.push(value);
    handles.len() - 1
}
#[test]
fn all_original_and_seeded_font_outputs_ranges_and_cache_handles_match() {
    let trace: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/ligature-font-trace.json"
    ))
    .unwrap();
    let instances = array(&trace["instances"]);
    let calls = array(&trace["calls"]);
    let mut remaining = vec![0; instances.len()];
    for call in calls {
        remaining[integer(&call["instance"])] += 1;
    }
    let mut fonts: Vec<Option<Font>> = (0..instances.len()).map(|_| None).collect();
    let mut handles = Vec::new();
    for (row, call) in calls.iter().enumerate() {
        let id = integer(&call["instance"]);
        if fonts[id].is_none() {
            let instance = &instances[id];
            fonts[id] = Some(
                Font::load_buffer(
                    font_bytes(instance["font"].as_str().unwrap()),
                    FontOptions {
                        cache_size: integer(&instance["cacheSize"]),
                    },
                )
                .unwrap_or_else(|error| panic!("constructor {id}: {error}")),
            );
        }
        let text: Vec<u16> = array(&call["input"])
            .iter()
            .map(|value| u16::try_from(integer(value)).unwrap())
            .collect();
        let font = fonts[id].as_mut().unwrap();
        let actual = if call["method"] == "full" {
            font.find_ligatures(&text).map(|result| {
                let data = result.borrow();
                let input: Vec<_> = data
                    .input_glyphs
                    .borrow()
                    .iter()
                    .copied()
                    .map(glyph)
                    .collect();
                let output: Vec<_> = data
                    .output_glyphs
                    .borrow()
                    .iter()
                    .copied()
                    .map(glyph)
                    .collect();
                let ranges = data.context_ranges.borrow().clone();
                let ids = vec![
                    handle_id(&mut handles, Handle::Data(Rc::clone(&result))),
                    handle_id(&mut handles, Handle::Glyphs(Rc::clone(&data.input_glyphs))),
                    handle_id(&mut handles, Handle::Glyphs(Rc::clone(&data.output_glyphs))),
                    handle_id(
                        &mut handles,
                        Handle::Ranges(Rc::clone(&data.context_ranges)),
                    ),
                ];
                json!({"inputGlyphs":input,"outputGlyphs":output,"ranges":ranges,"handles":ids})
            })
        } else {
            font.find_ligature_ranges(&text).map(|ranges| {
                let contents = ranges.borrow().clone();
                let id = handle_id(&mut handles, Handle::Ranges(ranges));
                json!({"ranges":contents,"handles":[id]})
            })
        };
        if let Some(error) = call.get("error") {
            let actual = actual.unwrap_err();
            assert!(
                matches!(actual, FontError::CacheEmptyKey),
                "row{row}: {actual}"
            );
            assert_eq!(error["name"], "TypeError");
            assert_eq!(
                actual.to_string(),
                error["message"].as_str().unwrap(),
                "row{row}"
            );
        } else {
            let actual = actual.unwrap_or_else(|error| panic!("row{row}: {error}"));
            for (key, value) in actual.as_object().unwrap() {
                assert_eq!(value, &call[key], "row{row} field{key}");
            }
        }
        remaining[id] -= 1;
        if remaining[id] == 0 {
            fonts[id] = None;
        }
    }
    assert_eq!(calls.len(), 2540);
}
#[test]
fn cached_results_preserve_external_mutation_and_uncached_array_alias() {
    let bytes = font_bytes("FiraCode-Regular.otf");
    let mut font = Font::load_buffer(bytes, FontOptions { cache_size: 100 }).unwrap();
    let text: Vec<_> = "==".encode_utf16().collect();
    let first = font.find_ligatures(&text).unwrap();
    first.borrow().output_glyphs.borrow_mut()[0] = GlyphValue::Number(777);
    first.borrow().context_ranges.borrow_mut().push([99, 100]);
    let second = font.find_ligatures(&text).unwrap();
    assert!(Rc::ptr_eq(&first, &second));
    assert_eq!(
        second.borrow().output_glyphs.borrow()[0],
        GlyphValue::Number(777)
    );
    let ranges = font.find_ligature_ranges(&text).unwrap();
    assert!(Rc::ptr_eq(&ranges, &first.borrow().context_ranges));
    assert_eq!(ranges.borrow().last(), Some(&[99, 100]));
    let mut font = Font::load_buffer(
        font_bytes("UbuntuMono-Regular.ttf"),
        FontOptions { cache_size: 100 },
    )
    .unwrap();
    let first = font.find_ligatures(&text).unwrap();
    assert!(Rc::ptr_eq(
        &first.borrow().input_glyphs,
        &first.borrow().output_glyphs
    ));
    let second = font.find_ligatures(&text).unwrap();
    assert!(!Rc::ptr_eq(&first, &second));
}
#[test]
fn woff_compressed_tables_match_actual_javascript_font_outputs() {
    let bytes = include_bytes!("../../tests/fixtures/ligature-fonts/Monoid-Regular.woff");
    let mut font = Font::load_buffer(bytes, FontOptions::default()).unwrap();
    let cases: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/ligature-woff.json")).unwrap();
    for case in array(&cases) {
        let text: Vec<_> = array(&case["input"])
            .iter()
            .map(|value| u16::try_from(integer(value)).unwrap())
            .collect();
        let result = font.find_ligatures(&text).unwrap();
        let result = result.borrow();
        assert_eq!(
            json!(result
                .input_glyphs
                .borrow()
                .iter()
                .copied()
                .map(glyph)
                .collect::<Vec<_>>()),
            case["inputGlyphs"]
        );
        assert_eq!(
            json!(result
                .output_glyphs
                .borrow()
                .iter()
                .copied()
                .map(glyph)
                .collect::<Vec<_>>()),
            case["outputGlyphs"]
        );
        assert_eq!(json!(*result.context_ranges.borrow()), case["ranges"]);
        assert_eq!(
            json!(*font.find_ligature_ranges(&text).unwrap().borrow()),
            case["rangeOnly"]
        );
    }
    assert_eq!(array(&cases).len(), 6);
}

#[test]
fn malformed_containers_return_errors_without_panicking() {
    for length in 0..12 {
        assert!(Font::load_buffer(&vec![0; length], FontOptions::default()).is_err());
    }
    assert!(Font::load_buffer(b"wOF2", FontOptions::default()).is_err());
    let source = font_bytes("FiraCode-Regular.otf");
    for length in [12, 13, 31, 63, 127] {
        assert!(Font::load_buffer(&source[..length], FontOptions::default()).is_err());
    }
}
