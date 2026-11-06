# HTML raw row subtraction error phase

Task556 repairs HTML explicit-range subtraction errors within still-open346 and322. Overall equivalence is NOT PROVEN.

The original BaseSerializeHandler evaluates endRow - startRow before calling HTMLSerializeHandler._beforeSerialize. HTML does not use the difference, but JavaScript still converts end first, then start, with the number hint. A Symbol, mixed BigInt/Number or nonprimitive conversion result can throw. User conversions can throw their original value, replace the next conversion, reset/resize or invoke a nested serializer. Those effects precede the HTML header and first cell-column conversion. Coordinate property reads still occur before subtraction.

The Rust browser adapter previously skipped this operation and converted the column or start coordinate first. It now reuses the caught host subtraction helper from accepted549. Numeric primitives need no host call because their subtraction cannot invoke a callback or throw. Serialization itself remains in Rust. No dependency or unsafe code is introduced.

The shared scripts/serialize-html-difference-cases.mjs fixture adds three groups with20 subcases to the existing93-case union. It covers primitive conversion getters/calls/valueOf/toString, actual engine errors, thrown identity, untouched column/header callbacks, reset/resize, nested serialization and replacement of the next operand. The runner evaluates exactly the same function against original and Rust terminals, keeping the original assertions intact.

Successful arbitrary-object/BigInt traversal still requires source comparisons, raw ring lookup, endpoint identity, increments and later errors at their original phases. This repair deliberately does not claim that converting raw coordinates once reproduces those operations. Full host replacement, palette identity, GC, family coverage, platform quality, comparisons and complete-suite proof remain open in their existing tasks. Equal infinite row bounds can hang the source and are not run.

## Observed verification

Windows Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 were used. The actual old HTML control removes only the six-line operation, builds a matching debug WASM package, and reports93/96 in EACH Chromium148, Firefox150 and WebKit26.4. Exactly the three new groups fail, while the prior93 cases remain matching. The driver restores the exact fixed working bytes in finally. Original reference and physical-copy manifests match all776 entries before and after source execution and browser runs.

The fixed execution matches96/96 in Chromium and WebKit, but95/96 in Firefox. All20 new phase/type/identity/reentry subcases agree except two Firefox error-message strings for Symbol.toPrimitive returning an object. The helper reports endRow/startRow, while the warmed original reports the object's source literal. A separate Firefox probe observes the original itself reporting endRow when cold and the object literal after the unchanged corpus, for the same input and operation. Its report is retained, not used to normalize results or warm up production glue. These engine/JIT-dependent diagnostic differences remain an explicit task346 gap. The raw runner still exits1 for Firefox; no all-browser green claim is made.

Both complete unchanged original suites pass120/120 source and120/120 Rust cases across the three engines, zero failed, skipped or unrun and no retries. Native xterm-web library all-feature tests pass56, zero failed/ignored/filtered. Native all-feature and WASM library Clippy pass with -D warnings, and the actual debug WASM distribution builds. The exact aggregate cargo fmt --all -- --check fails with Windows OS206; all five whole-package formatting checks pass. This is not a waiver or a full workspace/platform/release claim. Full304 quality remains open.

The first source-only summary harness failed after all20 source operations because Array.prototype.values was mistaken for an object's values array. Its failure is retained. The summary was corrected using Array.isArray without changing the fixture, original source, production Rust or assertions. The subsequent source-only run passes20 subcases. It is additional Node evidence, not browser/Rust proof.

## Reproduction and evidence

With owned O: TEMP/TMP/TMPDIR and Cargo target, Cargo -j2 and a current team allocation:

```sh
cargo test --locked -j2 -p xterm-web --lib --all-features
cargo clippy --locked -j2 -p xterm-web --lib --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
CARGO_BUILD_JOBS=2 node scripts/build-demo.mjs --debug
SOURCE_ROOT=/path/to/reference node scripts/serialize-browser-check.mjs --suite
```

The last command is expected to retain Firefox's diagnostic mismatch, not silently return success. For the negative control, remove only the six-line HTML subtraction operation, rebuild and run the same96-case fixture. Restore exact fixed source bytes and rebuild before judging the repair. Never reuse WASM built from a different source identity. Preserve engine messages verbatim.

results/serialize-html-difference/before and initial-fixed retain raw reports, all source/Rust traces, screenshots, build receipts and compiler logs. initial-fixed also retains both unchanged-suite reports. firefox-diagnostics.json proves the original's cold/warm diagnostic variation. source-node.json records the separate source-only run and source-input hashes. formatting.json records all six actual formatting commands. summary.json recounts the retained raw evidence without rewriting it. producer-pins.json and runtime-pins.json identify owned source/evidence and the actual executable, compiler WASM, served package including snippets, CSS and build pointer. Runtime paths become historical after cleanup.

Canonical main is not modified here. Accepted549 was privately copied only as a prerequisite, with coordinator approval. Copied6158c92a/7f027842/c284bad6/d5bb5892 correspond to original b1b0a742/acb63750/b8f448b0/afa135f8 and must be excluded from556 pickup. Own source is3772d40f plus later evidence/receipts. No dependency, unsafe, source assertion, AGENTS, credential or unrelated producer edit was made. All compiler/browser/server work was released in post409 comment4910. This file does not confirm final worktree removal.
