# Underline geometry sampling

Task 500 repairs Linkifier underline geometry sampling around reentrant range getters. It does not close selection/link family 321 or establish full equivalence.

## Source contract

`source_repo/src/browser/Linkifier.ts::_fireUnderlineEvent` reads `link.range`, then snapshots `buffer.ydisp`. It evaluates start x minus one, start y minus offset minus one, end x, and end y minus offset minus one. `_createLinkUnderlineEvent` then reads the live columns. Endpoint getters and numeric coercions can scroll or resize the terminal. The offset remains the snapshot taken after the range getter, while columns reflect changes through the final arithmetic.

Rust previously read both geometry values before the range getter. A range getter that scrolls the terminal therefore drew the underline on the wrong viewport row. An endpoint getter that resizes used stale columns. The private payload method now accepts live callbacks. State reads columns from its Cell and offset with a short borrow only when the source would sample it. No borrow spans a JavaScript getter or coercion. Native typed links have no getter reentry and retain their typed geometry calculation. No dependency, unsafe operation or JavaScript terminal-engine delegation is added.

Accepted 489 remains a distinct prerequisite. Its live-column hit tests are retained, along with earlier provider transaction and hover-event fixes. Task 500 changes only underline sampling, not hit tests, provider mapping, selection coordinates or raw numeric domains.

## Verification commands

Use the original read-only reference and existing Playwright and wasm-bindgen installations. Set both TEMP and TMP to an owned temporary directory on O: on this Windows run, use Cargo -j2, and obtain a resource allocation before compiler or browser work.

```sh
SOURCE_ROOT=/absolute/reference/source_repo node scripts/link-underline-geometry-methods.mjs
SOURCE_ROOT=/absolute/reference/source_repo PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs node scripts/public-link-provider-source.mjs --underline-geometry --generate
WASM_TEST=wasm_link_underline_geometry PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs WASM_BINDGEN=/absolute/wasm-bindgen node scripts/browser-host-check.mjs
```

The fixture must come from successful original runs, not edited Rust expectations. The method oracle executes unchanged TypeScript with 22 show/hide cases asserting range, offset, endpoint/coercion and columns read order plus thrown-marker identity. The Rust component test replays the same 11 geometry modes for both show and hide with a real controller and reentrant resize, inspecting emitted coordinates and columns. The shared opened-Terminal trace uses public provider registration, real DOM mouse events, scrolling, resizing and live decoration setters. Its 24 show/hide cases inspect actual DOM underlines and thrown identity.

## Observed results

The unchanged method passed 22 cases. The original opened Terminal passed the shared 24-case trace in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Rust passed two functions per engine, one replaying 22 component emission cases and the other replaying the same 24 public Terminal cases. Both show and hide are covered. The exact pre-repair production files from the explicitly preserved 489 prerequisite compiled against the new assertions and failed Chromium with zero passed and two failed. The public test failed at `range scroll: underline rows`, while the component emitted y coordinates 2 and 3 instead of 1 and 2. The driver restored both final files in `finally` and verified their hashes. Both final functions then passed all three engines.

Seven retained browser suites passed each engine, totaling 18 functions per engine. These are live-column hit tests, hover-event sampling, provider reentry, linkifier, disposal, terminal links and public providers. They are separate from the two new functions and are not the full original selection/link suites.

`cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 400 tests across 166 summaries, zero failed, ignored or filtered. Strict all-target/all-feature native and WASM Clippy passed. All five package-format checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS error 206, with its raw log retained. No full workspace-format-command pass is claimed. Tools were Node 24.15.0, Rust/Cargo 1.85.1 and wasm-bindgen 0.2.100 on native Windows. Both TEMP and TMP pointed to the owned O: temporary directory. All 776 original reference hashes and all 776 physical-copy hashes passed before execution.

The branch carries accepted 489 as an explicit private prerequisite atop main `5088f8fd`. Integration must pick only task 500 commits after the coordinator's 489 integration, not replay the private copies of 489. The pre-repair negative is reproducible with `node scripts/link-underline-negative.mjs archive/Wallada489-complete` under the same allocated browser environment. It compiles the exact old files against current tests rather than editing expected results.

## Evidence limits

Execution results and exact source/build hashes belong in `results/link-underline-geometry/`. Native Windows is the execution platform. Linux/macOS runtime evidence is not claimed. Full original selection/link suites, malformed/provider container behavior, raw drawing and selection number domains, deeper getter-driven current-link replacement, lifecycle/GC, demo acceptance, full workspace quality and comparison/mapping work remain required. Scoped passes do not prove full-family equivalence.
