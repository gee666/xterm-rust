# Link render invalidation range rechecks

Task 525 repairs the browser Linkifier render listener. This is a scoped selection/link fix, not complete task 321 or global equivalence.

The unchanged `source_repo/src/browser/Linkifier.ts` listener installed by `_handleNewLink` first checks the current link against the rendered rows. It then calls `_clearCurrentLink(start, end)`, which checks the range again unless either numeric bound is falsey. Getter mutations can make this second check false. In that case the link stays hovered, but the outer listener still asks providers again. A second-check exception stops the provider request. Each endpoint expression also reads the current link again, so clearing it in the start getter causes a TypeError before the end getter.

Rust previously tested the range only once and cleared unconditionally. `State::current_within_rows` now rereads the current handle for each endpoint. `clear_current_with_rows` repeats the source guard and truthiness rules. Ordinary mouseleave, resize and hover paths use the same clearing implementation without row bounds. No state borrow spans a getter or callback. The native typed link API and existing underline geometry remain unchanged.

## Reproduction and checks

Use Node.js, Rust 1.85.1, the locked wasm32 target and wasm-bindgen 0.2.100. Run from the owned worktree. Point `SOURCE_ROOT` to the read-only original and `PLAYWRIGHT_MODULE` to its installed Playwright module. Use owned O: TEMP/TMP and a private target. Coordinate compiler/browser allocation before these commands.

```sh
node scripts/link-render-recheck-methods.mjs
node scripts/public-link-provider-source.mjs --render-recheck --generate
WASM_TEST=wasm_link_render_recheck node scripts/browser-host-check.mjs
cargo test -p xterm-web --all-targets --all-features -j2
cargo clippy --workspace --all-targets --all-features -j2 -- -D warnings
cargo clippy --workspace --lib --all-features --target wasm32-unknown-unknown -j2 -- -D warnings
cargo clippy -p xterm-web --test wasm_link_render_recheck --all-features --target wasm32-unknown-unknown -j2 -- -D warnings
cargo fmt --all -- --check
```

Generation is only for the original oracle. Rust must compare against the retained fixture, not regenerate it. The unchanged-source methods cover ten cases. Eight are also run against opened original and Rust Terminals using public providers, DOM mouse events and real refreshes. Two additional direct-controller cases clear the current link by dispatching mouseleave from a range getter. The direct controller preserves exact exception values; the public trace checks error values captured from the render callback's window error event. Expected read sequences and leave/requery decisions are asserted before fixture comparison.

`public-link-provider-source.mjs` adds `--render-recheck` and retains all existing modes, including task 500's `--underline-geometry`. Task 514's independent `--mouse-path` mode is pending coordinator integration. Its runner additions must be retained when composing both producers. There are no private 514 prerequisite copies on this branch.

## Evidence status

The unchanged-source method checks passed all ten directed cases. Browser execution, negative control and strict quality checks are not yet recorded. Missing evidence is not a pass. See `results/link-render-recheck/` for exact receipts as they are produced. Native OS/browser/global matrices remain outside this scoped repair.
