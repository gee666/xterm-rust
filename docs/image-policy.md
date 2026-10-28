# Native image control policy

Task224 ports `ImageAddon.ts` defaults, reset behavior, DEC80, DA1 and XTSMGRAPHICS into `image/options.rs` and `image/policy.rs`. Parser registration, browser activation and real images remain task38.

Control methods produce ordered replies/clear/reset effects. The executor must release policy borrows before callbacks and re-read current handler membership after a reply. Runtime reset restores only constructor palette/scroll settings. Native options use declared bool/f64 values; raw JS option coercion remains task205.

```sh
cargo test -p xterm-addons --test image_policy
node scripts/image-policy-oracle.mjs
cargo clippy -p xterm-addons --test image_policy --example image_policy_trace -- -D warnings
```

Results:7 Rust tests and2,357 unchanged-source comparisons pass, including2,000 seeded operations and reply reentry. Scoped Clippy passes. The source oracle constructs the original ImageAddon with its real emitter; recording terminal/storage handlers compare control effects only, not DOM or decoded pixels. Summary: `results/image-policy/summary.json`.
