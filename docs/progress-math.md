# Progress public setter host operations

Task551 repairs the browser ProgressAddon public `progress` setter. Full utility/status task323 and full-port equivalence remain open.

## Source contract

The unchanged `source_repo/addons/addon-progress/src/ProgressAddon.ts` setter validates the first two `state` reads, assigns the third raw state, then evaluates `Math.min(Math.max(progress.value, 0), 100)`. JavaScript resolves the outer `Math.min` property and receiver before the inner `Math.max` and before the value getter. Replaced getters and methods can reenter the setter, replace the global Math object, dispose the addon, return arbitrary values, or throw an arbitrary value. The stored progress value changes only after that entire expression returns. Getter and emitted payload expose the raw return, even when it is not numeric.

The previous Rust browser binding unary-plus coerced `progress.value` and used the portable addon's fixed numeric clamp. It skipped both observable host methods and could not retain an arbitrary host result.

## Implementation boundary

The browser binding now uses one static wasm-bindgen ABI helper for that expression. This helper performs host property access, calls and ECMAScript coercion only. It does not implement terminal parsing, state, registrations or event dispatch and does not import the JavaScript terminal engine. Rust retains validation, partial assignment, binding state, subscription ordering and disposal. No dependency or unsafe code was added.

A `JsValue` mirror retains the public assignment's actual value, like the existing raw-state mirror. The portable native addon remains numeric. Successful protocol events clear the public mirrors. The internal event listener consumes the API-origin flag before notifying public listeners, so a nested protocol event cannot inherit the outer setter's API origin. The binding holds no RefCell borrow across host methods or event listeners.

## Scope still open

This task does not close browser OSC-path use of replaced Math methods or arbitrary raw previous-value reuse. For example, assign an object returned by replaced Math.min, restore the normal Math object, then report OSC9 indeterminate. The source can coerce that saved object when clamping the repeated value. The current numeric protocol backend cannot preserve that object's coercion. Full323 retains this existing protocol-boundary gap as well as the complete six-addon original/demo acceptance. A public-setter regression pass must not be reported as full Progress, utility-family or port equivalence.

## Shared assertions

`crates/xterm-web/tests/fixtures/progress-math.js` contains 30 cases shared by the unchanged-source oracle and the real Rust/WASM test. They cover numeric defaults, lookup/call ordering and receivers, Math replacement during an outer getter, throws at each expression phase, partial assignment, exact arbitrary return identity in getters/events, payload mutation isolation, setter reentry at each phase, noncallable methods, failure after a raw previous value, disposal during Math and invalid-state short circuit.

The source oracle bundles unchanged ProgressAddon and its original Emitter. Its small registration fixture is for original source execution only. The browser test constructs a real Rust Terminal, opens it in a DOM parent, loads the Rust addon and invokes that shared fixture through exported bindings. Every case restores Math before cleanup. Browser/server lifecycle remains owned by the existing browser-host-check runner.

## Commands and evidence

Run commands from the repository root. For the source-only oracle, esbuild comes from the supplied reference's installed dependencies. `REFERENCE_MODULES` permits a separate read-only dependency directory when a physical reference copy intentionally excludes node_modules.

```sh
node scripts/progress-math-oracle.mjs
WASM_TEST=wasm_progress_math node scripts/browser-host-check.mjs
```

Use the project's documented Windows environment syntax where shell assignment syntax differs. `WASM_BINDGEN` must identify CLI0.2.100, and `PLAYWRIGHT_MODULE` can identify the existing reference Playwright installation. All new temporary output belongs under the owned O: worktree.

## Observed results

Production source is commit `c7007809` on main base `224d1b339a43eddf899c2944d9dad734d8c44234`. Tests ran on native Windows with Rust/Cargo1.85.1 and Node24.15.0. The browser runner records each actual engine version, compiler command and artifact hash. Resource grants, starts and releases were explicit in the swarm's resource thread. No own compiler, browser or server remains running.

| Check | Observed result |
| --- | --- |
| Shared source public-setter assertions | 30 cases pass on original and physical copy, all776 reference hashes before/after each |
| Focused native Progress and live status | 12+12 tests pass |
| Native web library | 56 tests pass |
| Warning-denied native web library Clippy | pass, all features |
| Warning-denied WASM library and four test targets | pass, all features |
| Actual WASM no-run build for those four targets | pass, all features |
| New WASM shared fixture, initial and final | one Rust test expands30 cases, pass in each of Chromium/Firefox/WebKit on both runs |
| Exact main two-production-file control | Chromium0 passed/1 failed at the expected observable evaluation-order assertion |
| Retained WASM status suites | 7 coercion+19 status+1 raw-clear test functions pass in each of all3 engines |
| Changed Rust files and all5 package formatting checks | pass |
| Exact aggregate workspace formatting | fails Windows OS206, not waived or reported green |

All passing native/browser runs have zero failed, ignored or filtered tests. All browser phases have zero page errors and zero console errors. New positive runs and the retained coercion suite emit only the expected invalid-state warning, `progress state out of bounds, not applied`. The control fixture and its assertions are identical to the positive fixture. The old files came from the pinned main base, not a hand-written mutant. Both fixed file hashes were restored in finally. Initial and final served WASM hash is `1b705ca8eb7ac09bddf1d0b1af6d690be137f8e7b26e3f950913bcad9a43a20d`; the old control differs. Actual compiler artifacts and generated runtime files remain available for independent review before owned temporary cleanup. Their paths become historical after cleanup.

`results/progress-math/compiler.json` retains native/WASM commands and resource observations. `browser-phases.json` pins phase source, fixture, test, compiler and served WASM identity. Each phase subdirectory contains the raw browser reports, logs and commands. Source-original and source-physical-copy reports retain source assertions, counts and full-manifest integrity results. The new shared source assertions were not substituted for unchanged upstream suites. Required full-family original suites/demo matrices, native Linux/macOS, stable workspace quality, final mapping and comparisons remain open.
