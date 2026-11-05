# Raw scrolling clock samples

Task443 adds `SmoothOperation::try_start_with_clock`, `try_tick_with_clock` and `Scrollable::try_new_with_clock`. These public entry points accept raw fallible clock samples using the existing `ScrollValue` operators. The numeric constructors and tick methods retain their signatures and use the same implementation. There is no second scroll model or JavaScript animation algorithm.

Start first adds10 to the duration. It then calls the clock and subtracts10 from the returned value using the source operator. A clock object is not converted before that subtraction. Clock errors and coercion errors keep their original value. A tick selects its operation receiver before invoking the clock, then reads that operation's start time before subtraction. It reads duration only after subtraction returns. Clock callbacks and result coercion can reenter without holding model or animation borrows.

The browser test supplies independent raw clock values and callbacks to source and Rust. The source still invokes the original `Date.now()` call sites. The Rust callback boundary preserves its raw result/error. The126 new programs cover primitive strings, null, undefined, booleans, nonfinite numbers, bigint/symbol errors, object coercion, throwing tokens, and callback/result-coercion reentry. Reactions replace immediate/smooth positions, dimensions, duration or the clock. Start, replacement, reuse and tick paths compare values, events, pending frames, cancellation, call/coercion order and thrown identity. Reuse must not call the clock. A separate duration-order program checks addition before clock invocation.

Three native tests cover the new standalone numeric entry points, selected from-state before clock reentry and retained tick receiver after the clock cancels the current animation. The existing76 focused native tests remain unchanged. All79 focused Windows native tests and focused native/WASM strict Clippy passed. Five member-specific formatting checks passed. Exact `cargo fmt --all -- --check` still fails before execution with Windows OS206. These are scoped receipts, not workspace or Linux/macOS results.

## Browser status

The126 new and6409 retained programs are ready for execution. Browser execution results will be recorded under `results/raw-scroll-clock`; no browser pass is claimed yet.

## Commands

Use an owned O: worktree, TEMP/TMP/target there, Cargo-j2, existing esbuild/playwright and wasm-bindgen0.2.100. Obtain compiler/browser grants from the current resource owner. Then run:

```text
node scripts/raw-scroll-state-source.mjs --live
cargo test -p xterm-web --all-features --lib --test scrollable --test viewport_controller --test scroll_clock -j2
cargo clippy -p xterm-web --all-features --lib --test scrollable --test viewport_controller --test scroll_clock -j2 -- -D warnings
cargo clippy -p xterm-web --all-features --target wasm32-unknown-unknown --lib --test raw_scrollable_browser --test raw_scroll_animation_browser --test raw_scroll_state_browser -j2 -- -D warnings
WASM_TEST=raw_scrollable_browser node scripts/browser-host-check.mjs
```

## Remaining scope

BrowserScrollable, Element, scrollbar geometry and Viewport still use numeric state and the numeric clock binding. Task301 retains their raw binding migration, mutable returned state-object identity, scheduler reentry and raw event-exception behavior. The test scheduler does not prove a browser scheduler contract. Shared raw clock support does not finish301 or establish full equivalence. Full equivalence is NOT PROVEN.

The generated oracle exceeds350 lines because it bundles unchanged upstream Scrollable/Event/Lifecycle plus the source-derived test corpus. Production does not import it. Original scrollable.ts SHA-256 remains010ddcb39e40b268e4f37afc1da968ae30b64eaf356c918bb08484558150eba4.
