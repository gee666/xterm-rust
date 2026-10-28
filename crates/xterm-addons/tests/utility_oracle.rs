// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use serde_json::Value;
use xterm_addons::{base64, fit::*, web_links::*};

fn corpus() -> Value {
    serde_json::from_str(include_str!("fixtures/utility-addons.json")).unwrap()
}
fn units(value: &Value) -> Vec<u16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| u16::try_from(v.as_u64().unwrap()).unwrap())
        .collect()
}
fn number(value: &Value) -> f64 {
    match value.as_str() {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => value.as_f64().unwrap(),
    }
}
fn equal_number(actual: f64, expected: &Value) -> bool {
    let expected = number(expected);
    actual == expected || actual.is_nan() && expected.is_nan()
}

#[test]
fn base64_matches_source_utf16_and_malformed_utf8() {
    let corpus = corpus();
    let cases = corpus["base64"].as_array().unwrap();
    assert_eq!(cases.len(), 1307);
    for (i, case) in cases.iter().enumerate() {
        let input = units(&case["input"]);
        let actual = if case["op"] == "encode" {
            base64::encode_text(&input)
                .encode_utf16()
                .collect::<Vec<_>>()
        } else {
            base64::decode_text(&input)
        };
        assert_eq!(actual, units(&case["output"]), "base64 case {i}");
    }
}

#[test]
fn fit_matches_source_css_scrollbar_and_geometry() {
    let corpus = corpus();
    let cases = corpus["fit"].as_array().unwrap();
    assert_eq!(cases.len(), 512);
    for (i, case) in cases.iter().enumerate() {
        let m = &case["m"];
        let measurements = FitMeasurements {
            cell_width: number(&m["cell_width"]),
            cell_height: number(&m["cell_height"]),
            parent_width: m["parent_width"].as_str().unwrap().into(),
            parent_height: m["parent_height"].as_str().unwrap().into(),
            padding: std::array::from_fn(|j| m["padding"][j].as_str().unwrap().into()),
            scrollback: number(&m["scrollback"]),
            show_scrollbar: m["show_scrollbar"].as_bool(),
            scrollbar_width: m["scrollbar_width"].as_f64(),
        };
        match propose_dimensions(&measurements) {
            None => assert!(case["output"].is_null(), "fit case {i}"),
            Some(d) => assert!(
                equal_number(d.cols, &case["output"]["cols"])
                    && equal_number(d.rows, &case["output"]["rows"]),
                "fit case {i}: {d:?} expected {}",
                case["output"]
            ),
        }
    }
}

struct Line {
    wrapped: bool,
    cells: Vec<LinkCell>,
}
impl LinkLine for Line {
    fn is_wrapped(&self) -> bool {
        self.wrapped
    }
    fn len(&self) -> usize {
        self.cells.len()
    }
    fn cell(&self, index: usize) -> Option<LinkCell> {
        self.cells.get(index).cloned()
    }
    fn translate_trimmed(&self) -> Vec<u16> {
        let mut result = Vec::new();
        for cell in self.cells.iter().filter(|c| c.width != 0) {
            if cell.chars.is_empty() {
                result.push(32);
            } else {
                result.extend(&cell.chars);
            }
        }
        while result.last() == Some(&32) {
            result.pop();
        }
        result
    }
}
struct Buffer(Vec<Line>);
impl LinkBuffer for Buffer {
    fn line(&self, index: usize) -> Option<&dyn LinkLine> {
        self.0.get(index).map(|line| line as &dyn LinkLine)
    }
}
struct Matcher(Vec<LinkMatch>);
impl LinkMatcher for Matcher {
    fn matches(&self, _: &[u16], _: &RegexSpec) -> Result<Vec<LinkMatch>, LinkError> {
        Ok(self.0.clone())
    }
}

#[test]
fn web_link_window_and_utf16_mapping_match_source() {
    let corpus = corpus();
    assert_eq!(STRICT_URL_REGEX, corpus["regex"].as_str().unwrap());
    let cases = corpus["links"].as_array().unwrap();
    assert_eq!(cases.len(), 1765);
    for (i, case) in cases.iter().enumerate() {
        let buffer = Buffer(
            case["lines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|line| Line {
                    wrapped: line["wrapped"].as_bool().unwrap(),
                    cells: line["cells"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|cell| LinkCell {
                            chars: units(&cell["chars"]),
                            width: u8::try_from(cell["width"].as_u64().unwrap()).unwrap(),
                        })
                        .collect(),
                })
                .collect(),
        );
        let matcher = Matcher(
            case["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| LinkMatch {
                    index: usize::try_from(m["index"].as_u64().unwrap()).unwrap(),
                    text: units(&m["text"]),
                })
                .collect(),
        );
        let actual = compute_links(
            usize::try_from(case["y"].as_u64().unwrap()).unwrap(),
            &RegexSpec::default(),
            &buffer,
            &matcher,
        )
        .unwrap();
        let expected = case["expected"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "links case {i}");
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.text, units(&expected["text"]), "links case {i}");
            for (position, name) in [(actual.range.start, "start"), (actual.range.end, "end")] {
                assert_eq!(
                    position.x as u64,
                    expected["range"][name]["x"].as_u64().unwrap(),
                    "links case {i}"
                );
                assert_eq!(
                    position.y as u64,
                    expected["range"][name]["y"].as_u64().unwrap(),
                    "links case {i}"
                );
            }
        }
    }
}
