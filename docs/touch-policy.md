# Portable touch gesture policy

`crates/xterm-web/src/input/touch.rs` ports the arithmetic in `source_repo/src/browser/scrollable/touch.ts`. It retains the Microsoft MIT notice. It has no native runtime dependencies, browser calls or unsafe code. The WASM-only re-export points to Joyce's document-owned `GestureContext` adapter in task 108.

`TouchData<T>` retains an initial target and initial sample plus the original four-sample rolling window. `last()` and `translation(x,y)` observe without mutating. The DOM adapter must dispatch the prepared movement before calling `commit_move(now,x,y)` on the captured record. A reentrant start may replace the active-map entry while that older record remains alive; the portable helper does not borrow or own the active map.

`end(hold_now,inertia_start)` returns `EndGesture::Tap`, `ContextMenu` or `Inertia`. The two clocks are separate because the source reads Date.now separately for the end timestamp and hold calculation. The source hold threshold is 700ms and both coordinate displacements must be strictly less than 30px. Inertia uses first/last samples in the rolling window, while hold/distance retains the original sample. The DOM adapter decides whether the active-touch-count snapshot allows scheduling inertia.

`Inertia::step(now)` applies the original -0.005/ms friction before displacement and returns `InertiaFrame` with both translations and a `continues` flag. The final zero frame is emitted. Scheduling, cancellation and insertion-order target snapshots belong to `GestureContext`, not to this arithmetic helper.

`TapCount` preserves the original one-or-two count and strict 400ms comparison. Its reset only assigns timestamp zero. The DOM adapter must call reset for ordinary CHANGE/CONTEXT_MENU before ignore-target filtering, not for START/END or inertial CHANGE. Nonfinite and backwards clock arithmetic remains f64 arithmetic rather than an invented clamp. Infinite velocity can remain positive until cancellation, just as source code can; tests bound frame steps rather than spinning an animation.

The exact integration API is recorded in `docs/touch-contract.md`. Detailed acceptance is `docs/acceptance/touch-policy.md` and criteria 398-400. Native policy verification does not close the DOM/browser criterion.

## Original-source oracle

There is no supplied touch-specific test suite. `scripts/touch-oracle.mjs` bundles and executes the unchanged supplied Gesture class with its real Dom/Lifecycle/Async helpers. Only its environment is replaced:

- A Node tree supplies containment, simple CustomEvent records and target dispatch logging.
- A controlled clock supplies Date.now and new Date().getTime values, including injected nonfinite values to test the arithmetic outside ordinary wall-clock ranges.
- A queued requestAnimationFrame host advances a specified single turn. Disposal flushes cancelled work once, never an unbounded animation.
- Tests call the original private start/move/end/dispatch methods directly rather than using touch hardware.

The oracle does not replace movement, hold, tap or inertia method bodies. It retains source hashes for all bundled modules and uses the original animation-queue helper. The Node host maintains listener registries and checks that source disposal releases them. Those registries are environment substitutes, not evidence that actual browser DOM listeners work. Host warnings and errors fail the policy oracle rather than being swallowed.

The committed fixture contains 639 traces: 127 directed cases and 512 xorshift32 cases using seed `0x704c4b31`. It includes 2,251 movement calls, 3,353 frame attempts and 8,061 per-operation snapshots. Encoded numbers retain NaN, positive/negative infinity and negative zero. Rust compares exact f64 bits except that all NaN values compare as JavaScript NaN. Every failure names seed, case, step, operation and the complete retained trace.

The Rust suite has six directed policy tests plus the complete fixture replay. It covers non-mutating movement preparation, strict hold/distance boundaries, four-sample velocity versus initial hold position, final zero frames, equal/backwards/nonfinite clocks, tap reset boundaries and retained old-record independence. The 639-line fixture is generated source-result data and intentionally exceeds the normal hand-written source length limit.

## Commands

Run from the repository root with Node 24 or later, the original reference build dependencies installed, and the Rust toolchain:

```text
node scripts/touch-oracle.mjs
cargo test -p xterm-web --test touch_policy
cargo test -p xterm-web --release --test touch_policy
cargo clippy -p xterm-web --test touch_policy -- -D warnings
rustfmt --edition 2021 --check crates/xterm-web/src/input/touch.rs crates/xterm-web/tests/touch_policy.rs
```

Use `node scripts/touch-oracle.mjs --generate` only after reviewing an intended reference change. Normal execution checks the fixture, rather than rewriting expected results. `results/touch-policy/oracle.json` records hashes, seed, counts, platform, versions and substitutions.

An initial integrated Cargo run encountered an unrelated in-progress buffer reflow symbol change. A focused exact-source rustc harness under `tmp/`, with `-D warnings` and debug information disabled, passed the initial seven tests and 8,036 snapshots. Five further directed traces added initial nonfinite/large/fractional coordinates and timestamps for 8,061 snapshots in the final fixture.

After shared compilation settled, the final integrated Cargo suite passed all seven tests in both debug and release. Targeted Clippy passed with `-D warnings` and focused rustfmt passed. The unchanged-source oracle matched all 639 traces. These observations are native Windows with Rust 1.85.1 and Node v24.15.0. Logs are `results/touch-policy/{debug,release,clippy,fmt}.txt` and `oracle-run.txt`. They are scoped checks, not a whole-workspace or browser pass. Native Linux/macOS, DOM field/callback/target lifecycle and real browser scrolling remain separate, still-required checks. Resource coordination paused new Cargo/Node build runners after these commands completed; no additional build is inferred from that pause.
