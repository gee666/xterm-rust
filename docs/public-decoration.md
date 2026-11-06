# Public decoration registration

Independent review accepted task 537 at `b31bc29f`, with no findings. It implements a milestone of task 331 and is not integrated. Full decoration/marker parity and overall equivalence remain NOT PROVEN.

## Implementation and binding

`Terminal.registerDecoration(options)` uses the live Rust `DecorationService`. It does not load the JavaScript terminal engine. The bridge evaluates JavaScript validation operators and preserves the original options object. Rust owns indexing, cell lookup, cached colors, DOM geometry, rendering events and disposal.

The public method is shared across Terminal instances and requires no proposed-API opt-in. `_registerDecoration` is an adapter entry point, not an additional upstream API. Its generated binding accepts a real exported Marker, including one from another Terminal in the same WASM instance. Arbitrary structural marker objects remain in task 331.

`Decoration::property` reads one live host field without a native options borrow. Native typed decorations still use shared `DecorationOptions`. For host decorations, `options()` is only the native marker/options capability; use `property` for host fields. `register_decoration_source` returns a Decoration directly because its host already performed the disposed-marker guard before constructor getters. Typed `register_decoration` still returns `Option<Decoration>`. A constructor getter may dispose a marker after the source guard, so the host path does not repeat it.

Native cell iteration now yields `Result<Decoration, DecorationError>`, and callback traversal returns `Result`. This Rust representation change propagates accessor errors instead of panicking or discarding them. Existing Rust assertions unwrap successful items, retaining their old expectations and fixture bytes. Row construction retains the original error through `Rc<dyn Error>` instead of converting it to a string.

Height is captured once per cache operation, but addition runs for each loop comparison, preserving object conversion and string concatenation. The existing non-advancing Number-span error remains a documented safety divergence from a source hang.

## Ownership and failure paths

A JavaScript WeakMap associates active records with the actual public Terminal receiver. Native property readers hold only a WeakRef to each record. Event functions retain listener entries in JavaScript, while native callbacks refer to those entries weakly. Native code has no persistent strong edge into options/listener graphs that can refer back to the Terminal or decoration.

Host capabilities and cleanup are installed before registration can invoke cache getters. A getter failure after source insertion therefore retains a usable cleanup path for a later reset. Successful native disposal clears the JavaScript listener table and removes the active record. A throwing disposal listener preserves the record and permits source retry behavior.

This is not complete GC or adversarial reentry proof. Registration-time disposal inside a geometry getter can leave source cache state after disposal. That partial-state lifetime, and getter-driven removal during an active lookup/render, need explicit task-331 coverage before claiming full raw-object lifecycle parity.

## Observed verification

Evidence is indexed in `results/public-decoration/index.json`. All commands ran on native Windows with Node 24.15.0, Rust/Cargo 1.85.1 and wasm-bindgen 0.2.100. Browser versions and raw outputs are retained.

- The unchanged original Terminal runs the shared 14-phase public trace in Chromium, Firefox and WebKit. All three match the retained fixture.
- The actual exported Rust Terminal passes one test function containing that same complete trace and fixture comparison in each engine, both initially and after the negative control.
- All 16 changed production files were replaced with their exact pre-repair `0442e357` versions, with new modules absent. The old engine compiles and fails Chromium with zero passed and one failed at the missing public API. All working bytes were restored in `finally`. Initial and final compiler WASM, served WASM and binding hashes are identical; all negative hashes differ.
- Six retained browser suites pass 18 functions per engine: buffer decorations 8, Terminal decorations 2, markers 4, overview ruler 1, Terminal overview 2 and row factory 1.
- A matching production debug package runs unchanged original `registerDecoration` and `decoration color overrides` suites. There are 33 discovered case-projects, 27 executed/passed, 6 original WebKit renderer skips, zero failures and zero unrun bodies. Chromium and Firefox each pass 11, WebKit passes 5. This is a scoped original-suite run, not full core acceptance.
- The private producer passes native `cargo test --workspace --all-targets --all-features -j2`: 2,221 tests across 516 summaries, no failures, ignores, measurements, filtering or warnings. Two workspace doctests pass.
- Warning-denied native workspace all-target/all-feature Clippy, WASM workspace library Clippy and the actual WASM public-test Clippy pass.
- The seven focused native decoration targets pass 36 tests, including six new host tests, the unchanged 1,024-operation fixture and 405 raw snapshots.
- Whole-member formatting passes separately for all five workspace members. The exact `cargo fmt --all -- --check` command fails with Windows error 206. It is retained as a failed command, not reported as a pass.

No Linux/macOS runtime, complete original-suite matrix, complete feature/release matrix, performance comparison or overall equivalence claim follows from these results. No unsafe code or dependency was added.

## Retained development failures

The first shared-source trace compared a raw fractional Number string with CSSOM's serialized pixel value. A bounded diagnostic observed `16.4762px` versus `16.476190476190474px`. The corrected expectation uses a real CSSStyleDeclaration to serialize the expected position, with exact equality and no tolerance. Source assertions and Rust production behavior were not altered to obtain this source pass.

The first negative-control driver searched only wasm-bindgen-test stdout. The intended missing-API panic was in the retained console entry, while stdout contained the resulting unreachable trap. The driver now checks stdout and console. It resumed from verified source/artifact hashes without replaying the negative run, then rebuilt and checked the restored final engine. Both driver versions and the original failure are retained.

The first original-suite attempt discovered zero tests because the isolated dependency copy lacked `@lunapaint/png-codec`. Seven required packages were copied from the existing install, without a new install or reference junction. All 313 regular copied files were SHA-verified. The fixed setup then passed the unchanged suites. The setup failure is retained separately from application outcomes.

Early schema commands set TEMP to O: but assigned TMP from the old shell variable in the same export statement. Their TMP location was not enforced, and no unknown temporary paths were deleted. The later compiler phase and all browser/final commands set TEMP, TMP and TMPDIR explicitly under the owned O: worktree. Targets and retained outputs remained there throughout.

## Remaining task-331 contracts

- Writable marker arrays, marker fields and arbitrary structural/cross-WASM marker objects.
- Raw writable decoration fields and property-descriptor/prototype changes, including replacement of marker, event or element properties.
- Complete phase-by-phase overview getter behavior. The typed zone store reads live options per redraw but does not repeat getters during every zone comparison.
- Overview positions outside full/left/center/right. The typed store reports an explicit error rather than claiming support for those raw values.
- Arbitrary raw color values, BigInt relational operands and complete GC/reentry behavior.
- Full original browser suites, WebGL/addon interactions, native OS matrices and final integration proof.

## Resources

Browser/server work and the final compiler allocation were released in post 409, ending with comment 4573. No owned jobs remain. Review verified every indexed hash, reran seven cached executables containing 36 tests, checked all 25 changed Rust files' formatting and left no scratch files. See `results/public-decoration/review.json`.

Only `O:/xterm-rust-swarm-16/tmp/worktrees/SarahDecoration` is owned temporary storage. It contains a regular target and physical reference/dependency copies, with no reference junction. Accepted owner cleanup will archive the branch, inspect every path/process, remove only verified generated outputs and use normal Git worktree removal. Post 409 and the private `sarah537-cleanup.json` receipt establish actual completion. This pre-removal document does not claim the root is already absent and grants no build, browser, main-ref or cleanup replay.
