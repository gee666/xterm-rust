# Raw keypress numeric fields

## Intent and scope

Task378 continues the complete16-agent Rust port. It fixes a concrete task319 boundary rather than claiming full input/accessibility parity. A decorated KeyboardEvent can return a string, boolean, object, Symbol or BigInt from charCode/which/keyCode. Upstream _keyPress first performs truthiness and strict-zero/nullish checks, then converts the selected value at String.fromCharCode. The old DOM adapter rejected non-number values before those guards.

The fix covers charCode, which and the keypress-only keyCode fallback. Numeric keyCode handling in keydown/keyup protocol encoders is unchanged. Full source-family suites, other decorated fields, general lifecycle equivalence and source-engine overrides remain the broader319 obligation. No supplied source or assertion was changed.

## Rust representation

EventNumber<E> retains Undefined, Null and Number without host conversion. Other retains a non-number/non-nullish value's truthiness and a fallible conversion callback. It is never strictly equal to numeric zero. This deliberately distinguishes the truthy string "0", which produces NUL, from numeric zero, which may suppress delivery.

The dispatcher selects the last source-ordered getter result, tests modifiers and only then calls to_number. Errors propagate before onKey, show-cursor and data effects, leaving the source's keypress/dead-key flags intact. Conversion can synchronously dispatch nested input with no model borrow held. DOM values use the existing portable/WASM option-number bridge and recover thrown-value identity through the existing error adapter.

KeyEventSource::key_press_code defaults to the typed numeric keyCode read. The DOM implementation retains a raw fallback value without changing the encoder interface. Its admission path is intentionally separate from strict encoder validation, so fixing keypress cannot silently change keydown/keyup encoding. Static native callers keep their existing representation. EventNumber now has an error parameter, defaulting to Infallible; to_number returns Result, Clone and Debug remain available, and Copy is no longer possible for retained host callbacks. No dependency, unsafe code or production JS terminal logic was added.

## Verification

Results are retained under results/raw-keypress378. Commands use installed Rust1.85.1 Cargo/rustc executables explicitly, CARGO_BUILD_JOBS=2 and TEMP/TMP under the owned O: worktree.

- Source fixture checks compare the complete unchanged _keyPress and _isThirdLevelShift method bodies, including comments. A separate assertion pins the exact upstream Mac/Windows platform selectors. Existing InputEvent/CompositionHelper integrity checks remain intact. All5 source checks pass.
- Native keypress_raw_values6 plus existing keyboard_dispatch9, getter2, callback3 and input_event_getters6 pass26 functions in total. New assertions cover truthy raw zero conversion, modifier suppression without conversion, changed second reads, raw fallback keyCode, thrown conversion state and nested input ordering.
- wasm_keypress_raw executes the actual Rust Terminal DOM listener. Two test functions expand320 raw-kind/route/modifier combinations plus24 getter/conversion/callback error and nested-input cases per engine. Follow-up real InputEvents observe whether keypress flags were committed. OnKey preserves the original event object, text remains UTF-16, and thrown object/undefined values retain identity.
- Restoring all3 old production files from ff0bec8 causes both new functions to fail in Chromium. This negative control stops at its first counterexample per function; it is not344 executed failing baseline cases. The patched files are restored in finally before the passing rerun.
- Patched Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 pass keypress2, keyboard15, InputEvent4, raw InputEvent2 and full library26 functions per engine. This is49 functions per engine, not49 upstream suites. Existing raw InputEvent90 and InputEvent guard288 case expansions are preserved.
- Native all-target and focused WASM library/new-test warning-denied Clippy pass. Exact workspace rustfmt check passes. These are scoped checks, not the final all-feature/release/native-platform matrix.

After the test-only platform selectors were pinned to the exact upstream constants, the complete344-case three-browser matrix passed again with Rust source unchanged. The original negative-control and earlier full-suite receipts are preserved separately. Canonical main integration succeeded at aa3f79211e48175f317261592ff57bd20e4dc79c. Overall equivalence remains NOT PROVEN.

## Canonical integration

The accepted378/379/380/383/384/385 union was frozen on d0f8a34 and merged through the unchanged canonical helper. It passed1894 native core/web tests across367 summaries with zero failures or ignored tests, changed-crate builds, affected all-target checks, the default workspace check and applicable WASM library checks. Fullfmt and source checks passed before freeze; all9 own and10 core-view implementation hashes stayed unchanged.

This default-feature gate does not rerun every addon test or prove the final platform/feature/release matrix. Task384's successful six-native-job CI/Miri/WASM evidence remains explicitly tied to its older ff0bec8 snapshot. No historical receipt was relabeled as latest-source proof. Raw archived CI log whitespace was retained verbatim; the source-only diff check passed.

## Owned temporary work

Only O:/xterm-rust-swarm-16/tmp/KP is owned for378. It was created with the existing worktree helper, then safely moved to a short path for Windows fullfmt limits. The reference is a physical copy, never a junction. Runners close their browsers/servers in finally. Preserve committed source, failed cases and concise receipts; check links, tracked/untracked status and owned processes before deleting this tree. Task356's older O:A tree was already removed.
