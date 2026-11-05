# Attach listener reentry

Overall equivalence is NOT PROVEN. The full request remains in `docs/intent.md` and `PORT_REQUIREMENTS.md`. Task404 is a focused repair under full utility/status closure task323, not a substitute for it.

## Scope and intent

Preserve unchanged `addon-attach/src/AttachAddon.ts` synchronous socket callback behavior through the Rust browser adapter. Nested message dispatch must complete in source order, getter exceptions must keep their original identity, and disposal inside message processing must not suppress the already-running source callback or trap.

The shared fixture compares unchanged source methods with a recording terminal against real Rust Terminal writes. It covers plain, recursive, binary and empty messages, inner getter exceptions, caught exceptions, and explicit/close/error disposal inside the outer data getter. A second test uses an actual connected WebSocket and real Rust Terminal to verify nested EventTarget dispatch and listener removal. No original source or assertion is modified. These are focused source-method and integration checks, not complete original addon or demo suites.

## Repair and results

The browser listener used `Box<dyn FnMut(Event)>` despite keeping no mutable closure state. wasm-bindgen forbids calling that wrapper recursively. The repair uses `Box<dyn Fn(Event)>`; the existing Rust callback policy, queued writes, exception conversion, listener registration and disposal stay unchanged. No dependency, unsafe code, native API or JavaScript terminal implementation was added.

`results/attach-reentry/` retains the observations on native Windows with Node24.15.0, Rust/cargo1.85.1 and wasm-bindgen0.2.100:

- `source.json`:11 source-method cases discovered/executed/passed, zero failed/skipped/unrun. All776 reference-manifest entries were unchanged before and after. Source, shared trace and expected fixture SHA-256 hashes are recorded.
- `before.json`: both new test functions fail in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Recursive callbacks trap before the inner data getter. The actual connected WebSocket renders A instead of BA and reports `closure invoked recursively or after being dropped`.
- `after.json`: both new functions pass in each engine, including33 source-matched traces and three actual connected-WebSocket runs. Explicit, close and error disposal during message data access still permit the active outer write, just as source does.
- `wasm_utility_addons.json` and `wasm_utility_bindings.json`: three existing test functions each pass in all three engines. Including the new suite,24 browser test executions pass, zero failed/ignored. The existing component suite includes a real WebSocket delivering string and binary data; exported clipboard and constructor checks are also rerun.
- `native.log`:16 native utility_attach/utility_clipboard/utility_oracle/utility_live_terminal tests pass. `clippy-wasm.log`: warning-denied Clippy passes for all three browser targets. The first Clippy run rejected option_env().expect() in the new test; `clippy-wasm-before.log` retains that failure, and the test now uses the established explicit missing-setup panic pattern.
- `fmt.log`: exact `cargo fmt --all -- --check` fails with Windows OS206. Formatting all five workspace packages separately passes in `fmt-packages.log`. This is not represented as an exact full-command pass. Full workspace all-feature quality, native Linux/macOS runtime, complete original suites and full utility/demo closure remain separate gates.

## Reproduce

Reuse the source-only Node dependencies, Playwright engines and wasm-bindgen0.2.100 with SOURCE_ROOT, NODE_PATH, PLAYWRIGHT_MODULE and WASM_BINDGEN. The WebSocket wrapper reads `ws` through NODE_PATH when the physical worktree reference has no node_modules. Keep scratch and builds on O: and respect team resource limits.

```sh
node scripts/attach-reentry-oracle.mjs
WASM_TEST=wasm_attach_reentry node scripts/utility-addon-browser-check.mjs
WASM_TEST=wasm_utility_addons node scripts/utility-addon-browser-check.mjs
WASM_TEST=wasm_utility_bindings node scripts/browser-host-check.mjs
cargo test -j2 -p xterm-addons --test utility_attach --test utility_clipboard --test utility_oracle --test utility_live_terminal
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_attach_reentry --test wasm_utility_addons --test wasm_utility_bindings -- -D warnings
cargo fmt --all -- --check
```

The oracle removes its scratch bundle in finally. Use `--generate` only after reviewing a real source or trace change. Never alter the original source to clear a failure. Browser wrappers close all browser processes, the HTTP listener and every WebSocket peer in finally. Retained JSON reports include historical temporary artifact paths and hashes, not committed WASM binaries.

## Temporary ownership

Only `O:/xterm-rust-swarm-16/tmp/E` is owned by this task. It contains its private target, source reference copy, scratch bundles and generated browser artifacts. Existing source dependencies and wasm-bindgen are reused read-only through environment paths. Preserve regression source and concise reports on actual main before removing this worktree. No source junctions, persistent server, global build queue or credential changes are needed.
