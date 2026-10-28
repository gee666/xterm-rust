# Decoder review 217

Read the fixed-slice adapter and UTF-8 pending-state change against the original decoder. In an isolated worktree, compiled the unchanged decoder module and its two test files standalone: all 11 tests passed. A source-derived fuzz run with seed `0x217d` passed 5,000 public-interim states, checking both Vec and fixed outputs, logical counts and next state.

Full `cargo test` was initially blocked by unrelated committed-base integration errors. This review accepts the decoder change only, not the whole core build.
