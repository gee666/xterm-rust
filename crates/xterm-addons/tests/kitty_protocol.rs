// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Recorded/deferred display is a protocol boundary, not successful pixel proof.
#[path = "support/kitty.rs"]
mod kitty;
#[path = "support/kitty_runtime.rs"]
mod runtime;

use serde_json::Value;

#[test]
fn original_kitty_protocol_state_replies_and_async_phases_match_native() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/kitty-protocol.json")).unwrap();
    assert_eq!(corpus.len(), 235);
    let mut count = 0;
    for row in corpus {
        let name = row["name"].as_str().unwrap();
        let operations = row["operations"].as_array().unwrap();
        let snapshots = row["snapshots"].as_array().unwrap();
        assert_eq!(operations.len(), snapshots.len(), "{name}");
        let mut actual = runtime::Runtime::new(row["limit"].as_f64().unwrap());
        for (step, operation) in operations.iter().enumerate() {
            actual.step(operation);
            assert_eq!(
                actual.snapshot(),
                snapshots[step],
                "trace {name} step {step}, operation {operation}"
            );
            count += 1;
        }
        assert!(actual.finished(), "unconsumed future in {name}");
    }
    assert_eq!(count, 7129);
}
