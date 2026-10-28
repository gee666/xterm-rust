// Copyright (c) 2023 The xterm.js authors. MIT license.
// The generated replay retains all operations from the 15 header and 1 metrics
// source test bodies after their 134 unchanged Chai assertions passed.
mod image_helper_support;
use serde_json::Value;
use std::collections::HashSet;
use xterm_addons::image::header::HeaderParser;

#[test]
fn replay_every_original_header_and_metrics_test_operation() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/image-helpers-upstream.json")).unwrap();
    let mut parser = HeaderParser::default();
    let mut names = HashSet::new();
    for (index, case) in fixtures.iter().enumerate() {
        names.insert(case["test"].as_str().unwrap());
        let actual = image_helper_support::apply(&mut parser, &case["operation"]).unwrap();
        assert_eq!(
            actual, case["expected"],
            "source test {} replay operation {index}",
            case["test"]
        );
    }
    assert_eq!(fixtures.len(), 539);
    assert_eq!(names.len(), 16);
}
