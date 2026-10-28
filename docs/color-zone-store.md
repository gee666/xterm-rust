# Color zone store

`crates/xterm-web/src/renderer/color_zone_store.rs` ports `browser/decorations/ColorZoneStore.ts`. It supplies ordered overview zones to the renderer but does not itself draw a canvas. Task32 retains actual overview lanes, DPR scaling and browser pixel/DOM integration.

Zones, the active zone list and padding options retain shared identity through Rc/RefCell handles. A getter trims unused pool references without resetting the source pool cursor. Clear empties the same list and resets its cursor. Old retained zone handles observe subsequent reuse. An absent overview setting creates no zone. Color and position equality are exact, including the distinction between absent position and full. Absent position only falls back to full for padding lookup. Missing padding properties behave like JavaScript undefined and disable adjacency. Intersection is tested before adjacency, so negative/NaN padding does not prevent a directly intersecting line from being ignored. Only the first matching zone extends; bridging lines do not union later zones.

Colors are lossless UTF16 vectors, positions use the shared OverviewPosition enum, and line/padding numbers retain f64 behavior. Rust borrow conflicts are returned as typed errors. Release external borrows before invoking store methods. The source pool and publicly retained objects remain mutable; the Rust API does not replace identity with cloned value snapshots.

## Reproduction

```sh
node scripts/color-zone-oracle.mjs
cargo test -p xterm-web --test color_zone_store --test color_zone_differential
cargo clippy -p xterm-web --test color_zone_store --test color_zone_differential -- -D warnings
```

Use `--generate` explicitly to regenerate the fixture from original JavaScript. Default execution checks for drift and does not rewrite tracked expected values. Source hashes and exact counts are in `results/color-zone/oracle.json`.

The original three tests pass unchanged under Mocha. Seed0xc01020ae generates2048 snapshots comparing ordered zone values, object identity and retained old handles through add, clear/reuse, padding replacement and zone getters. The Rust files map all three source assertions and add seven deterministic boundary/identity tests plus one differential replay. After the reviewed shared-target cleanup, all 11 scoped Rust tests pass in debug and release. Exact logs are `results/terminal-review/current-zones.log` and `release.log`, with disk observations in `controlled-native.json` and `controlled-quality.json`. The default JavaScript oracle also verifies unchanged fixture bytes in `results/color-zone/oracle-current.log`. Scoped formatting passes. Focused web Clippy is currently stopped by unrelated image-addon precedence and NaN-sensitive range warnings, retained in `results/terminal-review/zones-clippy-clean.log`. Independent source/code review is still required; this is not a whole-workspace or browser-rendering equivalence claim.

Overall equivalence is NOT PROVEN. This store alone does not establish actual overview rendering, browser E2E or native Linux/macOS evidence.
