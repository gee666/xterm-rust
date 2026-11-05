# Deferred kern constructor validation

Task391 fixes structural `kern` constructor rejection in the Rust ligature font
reader. It does not complete font metadata task342 or establish full port
equivalence. Overall equivalence remains NOT PROVEN.

## Behavior

The supplied opentype.js constructor selects the final `kern` directory entry,
without reading earlier entries. It reads that entry after outlines, horizontal
metrics and cmap glyph-name assignment, but before GSUB. Rust now preserves this
selection and ordering, including compressed WOFF preparation. The same private
`DeferredTable` type delays GSUB extraction until its actual constructor phase.
A malformed GSUB descriptor can no longer replace an earlier kern error.

`font_kern.rs` validates the exact Windows and Mac bytes read by the source. It
checks the top-level version, Windows subtable version, Mac coverage byte and
first-subtable pair count/data. Source skip operations do not read bytes. Zero
pairs therefore do not require the skipped search fields to exist. Mac coverage
formats other than zero return without reading pair data. The source does not
validate Windows coverage, declared subtable length, table count or later
subtables, and Rust does not invent stricter rejection rules for them.

Uncompressed tables retain access to the remaining original DataView bytes, not
just their directory's nominal length. Compressed tables use the decompressed
buffer. Tests cover nominal lengths of zero for both kern and deferred GSUB.
The source FontImpl uses GSUB for terminal ligatures, not kerning pairs. Rust
therefore validates kern construction without allocating an unused pair map.
No dependency, unsafe code or JavaScript font-parser delegation was added.

Native failures remain typed `FontError` values. `KernVersion(u16)` retains the
source version message. All new kern cases compare the expected failure cause,
including cmap, missing-name/hmtx and deferred-GSUB precedence, rather than just
comparing success versus failure. This is not a claim that every native bounds
or missing-table error has the source JavaScript error class/message.

## Evidence

The unchanged installed opentype.js source SHA-256 is
`737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`.
The oracle reconstructs font inputs independently from the shared Rust fixture
builder. The original950 cases and assertions are unchanged. It adds520 kern
cases and one GSUB directory-length case, for1471 cases total,522 accepted and949
rejected. Cases include SFNT/WOFF, truncations, supported and unsupported versions,
pair counts, duplicate descriptors, invalid ignored compressed payloads and
failure ordering. All776 reference-manifest files were verified unchanged.

`results/font-kern/` retains the following scoped Windows evidence:

- `baseline-corrected.log` records327 actual pre-fix constructor mismatches,
  comprising326 kern inputs and one GSUB nominal-length input.
- `native-final.log` records two passing metadata test functions, including all
  1471 case comparisons and the new typed kern error comparisons.
- `ligatures.log` records seven passing existing ligature test functions.
- Full workspace formatting and warning-denied all-target/all-feature native
  Clippy pass. `test-workspace.log` records2062 passing native test functions
  across412 summaries, with zero failures or ignored tests.
- Focused font-addon WASM test Clippy passes with warnings denied.
- Chromium148.0.7778.96 and WebKit26.4 each pass two Rust/WASM test functions in
  `browser-first.json`. Firefox150.0.2 passes those same functions in
  `firefox-final.json`. The compiled WASM and binding hashes are identical across
  both runs. One function compares all1471 constructors; the other checks real
  asynchronous addon loading for malformed head and kern fonts. Each rejected
  font retains fallback ranges, caches failure and does not refresh. A different
  valid family loads successfully and refreshes once. The shared source scenario
  asserts identical operation traces for both inputs.

The first full browser run timed out in Firefox after60 seconds. This failure is
retained, not relabeled as a pass. A bounded180-second diagnostic on the exact
same artifact completed in115.44 seconds. The existing runner now accepts a
validated `WASM_TEST_TIMEOUT_MS` option and records it in its receipt. A final
Firefox run with180000 completed in117.63 seconds with two tests passing. Six
Node tests reject invalid timeout values before any build or browser job starts.
The default remains60000. No test or source assertion was removed or weakened.

Early `before.log` and `native-first.log` are not the corrected negative control.
A new oracle glyph-count mutation initially reused a descriptor object across
cases, contaminating later inputs. The generator now clones descriptors per
case. Prefix comparison proves all original950 outcomes remain unchanged. The
corrected baseline and final1471-case fixture are the evidence for the fix.

## Reproduction

Use the existing reference Node dependencies, matching wasm-bindgen CLI0.2.100
and an owned short O: worktree. Set TEMP first, then TMP and TMPDIR separately to
that owned scratch directory. Keep Cargo jobs2 and observe the swarm resource
cap. `SOURCE_PACKAGE_JSON`, `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` may select the
existing read-only dependencies and tools.

```sh
node scripts/font-metadata-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --metadata
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
node --test scripts/browser-host-check.test.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser WASM_TEST_TIMEOUT_MS=180000 node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_metadata_browser -- -D warnings
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -j2 -- -D warnings
cargo test --locked --workspace --all-targets --all-features -j2
```

Shell assignments above use POSIX syntax. PowerShell users set `$env:` values.
Generated browser artifacts, private Cargo output and test copies remain owned
scratch and are removed after review and preservation of concise receipts.

## Remaining scope

The source Mac parser warns when its table count exceeds one. This task validates
structural constructor reads, not that warning callback, its reentry or thrown
values. Those remain explicit task342 gaps. CFF/CFF2, GDEF/GPOS, variations,
color/SVG/meta, complete glyph/outline validation and other malformed-font error
ordering also remain342. These browser tests use the actual Rust font reader and
addon with a boundary host, not a full interactive Terminal renderer. Full
original/demo suites, final stable-union quality and native Linux/macOS execution
of this new patch are not established by these scoped Windows receipts.
