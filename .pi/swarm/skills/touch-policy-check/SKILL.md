---
name: touch-policy-check
description: Use when checking touch rolling movement, hold/tap decisions, clock boundaries or inertia against the unchanged source Gesture class.
---

Run `node scripts/touch-oracle.mjs` with Node24+ and source_repo build dependencies installed. It checks639traces and8061snapshots against the committed fixture. Use --generate only after reviewing an intentional reference change. No original touch-specific suite is claimed.

Run `cargo test -p xterm-web --test touch_policy`, the same with --release, targeted Clippy -Dwarnings and rustfmt on input/touch.rs plus tests/touch_policy.rs. Keep artifacts in the sharedtarget; honor .cargo/config.toml and check free disk before largebuilds.

Read docs/touch-policy.md and docs/acceptance/touch-policy.md for exact clocks, samplehistory, strictthresholds, NaN/Infinity/-0 and finalzero-frame behavior. Source hashes/seed/counts are results/touch-policy/oracle.json. Native policy success does not establish task108 DOM dispatch/target/reentry or real browser touch-to-scroll integration.
