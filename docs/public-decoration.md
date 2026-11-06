# Public decoration registration

Task 537 is a milestone of task 331, not full decoration or marker equivalence. Overall equivalence remains NOT PROVEN. The implementation and shared browser assertions are staged; browser execution, original-suite execution, independent review and integration are still pending.

## Implementation

`Terminal.registerDecoration(options)` calls the live Rust `DecorationService`. It does not load the JavaScript terminal engine. The bridge evaluates JavaScript validation operators and preserves the original options object. Rust owns indexing, cell lookup, cached colors, DOM geometry, rendering events and disposal.

The public method is shared across Terminal instances and requires no proposed-API opt-in. `_registerDecoration` is an adapter entry point, not an additional upstream API. Its generated binding accepts a real exported `Marker`, including one from another Terminal in the same WASM instance. Arbitrary structural marker objects remain in task 331.

`Decoration::property` reads one live host field without a native options borrow. Native typed decorations still use shared `DecorationOptions`. For host decorations, `options()` is only the native marker/options capability; use `property` to read host fields. `register_decoration_source` assumes its host already performed the disposed-marker guard before constructor getters. It does not repeat that guard after a getter may have disposed the marker.

Native cell iteration now yields `Result<Decoration, DecorationError>`, and callback traversal returns `Result`. This is a deliberate Rust representation change so accessor errors propagate instead of panicking or disappearing. Existing Rust assertions unwrap successful items, retaining their old expectations and fixture bytes. Row construction retains the error object through an `Rc<dyn Error>` rather than converting it to a string.

Height is captured once per cache operation, but addition is evaluated for each loop comparison, matching JavaScript object conversion. String height retains concatenation. The existing non-advancing Number-span error remains a documented safety divergence from a source hang.

## Ownership

A JavaScript WeakMap associates active records with the actual public Terminal receiver. Native field readers hold only a WeakRef to each record. Event functions keep listener entries in the JavaScript record, while native callbacks refer to entries weakly. This avoids a native strong edge into options/listener graphs that may refer back to the Terminal or decoration.

Successful native disposal clears the JavaScript listener table and removes the active record. A throwing disposal listener preserves the record and permits the source retry behavior. This design has not yet received GC or adversarial reentry proof. In particular, registration-time disposal inside a geometry getter can leave source cache state after disposal. That partial-state lifetime needs explicit task-331 coverage before claiming full raw-object lifecycle parity.

## Remaining task-331 contracts

- Writable marker arrays, marker fields and arbitrary structural markers.
- Raw writable decoration fields and property-descriptor/prototype changes, including replacement of the options, marker, event or element properties.
- Complete phase-by-phase overview getter behavior. The current typed zone store reads live options for each redraw but does not reproduce repeated getters during every zone comparison.
- Overview positions outside the declared full/left/center/right values. The current typed store reports an explicit error rather than pretending to implement those values.
- Arbitrary raw color values, BigInt relational operands, cross-WASM marker objects and complete GC/reentry behavior.
- Full unchanged browser suites, WebGL/addon interactions, native OS matrix and final integration proof.

## Verification so far

The original six native decoration targets pass 30 tests, including the unchanged 1024-operation fixture and 405 raw snapshots. Five new host-property tests pass. Native core/web all-target/all-feature Clippy and actual WASM public-test Clippy pass with warnings denied. The actual new WASM target compiles. No browser pass is claimed yet.

`cargo fmt -p xterm-core` and `cargo fmt -p xterm-web` succeeded. A combined two-package format command hit Windows error 206. This is not a workspace-format pass.

Early schema commands set TEMP to O: but assigned TMP from the old shell variable in the same export statement. Their TMP location was not enforced. No unknown temporary paths were deleted. The later compiler phase records TEMP, TMP and TMPDIR explicitly under the owned O: worktree in `tmp/compile4413-env.json`. Targets and retained outputs are in that worktree. Preserve this distinction in final evidence.

The shared trace in `scripts/public-decoration-trace.js` exercises the real public API, option and marker identity, validation phases and thrown identity, foreign native markers, DOM geometry, cached cell colors, overview canvas pixels, event collection, disposal errors/retry, reset and disposal. `scripts/public-decoration-source.mjs` runs it on the unchanged original Terminal and checks source-map identities. `wasm_public_decoration` runs it on actual exported Rust Terminal instances. Neither is a full-family coverage claim.
