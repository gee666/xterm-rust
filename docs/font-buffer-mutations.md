# Font buffer writes during warning callbacks

Task406 closes the task342 gap where a Mac `kern` warning callback writes through an alias of the local-font ArrayBuffer. It does not complete broader font metadata or terminal equivalence.

## Constructor behavior

The supplied opentype.js copies table-directory descriptors and parses cmap, glyph names and fixed metadata before reading deferred `kern` and GSUB. A Mac `kern` count above one calls the live `console.warn` before coverage and pair reads. The callback can change the buffer retained by its `blob().arrayBuffer()` provider.

`Font::load_buffer_with_refresh` accepts the initial snapshot plus a diagnostic callback returning an optional current whole-buffer snapshot. This keeps the portable core independent of browsers. The existing snapshot and warning-only APIs keep their contracts. Already parsed metadata, kern version/count and directory descriptors remain fixed. Later uncompressed kern reads use refreshed bytes. Already inflated kern bytes stay independent of the external buffer, while later GSUB decompression or parsing uses the refreshed bytes and original descriptor. No font parser runs in JavaScript for Rust; the added JS boundary copies browser storage and catches failures. No dependency or unsafe code was added.

The live addon retains both its initial bytes and original ArrayBuffer until construction completes. It invokes the real warning first, with no addon-state borrow, and snapshots only after successful callback return. A thrown callback value still wins over subsequent parsing errors.

## Evidence

`results/font-buffer-mutations/` contains raw scoped Windows receipts. Source dependency and production-file hashes are in `results/font-addons/font-buffer-mutation-oracle.json`.

- The unchanged source ligature loader generates39 constructor outcomes and39 async addon traces. Inputs cross SFNT, raw WOFF and WOFF with compressed kern. They cover coverage/pair mutations that repair or break bounds, already-read header/directory/cmap/head mutations, deferred GSUB corruption, removing all calt tags, and a throwing mutation callback. FiraCode's real shaping ranges distinguish successful native loading from fallback.
- A native Rust test independently reconstructs all39 inputs and compares warnings, typed errors, callback errors and shaping ranges. The fixture is generated data and intentionally exceeds350 lines. The equivalent JS and Rust builders independently reproduce oracle inputs; this duplication checks cross-language input construction instead of sharing a font parser.
- Two new WASM test functions pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. They check all39 constructor cases and39 live-addon cases per engine. These use the actual Rust font loader/addon with a recorded terminal boundary, not a full interactive Terminal suite.
- The unchanged task398 binary also passes two functions per engine, retaining192 constructor cases and warning receiver/accessor/throw/reentry/family-change traces. Native metadata tests preserve the1471 earlier constructor corpus too.
- The mutation negative control removes only consumption of refreshed bytes. The new native corpus fails at `SFNT IgnoreBadPairs`, where the source's callback repairs coverage before later pair reads. The exact producer file is restored in a finally block.
- Native all-target/all-feature workspace Clippy passes with warnings denied. Workspace tests pass2068 tests across419 summaries, with zero failed or ignored. Focused warning-denied WASM Clippy covers the new and retained warning/metadata binaries. These are Windows scoped receipts, not final union or native Linux/macOS evidence.
- `cargo fmt --all -- --check` hits Windows OS206. A temporary short drive alias did not solve Cargo's canonicalized argument paths; it was removed in finally. Running the same format check separately for each of all five workspace packages passes. The aggregate gate failure is retained, not relabeled as a pass.

The first180-second workspace test invocation timed out after earlier passing suites. No owned descendants survived. Its partial log is retained separately; the cached600-second-budget rerun completed successfully. During source-input development, removing only the first calt record did not remove all shaping groups. The oracle guard rejected that expectation; the input now deliberately changes every calt tag without changing source assertions.

## Reproduce

Use a private short O: checkout, private target and owned `tmp/`. Set `SOURCE_PACKAGE_JSON` and `PLAYWRIGHT_MODULE` to existing read-only source dependencies and `WASM_BINDGEN` to the installed matching0.2.100 CLI. Observe the team's heavy-job cap and Cargo jobs2.

```sh
node scripts/ligature-addon-oracle.mjs --buffer-mutations
cargo test --locked -j2 -p xterm-addons --test font_buffer_mutations --test font_metadata_constructor
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_buffer_mutations_browser WASM_TEST_TIMEOUT_MS=180000 node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_kern_warning_browser node scripts/browser-host-check.mjs
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_buffer_mutations_browser --test font_kern_warning_browser --test font_metadata_browser -- -D warnings
cargo test --locked -j2 --workspace --all-targets --all-features
cargo fmt --all -- --check
```

`--generate` deliberately updates source-generated expectations. It is not used to clear a Rust failure. On Windows OS206, the recorded equivalent package checks use `cargo fmt -p PACKAGE -- --check` for every workspace package; this does not erase the aggregate failure.

## Remaining work

Task342 still owns other metadata and exact broader JS error mappings. This work covers byte writes, not detached or resized ArrayBuffers, SharedArrayBuffer concurrency or patched global typed-array constructors. A detached buffer copy is caught rather than trapping WASM, but its exact source DataView error text is not established here. Future post-warning metadata parsers must use the same refreshed bytes without rereading descriptors. Akiko's meta task407 is coordinating that union explicitly. Full original Terminal/addon/demo acceptance, final stable-union gates and native OS evidence remain open. Overall equivalence is NOT PROVEN.
