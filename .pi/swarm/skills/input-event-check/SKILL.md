---
name: input-event-check
description: Use when checking lazy InputEvent properties, duplicate suppression, getter reentry or clipboard/IME/keyboard DOM regressions against source method bodies.
---

# InputEvent checks

Read docs/input-event-dispatch.md. Run `node --test scripts/input-event-fixture.test.mjs` to verify both copied oracle bodies against unchanged source. No dependency installation is required when existing source_repo tooling is available.

Run scoped native checks with `cargo test -p xterm-web --test input_event_getters --test keyboard_dispatch_callbacks -j2`, plus warning-denied Clippy on those targets. Run each of `wasm_input_event`, `wasm_input`, `wasm_input_callbacks` and `wasm_keyboard_live` using `WASM_TEST=NAME node scripts/browser-host-check.mjs`. Use CARGO_BUILD_JOBS=2 and the current team resource agreement, not the retired build queue. Set WASM_BINDGEN and PLAYWRIGHT_MODULE to installed tools if needed. Put new private targets and TEMP/TMP on the authorized O: owned worktree.

The new suite has four test functions. Its guard test expands 288 combinations against unchanged CoreBrowserTerminal._inputEvent; these are not 288 upstream test definitions. Remaining tests cover exact exception values/read prefixes, empty second data and live-state getter reentry. The keyboard IME test compares its changing keyCode trace with the unchanged CompositionHelper.keydown body.

Keep failures as well as passes. Preserve concise browser reports, full output logs and hashes before deleting temporary artifacts. A scoped component pass does not establish full Terminal/API/addon parity. Admitted non-string second data remains a raw CoreService event integration gap, not a supported empty-text fallback.
