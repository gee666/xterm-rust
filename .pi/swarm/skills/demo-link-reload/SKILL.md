---
name: demo-link-reload
description: Use when the demo web-links reload case loses hover at the same pointer cell and source semantics must be checked before changing the Rust linkifier or custom probe.
---

# Demo link reload

Read `docs/demo-link-reload.md` and retained task327 failures first. Do not replay a full suite to restate historical counts.

```sh
node --test scripts/demo-link-probe.test.mjs
SOURCE_ROOT=/path/to/exact/source_repo node scripts/link-demo-reload-source.mjs
```

The unit controls test the actual probe's pointer path, not the terminal engine. The browser command checks unchanged bundles/sourcemaps, real first hover/popup, source same-cell negative, then different-row return and second hover/popup in Chromium, Firefox and WebKit. It writes `results/link-demo-reload/source.json`; retain prior attempts before rerunning. Set owned O: TEMP/TMP/TMPDIR and obtain a browser allocation before execution. No Cargo build is needed for the source run.

A Rust demo run separately requires exact current Rust-input/package identity and an explicit browser allocation. Preserve both pointer/popup assertions, original tests, old failed reports and host errors. Never clear private linkifier state or add a fake service to make the custom probe pass. No retired build-slot queue.
