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

Generation is only for the original oracle. Rust must compare against the retained fixture, not regenerate it. The unchanged-source methods cover twelve cases. Eight are also run against opened original and Rust Terminals using public providers, DOM mouse events and real refreshes. Four additional direct-controller cases clear the current link by dispatching mouseleave from a range getter. The direct controller preserves exact exception values; the public trace checks error values captured from the render callback's window error event. Expected read sequences and leave/requery decisions are asserted before fixture comparison.

`public-link-provider-source.mjs` adds `--render-recheck` and retains all existing modes, including task 500's `--underline-geometry`. Task 514's independent `--mouse-path` mode is pending coordinator integration. Its runner additions must be retained when composing both producers. There are no private 514 prerequisite copies on this branch.

## Observed results

On main base `11ad71a8`, unchanged source methods passed all twelve cases. The shared opened original Terminal trace passed eight cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Rust passed two functions per engine, twelve direct-controller cases plus the eight public Terminal cases, both initially and after the negative control. The exact three pre-repair production files from `11ad71a8` compiled against the new assertions and failed Chromium with zero passed and two failed at the missing second range reads. All three repaired file hashes were restored in `finally`. Initial and final compiler WASM, served WASM and bindings hashes match. All three negative hashes differ. Compiler paths are reused by Cargo, so the negative compiler hash is a historical receipt, while its distinct served WASM remains inspectable until cleanup.

Eight retained browser targets passed twenty functions per engine. These are public providers, provider reentry, hover-event sampling, live columns, linkifier, disposal, terminal links and task 500 underline geometry. These counts are functions, not assertion totals or complete source-suite coverage.

Native Windows `cargo test --locked -p xterm-web --all-targets --all-features -j2` passed 417 tests across 174 summaries, zero failed, ignored, measured, filtered or warnings. Strict native workspace all-target/all-feature Clippy, WASM workspace all-feature library Clippy and the new WASM test Clippy passed on Rust/Cargo 1.85.1 and Node 24.15.0. All five whole-member formatting checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS error 206. The raw failure is retained, not waived or reported as a workspace-command pass.

The existing source-only oracle also passed all seven unchanged Linkifier/OscLinkProvider unit assertions. Its 4,097 seeded OSC cases, 12,219 reply operations across 2,048 scenarios and 8,192 debounce operations regenerated byte-identical retained fixtures. These source checks are not browser or full-family proof. Raw seeds, hashes and outcomes are retained separately.

The first source-browser run passed Chromium and Firefox but failed WebKit error identity because the adapter accepted an unrelated render event before the expected global error. The adapter now waits for the matching ErrorEvent on error phases, with the same deadline and assertions. Trace arrays are copied before later cleanup can mutate recorded events. The initial failure is retained under `development/`; no production change or assertion weakening fixed it.

`results/link-render-recheck/index.json` records source inputs, raw receipts, temporary browser artifacts and cached native link executables for independent review. Logs retain their terminal blank lines. Source-only whitespace checks pass; raw receipts are not trimmed for Git's whitespace checker. Both original and physical-copy references matched all 776 baseline hashes before testing. Runtime files remain temporary until reviewed and safely removed.

## Scope limits

Full selection/link task 321 and overall equivalence remain NOT PROVEN. Complete original browser/demo suites, sparse/custom provider containers, raw drawing and selection number domains, deeper callback replacement, lifecycle/GC, stable/release workspace quality, native Linux/macOS and final mapping/comparison evidence remain required. Scoped passing tests do not close those gaps.
