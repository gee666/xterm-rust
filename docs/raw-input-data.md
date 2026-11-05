# Raw InputEvent data

## Intent and scope

Continue the complete supplied xterm.js Rust port with exactly 16 coordinated agents. Preserve source assertions, browser/headless/addon contracts and native Windows/Linux/macOS requirements. Use reviewed integration on real main, owned O: temporary work and explicit cleanup. Overall equivalence remains NOT PROVEN.

Task356 fixes the concrete admitted non-string InputEvent.data gap from task319 and the verified Herodotus343 handoff. It does not close that whole family or replace the final original-suite, platform, feature/release or performance matrices.

## Implementation

- input/dom/keyboard.rs uses the existing guard/dead-key policy and rereads data only after admission. InputHost::data_value preserves the JS value.
- CoreService::trigger_data_event_with owns shared admission, buffer capture, scroll and user-input effects. Native trigger_data_event uses it too.
- terminal/input_data.rs performs eager String-hint debug conversion even with logs disabled, then uses LogService's dynamic-method/lazy-parameter policy. Rust invokes split, map and charCodeAt with their original receivers. Custom map return values are retained, not assumed to be arrays. A retained map callback is JS-GC-owned.
- One browser emitter receives forwarded native UTF-16 strings and identity-preserved raw values. It follows native disposal and supports synchronous nested delivery. The native UTF-16-only on_data API remains typed and does not receive non-string browser payloads.
- No dependency, unsafe code, source-engine delegation or original source/assertion change was added. One generic JS property-access operation supports boxing and native host errors. Generated engine TypeError names are compared; exact engine-specific error wording is not asserted.

## Verification

Durable receipts are in results/raw-input356. Run with the installed Rust1.85.1 compiler/Cargo by explicit executable paths for this host. Merely pointing CARGO at another version while allowing a rustup rustc shim produced retained setup failures. No toolchain or source reference was altered.

- Native core_data_delivery3 and core_service11 test functions pass. The new host-delivery test expands16 admission combinations, then tests option-changing reentry and errors. Existing input_event_getters6 and keyboard_dispatch_callbacks3 pass.
- Source fixture integrity7 tests pass. The new5 compare unchanged CoreService.triggerDataEvent and LogService lazy/log/trace/debug bodies with the supplied reference. Existing2 retain InputEvent and CompositionHelper assertions and hashes.
- wasm_input_raw2 functions expand90 source/Rust cases per engine:66 raw-kind/log-level/disableStdin combinations and24 object callback/error/reentry/disposal combinations. Null, undefined, booleans, numbers, NaN, negative zero, BigInt, Symbol, arrays, functions and objects are covered. Object String-hint, split/map/charCodeAt getters, receiver identities and throws are observed.
- Restoring the old string-only second-read admission in a temporary negative control makes both new functions fail in Chromium. The producer was restored in finally, then the full patched matrix passed.
- Patched Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 pass raw2, InputEvent4, input callbacks3, input7, keyboard-live15, terminal wheel5 and terminal writer5 test functions per engine. InputEvent additionally retains its288-case guard expansion. This is41 functions per engine, not41 source suites or complete port coverage.
- Native core/web all-target warning-denied Clippy, focused WASM library/raw/InputEvent warning-denied Clippy and exact cargo fmt --all -- --check pass.

After rebasing onto main5ca9f593 containing accepted354, the complete WASM library passes26 functions in each of the three engines, and the raw2-function90-case suite passes again. Both source and Rust full-composition assertions remain intact. Scoped native/WASM Clippy and fullfmt pass on the refreshed branch. Final integrated results belong in a separate canonical receipt. Standalone text logs omit trailing empty lines only; JSON receipts retain the complete browser output.

## Owned temporary work

Only O:/xterm-rust-swarm-16/tmp/A is owned. It was created as Agamemnon16 with the existing worktree helper, then safely moved from outside that directory for Windows rustfmt command-length limits. source_repo is the helper's physical reference copy, not a junction. Browser runners own and close their servers/browsers. Do not remove historical worktrees, original dependencies or another agent's artifacts. Preserve commits, regression fixtures and concise raw evidence before whole-worktree cleanup.
