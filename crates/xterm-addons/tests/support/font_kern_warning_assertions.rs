// Shared native/WASM assertions against unchanged-source constructor outcomes.
use super::corpus;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions, FontWarning};

pub fn assert_corpus() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/font-kern-warnings.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warnings = Vec::new();
        let sentinel = String::from("host callback failure");
        let actual = Font::load_buffer_with_warnings(
            &corpus::construct(case),
            FontOptions::default(),
            |warning| {
                assert_eq!(warning, FontWarning::MultipleKernSubtables);
                warnings.push(warning.message());
                if case["throws"] == true {
                    Err(&sentinel)
                } else {
                    Ok(())
                }
            },
        );
        let expected = case["outcome"]["status"].as_str().unwrap();
        match actual {
            Ok(_) => assert_eq!(expected, "ok", "{}", case["name"]),
            Err(FontLoadError::Font(error)) => {
                assert_eq!(expected, "error", "{}", case["name"]);
                assert_eq!(
                    Some(error),
                    corpus::constructor_error(case),
                    "{}",
                    case["name"]
                );
            }
            Err(FontLoadError::Warning(error)) => {
                assert_eq!(expected, "warning", "{}", case["name"]);
                assert!(std::ptr::eq(error, &sentinel));
            }
        }
        assert_eq!(
            serde_json::json!(warnings),
            case["warnings"],
            "{}",
            case["name"]
        );
    }
}

pub fn assert_reentry_and_earlier_errors() {
    let mut payload = vec![0; 18];
    payload[1] = 1;
    payload[7] = 2;
    let case = serde_json::json!({"tag":"kern", "data":payload});
    let mut count = 0;
    Font::load_buffer_with_warnings(&corpus::construct(&case), FontOptions::default(), |_| {
        count += 1;
        // An independent constructor does not share a borrow with its caller.
        Font::load_buffer(
            include_bytes!("../fixtures/ligature-fonts/UbuntuMono-Regular.ttf"),
            FontOptions::default(),
        )
        .map(|_| ())
    })
    .unwrap();
    assert_eq!(count, 1);
    for earlier in [
        serde_json::json!({"remove":["name"]}),
        serde_json::json!({"glyphCount":0}),
    ] {
        let mut invalid = case.clone();
        invalid
            .as_object_mut()
            .unwrap()
            .extend(earlier.as_object().unwrap().clone());
        let result = Font::load_buffer_with_warnings(
            &corpus::construct(&invalid),
            FontOptions::default(),
            |_| {
                count += 1;
                Ok::<(), FontError>(())
            },
        );
        assert!(matches!(result, Err(FontLoadError::Font(_))));
        assert_eq!(count, 1);
    }
}
