# Mac kern constructor warning callbacks

Task398 implements the Mac `kern` multiple-subtable warning omitted by task391.
This does not complete task342 or prove overall xterm.js equivalence.

## Contract

The unchanged supplied opentype.js reads the Mac version and 32-bit subtable
count, then calls `console.warn("Only the first kern subtable is supported.")`
when the count exceeds one. The call precedes coverage, pair-count and pair-data
reads. Therefore even a truncated font can warn before failing, and a thrown
warning value must replace any later bounds failure.

Rust's portable `Font::load_buffer_with_warnings` accepts a synchronous
`FnMut(FontWarning) -> Result<(), E>`. `FontLoadError<E>` separates font-data
failures from the original callback failure. The existing `load_buffer` remains
a snapshot API without a diagnostic sink, rather than depending on a console or
operating system. Hosts needing observable diagnostics use the new method.

The browser addon calls the real dynamic `console.warn` through a small caught
JavaScript boundary. It preserves the console receiver, accessor exceptions,
ignored return values and exact thrown values. There is no addon-state borrow
across this callback. Same-family reentry sees fallback while loading; a
font-family change starts another load and prevents an older completion from
refreshing the wrong family. Existing Promise success/failure handling retains
the thrown value and failure cache behavior.

No font parser runs in JavaScript for Rust. The only JS font parser in this work
is the unchanged source oracle. No dependency or unsafe code was added.

## Verification

Windows Rust1.85.1, Node24.15.0 and matching wasm-bindgen0.2.100 were used. Raw
receipts are under `results/font-kern-warning/`.

- The source constructor oracle covers192 cases. It crosses SFNT and compressed
  WOFF, counts0/1/2/4294967295, twelve truncation lengths and throwing/nonthrowing
  callbacks. Shared native and WASM assertions compare warnings, acceptance,
  typed bounds errors and the original callback error identity.
- Two new native test functions also exercise recursive independent font loading
  and earlier missing-name/cmap failures suppressing the warning.
- Ten shared async source/Rust addon scenarios cover successful and truncated
  fonts, method/accessor throws, number/symbol/null/undefined thrown values,
  same-family reentry and family changes. A second rejection exposes the saved
  previous error through the source debug callback, checking identity.
- The new WASM binary passes2 test functions in each of Chromium148.0.7778.96,
  Firefox150.0.2 and WebKit26.4. These are actual Rust font-reader/addon tests
  with a recorded terminal boundary host, not full interactive Terminal tests.
- The unchanged retained task391 binary independently passes2 functions in each
  engine, including its1471 constructor cases and async rejection scenarios.
  Its Firefox budget remains the documented180000ms, not an unreported default.
- Full workspace formatting, native all-target/all-feature warning-denied Clippy
  and2064 native tests across414 summaries pass. Focused warning-denied WASM
  test Clippy passes. These are scoped Windows receipts, not native Linux/macOS
  or a final stable-union quality result.
- The mutation negative control suppresses only the new warning call, then
  restores the exact file in a finally block. The corpus fails on Mac count2
  size8 with no warning where the source warns before a later bounds error.

Two development failures remain recorded. `browser-first.json` combined the old
and new async scenarios in one binary; the realm-wide catalog from the new test
made the old test query the wrong host. Its original assertions were not changed.
The new scenarios now have their own binary. `browser-missing-snippet.json`
records the next build's missing nested JS import. The WASM test now imports and
exercises the shared font builder explicitly, so wasm-bindgen copies that module.
Neither failure is relabeled as a passing baseline.

## Reproduce

Use an owned short O: checkout and private target. Set TEMP/TMP/TMPDIR to its
`tmp/` directory. Reuse read-only reference dependencies through
`SOURCE_PACKAGE_JSON` and `PLAYWRIGHT_MODULE`, and set `WASM_BINDGEN` to the
matching installed CLI. Keep Cargo jobs2 and observe the team resource cap.

```sh
node scripts/font-kern-warning-oracle.mjs
node scripts/ligature-addon-oracle.mjs --kern-warnings
node scripts/ligature-addon-oracle.mjs --metadata
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_kern_warning_browser node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser WASM_TEST_TIMEOUT_MS=180000 node scripts/browser-host-check.mjs
cargo fmt --all -- --check
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_kern_warning_browser --test font_metadata_browser -- -D warnings
cargo test --locked -j2 --workspace --all-targets --all-features
```

Only `--generate` deliberately rewrites source-generated expectations. Inspect
differences rather than regenerating fixtures to clear a failure.

## Remaining gaps

Task342 still owns complex metadata, exact broader font error mappings and
malformed outline behavior. Browser local-font bytes are copied into an immutable
Rust snapshot. A console callback that mutates an externally retained source
ArrayBuffer during parsing is not equivalent to the source's live DataView;
that existing aliasing gap is not repaired or claimed covered here. Full original
Terminal/addon/demo suites and native Linux/macOS execution remain unproven.
