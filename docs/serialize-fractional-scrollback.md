# Fractional numeric HTML scrollback

Task487 repairs finite primitive fractional `scrollback` passed to `serializeAsHTML`. Full raw serialization346, family322 and overall equivalence remain NOT PROVEN.

## Contract and repair

The original `_serializeBufferAsHTML` captures active buffer length, reads the scrollback option, then adds current terminal rows and clamps the result to zero through captured length. A fractional start row survives into `BaseSerializeHandler.serialize`. The loop increments that number without integer conversion. Missing numeric rows still emit empty row fragments. CircularList adds its current start index before remainder, so floating-point rounding can alias a numeric key to real storage. Header callbacks run before traversal and can resize/reset or throw exact JavaScript values.

Rust previously passed this HTML path through ANSI's integral scrollback helper. Fractional unclamped starts raised an early RangeError. `crates/xterm-web/src/serialize_addon/html.rs` now sends finite fractional primitive inputs through the existing `HtmlNumericRange` and shared Rust HTML traversal. It keeps the existing captured length, option/rows read order, column capture and header stages. Integral, undefined, nonfinite and nonnumeric paths are unchanged, including the accepted481 NaN guard now inherited from main. No native handler, ANSI serializer, reference assertion or JS engine delegation is added.

Four shared source/Rust probes cover positive/negative clamp boundaries; captured length versus getter resize; header reset/resize and lone UTF-16 surrogate; includeGlobalBackground/fontFamily/fontSize exception identity; rotated-ring traversal and warning reset/throw. The clamp probe also retains ANSI output/errors for the same inputs.

## Producer history and integration

Initial inspection and three owned commits used main26c7c42d, which already contained467's oversized-column implementation and two probes. Before compiling, main advanced through accepted batch488 to735e114a, including481. I preserved the old tip on archive/Srikandi487-before-refresh, rebased only the three own commits, and retained the integrated NaN guard and both existing probe sets. No accepted producer was replayed. Current own chain is f3f264f7 probes,1171cb8a production repair,8606bdc8 rounding boundaries, based on735e114a. The corpus now has78 cases,74 inherited plus4 new. Baseline source is exactly735e114a HTML, not the earlier26c7 source. The repair adds13 lines after481's existing NaN guard; later integration must preserve this union.

The bounded build/browser helper reuses task467's source/package swapping approach, with two corrections from review and audit. Build failures now restore a consistent entry source/package/pointer or verified unbuilt absence. Source swaps use writeFile, not copyFile: Windows copyFile preserved the older source mtime and Cargo incorrectly reused the baseline WASM for the first fixed build. I caught the identical package hashes before running any browser tests, invalidated that build and retained the exact diagnostic. Revised builds must show fresh=false for both xterm-web variants and distinct WASM hashes. Successful command status plus a source-hash report alone did not prove the compiled variant.

All snapshots, copied reference, targets, generated packages and runtime scratch live under the owned O: worktree. Source, package and build pointer are restored in finally after the negative control. Tests use the unchanged reference and existing serializer runner. Build and browser execution remain subject to the current309 resource grant.

## Evidence

Receipts are under `results/serialize-fractional-scrollback/`. `summary.json` pins66 evidence artifacts,8 source files and16 native executables. Build reports cover1,750 inputs. Corrected builds show xterm-web fresh=false in both Cargo logs; WASM hashes differ. Baseline input hash is523e620267926f788aa824e7a9dc6dbf2fa085ca4a8675ebe582c01cd6684b26; fixed is9f7386434a0dfa2af780b3d28a64d39e136a555ec92d8dc53d09b074c03cb49b.

- Baseline matches74/78 cases per Chromium148, Firefox150.0.2 and WebKit26.4. Exactly the four new probes differ.
- Fixed matches77/78 whole cases per engine. Every new HTML output, getter log, header exception identity and complete terminal snapshot matches. Rotated-ring warning reset produces6 warnings; warning throw produces1 warning and retains the exact sentinel in each engine.
- The remaining clamp-case mismatch is entirely ANSI serialization error text, unchanged between baseline and fixed. Source uses each browser's Array-constructor RangeError; Rust uses native coordinate or fixed-text errors. All12 HTML clamp outputs and getter logs match. There are4 ANSI result differences in Chromium and6 each in Firefox/WebKit. Those raw observations remain in the corpus and summary. The original runner still exits1, without filtering or normalizing assertions. Full ANSI scrollback repair remains346.
- The browser orchestration helper also exits1 because its final assertion expected a wholly green corpus. Its raw failure is retained, not presented as a pass. The separate audit verifies exact HTML fields and unchanged ANSI differences from those retained files. It does not rerun, alter or remove the original runner's assertions.
- Both baseline and fixed execute unchanged original source120 and Rust120 expanded suite cases, zero failed, skipped or unrun. These are40 per browser per implementation, not120 per browser.
- All776 original reference hashes match before and after both runs. The physical worktree reference copy also matches all776.
- All16 native serialization executables pass69 tests, zero failed, ignored or filtered. Warning-denied core library plus all16 serialization tests and corrected actual WASM-library Clippy pass. Web formatting, JavaScript syntax and whitespace checks pass. Exact workspace formatting fails WindowsOS206 before formatting and is not a passing gate.
- Native Windows Rust1.85.1, Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 were used. Full workspace all-feature/release/native Linux/macOS and complete browser/demo matrices remain unrun for this task.

## Reproduction

Use a fresh owned O: worktree and current team resource allocation. Set TEMP first to its tmp/runtime, then set TMP and TMPDIR from that value in a separate export. Keep CARGO_BUILD_JOBS=2 and private CARGO_TARGET_DIR on O:. `commands.json` records exact paths, versions, grants and commands. The preserved proof and audit programs are task-specific receipts, not new build infrastructure.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_html --test serialize_html_numeric_rows
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt -p xterm-web -- --check
cargo fmt --all -- --check
```

The browser command currently returns1 for the retained ANSI-only clamp mismatch; do not report it as green. The two native targets shown are a small reproduction subset, not the69-test total; quality.json records all16. Use735e114a HTML source for the negative control and1171cb8a for the repair, with the final rounding-boundary probes from8606bdc8. If swapping variants on Windows, rewrite the source contents with writeFile so Cargo cannot reuse an older-mtime variant. The first invalid fixed package and the deliberate source-only helper-failure test remain explicitly labeled in their diagnostic directories.

Only owned temporary root is O:/xterm-rust-swarm-16/tmp/worktrees/SrikandiHtmlFraction. All compiler/browser jobs and grants were released on409/3562 before review. Default generated reports and native TSV are output changes, not producer source; they will be restored after review. Review and cleanup receipts follow separately. Overall equivalence remains NOT PROVEN.
