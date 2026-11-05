# Live OSC callback ranges

Task440 fixes a concrete browser OSC link defect on main `dcde3e9c`. The old terminal adapter returned a stable JavaScript range to handlers, but its controller still used an immutable Rust range. A handler could move the range and observe later hit tests, mousedown equality and double-click selection ignore the change.

## Implementation

`NativeOscLinkProvider::with_javascript_handler` samples the raw option once for an existing row. It shares Rust OSC discovery and per-run protocol policy with the existing typed constructor. Missing rows still do not read options. The typed native constructor and its immutable Rust callback contract remain available without changes.

The browser path now creates `BrowserLink::JavaScript`. The existing Rust provider replies, overlap removal, hover state, pointer hit tests, mousedown equality, invalidation and selection all read that same retained payload. There is no second controller or scan algorithm. `raw_osc_bridge.js` constructs only binding objects and the three callback wrappers. The wrappers close over one range, use the original handler receiver, perform optional-nullish hover/leave calls and reread replaceable handler members. Source truthy default-activation selection is preserved even for primitive handlers.

The removed pointer-keyed range cache could preserve callback identity without preserving controller behavior. JS now owns the link/range/callback edges. Default activation uses a GC-owned Rust callback retaining only the browser window, not the terminal, provider or controller. No new dependency, unsafe code or JS terminal engine is added.

## Directed verification

From the repository root, with the matching installed Playwright and wasm-bindgen tools:

```sh
node scripts/public-link-provider-source.mjs --osc-live
WASM_TEST=wasm_osc_live_range node scripts/browser-host-check.mjs
cargo test --locked -p xterm-web --all-targets --all-features
cargo clippy --locked -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -p xterm-web --all-targets --all-features --target wasm32-unknown-unknown -- -D warnings
```

The source runner reuses the public-provider runner and checks the original bundle source map against OSC provider, Linkifier, selection, public Terminal and provider-service source bytes. Its fixture is generated only from that original. No original assertions or source files change.

The same 17-phase trace passes through original and exported Rust Terminal in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. It covers mutation during hover, same-cell suppression, later leave/reentry, mutation between mousedown and mouseup, double-click selection, replaced endpoint objects, ordered Proxy getters, exact thrown-object identity, replaced handler methods, wrapped links with nonzero scrollback, row-change cache replacement and disposal. Source selects `lmn` and `opq` after the two selection mutations.

A development assertion initially assumed a range survived a provider query on a different row. Source showed that the new row creates a fresh object. The retained development report and final trace explicitly distinguish those objects; the selection probe mutates the current callback range rather than an obsolete cached range.

A negative control swaps only the three changed existing production modules to exact main `dcde3e9c` bytes, runs the new test in Chromium and restores the fixed bytes in finally with exact SHA checks. The old implementation fails the actual browser assertion, keeping the pointer active outside the mutated range, missing activation and selecting the original full text. The restored implementation passes all three engines.

The retained OSC handler, policy, typed linkifier, public-provider and link-disposal suites pass 13 test functions per engine. With the new trace, six suites pass 14 test functions per engine, 42 expanded cases. These are not 42 upstream test suites. The original seven OSC/Linkifier assertions pass unchanged, and the regenerated 4,097-case seeded fixture remains byte-identical. Native xterm-web all-target/all-feature tests pass 360 tests over 148 summaries with zero failed or ignored tests. Native and WASM all-target/all-feature Clippy pass with warnings denied. All six changed Rust files pass rustfmt. Exact `cargo fmt --all -- --check` fails with Windows OS206; that failure is retained, not relabeled as a pass.

Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 were used on native Windows. Reports, commands, producer identities, negative-control hashes and raw quality logs are indexed by `results/osc-live-range/index.json`. Browser reports retain original temporary artifact paths and exact compiler/binding hashes. Binary build outputs are disposable, not evidence to commit.

## Remaining work

This scoped fix does not close selection/links task321 or public API task315. Utility web-links still uses typed snapshots. Arbitrary mapped reply containers, malformed/sparse payloads, out-of-domain drawing/selection coordinates, complete reentrant getter/Proxy coverage, randomized mutation and full lifecycle/GC equivalence remain open. Native Linux/macOS runtime, complete original/browser/demo matrices and final workspace quality are separate requirements. Full equivalence is NOT PROVEN.
