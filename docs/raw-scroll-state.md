# Shared raw scroll state

Task 409 supplies the state foundation for task 301. It does not complete raw Scrollable, smooth animation or Element parity. Those live browser consumers still use numeric values. Overall equivalence remains NOT PROVEN.

## Implementation

`scroll/state.rs` now implements one `ScrollState<V>` algorithm. The default value type is `f64`, preserving existing native and browser callers. Numeric `new`, `with_dimensions` and `with_position` remain infallible. The generic `try_` methods propagate value-operator failures without publishing a partial state. The original numeric regression assertions remain intact.

`scroll/value.rs` defines the small operator contract and the native numeric policy. `scroll/raw_state.rs` implements it for `JsValue`, preserving raw strings, booleans, objects, null, undefined, signed zero, NaN, infinities and exceptions. Only JavaScript property reads and language operators cross the binding boundary. Constructor clamping, raw values, equality, updates and event flags remain Rust algorithms. No dependency, unsafe code, secondary cache or JavaScript terminal engine was added.

For `forceIntegerValues=true`, all six bitwise conversions happen in argument order before validation. With it false, values remain raw unless a source clamp replaces them. Addition precedes relational comparison; subtraction runs only for a clamp. Strict equality never coerces objects. Raw update entry points read inherited properties twice when the first value is defined, once otherwise, preserving the second result even if it becomes undefined. They propagate a getter's thrown value unchanged.

## Differential tests

The supplied source has no dedicated ScrollState unit suite. These are additional source-derived tests, not renamed upstream tests. `scripts/raw-scroll-state-source.mjs` extracts the unchanged `ScrollState` class and transpiles it with the already-installed esbuild. The committed test-only oracle includes the complete source file hash, `010ddcb39e40b268e4f37afc1da968ae30b64eaf356c918bb08484558150eba4`. Running the script without `--write` verifies exact fixture bytes. No source assertions or original inputs were edited.

`raw_scroll_state_browser.rs` invokes the shared Rust state directly. The generated oracle runs separately on independently constructed inputs. Three test functions compare 287 cases per engine:

- 180 primitive cases, varying each of six fields under both integer policies, including BigInt and Symbol failures.
- 27 coercible-object cases, checking source operator order, raw identity, concatenation and thrown-value identity.
- 80 dimension/position update cases, checking inherited getters, repeated reads, changed second values, absent properties, raw-position restoration and failures.

Each successful case compares eight stored values, six event flags and self-equality. Failed cases compare coercion/getter traces and thrown identity for fixture-owned errors, or exception class for newly constructed language errors. This component test does not exercise reentrant Scrollable mutation, smooth ticks or live DOM consumers.

All three functions passed in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 on native Windows. No failed, ignored or filtered tests. Raw reports and logs are under `results/raw-scroll-state/`.

## Commands and scope

Run from a worktree with the reference available. Set `ESBUILD_MODULE`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to existing matching installations when absent locally. Set `TEMP`, `TMP` and the Cargo target under the owned O: worktree. No dependency installation or reference junction was needed.

```sh
node scripts/raw-scroll-state-source.mjs
cargo test -p xterm-web --lib --test scrollable --test viewport_controller -j2
cargo clippy -p xterm-web --all-features --lib --test scrollable --test viewport_controller -j2 -- -D warnings
cargo clippy -p xterm-web --all-features --target wasm32-unknown-unknown --lib --test raw_scroll_state_browser -j2 -- -D warnings
CARGO_BUILD_JOBS=2 WASM_TEST=raw_scroll_state_browser node scripts/browser-host-check.mjs
```

The native run passed 55 library, five Scrollable and 11 ViewportController tests. Both focused warning-denied Clippy commands passed. Changed-file rustfmt checks passed. `cargo fmt --all -- --check` failed with Windows OS error 206 before formatting, so no full workspace-format pass is claimed. Full workspace/all-target/all-feature tests, release checks, Linux/macOS runtime results and final raw scrolling integration remain separate gates.

The source baseline was main `5082b32a`; existing scrolling recovery `eb5d871` and later decoration/ruler fixes were preserved. All new build and scratch paths were inside `O:/xterm-rust-swarm-16/tmp/worktrees/BasilissaRaw`. Retained JSON includes artifact hashes and historical execution paths. Generated bindings/WASM are disposable, not distributed source.

## Independent review and canonical integration

Task 409 review accepted producer commit `be16e334`. The reviewer inspected the common algorithm, raw operators, property access and source fixture, then independently reran all 71 native tests, 287 differential cases per browser and scoped strict Clippy/format checks. `independent-browser.json` and its three engine logs retain that browser rerun.

The unchanged canonical helper rebased the reviewed source onto current main `d29e4bad`, passed its gates and fast-forwarded main to `28261af8029d26455e58fb0818655473453b49b2`. It built and tested changed crate xterm-web, checked affected native targets, checked the workspace and checked applicable WASM libraries. Its default-feature execution passed 344 tests across 133 test summaries with no failed or ignored tests. `canonical-merge.log` and `integration.json` retain the exact scope. The helper removed the private target. This was not a post-rebase browser rerun or a workspace all-feature/release/platform matrix pass.

The owned worktree contains a physical reference copy, not a reference junction. Cleanup must first preserve these committed sources/results, confirm no process is using the worktree, scan for links/reparse points and inspect dirty/untracked files. Only this run's generated browser artifacts, scratch and physical worktree copy may be removed. Other owners' work, original references and retained branches remain untouched. The coordination thread records the final cleanup outcome; this paragraph is not a claim that a later deletion already ran.
