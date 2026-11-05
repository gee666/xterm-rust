# Browser SerializeAddon

The compatible module is `browser/xterm.mjs`. It exports `SerializeAddon` with `activate`, `serialize`, `serializeAsHTML` and `dispose`. Both serializers read the actual Rust engine, not an xterm.js terminal. The WASM implementation is in `crates/xterm-web/src/serialize_addon/` and shares `xterm-core::serialize` with native consumers.

The JavaScript subclass retains `_terminal`. Rust retains only a weak `Shared` reference. The existing Terminal addon-manager WeakMap owns the loaded addon records. An addon can keep its host alive without making the otherwise unreachable Terminal/addon cycle a Rust-held JavaScript root. Disposal deliberately does not clear the host, matching the original empty dispose method. Actual forced-GC and unreachable-cycle evidence is still required by task346.

## Implemented paths

ANSI serialization includes normal and alternate buffers, cursor attributes, modes and margins. Each option phase runs after the preceding buffer operation. Integral number and marker-line ranges use the existing shared algorithm. Buffer access is ungated, including default `allowProposedApi: false`.

HTML serialization captures the cached public active view and numeric boundaries before header getters. Reset retargets that view; alternate activation leaves a captured normal view normal. The real selection service supplies end-exclusive bounds. Explicit range takes precedence over selection and scrollback. Before open, palette colors use the source default fallback. After open, colors come from the actual ThemeService. Global header values use default-hint UTF-16 concatenation without sanitizing source-visible raw strings.

The core `BufferAccess::missing_cell` hook lets the browser emit the original console.warn diagnostic and propagate a thrown JavaScript value through the normal error chain. Native callers keep the default no-host diagnostic behavior. `PublicBufferView` and live mode serialization are shared APIs, not copied browser algorithms.

## Explicit gaps

Full serialization equivalence is NOT PROVEN. Task346 and family322 retain SB01-SB08 in `docs/serialize-browser-acceptance.md`.

- Native row/column ranges are unsigned integers. Negative/fractional/NaN/infinite/large JS coordinates are not equivalent. The browser returns an explicit representation error rather than truncating them. This is a compatibility gap, not a source-approved divergence.
- Marker getters and custom coordinate coercions need their exact late source evaluation points. Numeric coercion currently happens earlier than source traversal.
- Public Terminal/buffer method or descriptor replacement is not fully honored. The serializer uses the actual native public namespace rather than calling arbitrary replaced methods.
- Theme palette array replacement and element mutation during header reentry need array-identity modeling. Normal live theme/OSC changes between calls are covered; a copied list of color handles does not prove arbitrary reentrant palette mutation.
- ANSI warning callbacks now have fallible late reads for rows, buffer length, cursor and current attributes. Traversal still reacquires a line for each cell rather than pinning the source row's line object. Reset or reflow during an earlier warning therefore still needs source/Rust line-identity checks.
- GC/WeakRef tests, every malformed UTF-16/number case, source/restored full internal margins/events and complete demo matrices remain unrun here.
- Equal infinite start/end bounds make the source row++ loop non-progressing. The numeric probe records that case as unrun rather than hanging a browser. No source-hang compatibility approval is claimed.

## Reproduction

Use the installed wasm-bindgen CLI matching the locked 0.2.100 runtime. Set `WASM_BINDGEN` when it is not on PATH. Set `SOURCE_ROOT` to the supplied unchanged reference, which must have the original generated demo bundle and Playwright dependencies. From an owned worktree, with the current swarm resource allocation:

```sh
CARGO_BUILD_JOBS=2 node scripts/build-demo.mjs --debug
SOURCE_ROOT=/path/to/source_repo node scripts/serialize-browser-check.mjs --suite
cargo test -j2 -p xterm-core --test serialize_diagnostics
cargo fmt --all -- --check
cargo clippy -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
```

The browser command compares identical operations in Chromium, Firefox and WebKit. It then runs the unchanged original SerializeAddon Playwright suite against both engines. A mismatch returns nonzero, including known raw numeric gaps. It never filters those gaps into a passing total. Generated servers have finite lifetimes and finally cleanup; transient runner files stay under the owned root `tmp/`.

`results/serialize-browser/` contains source hashes, browser versions, exact counts, errors, raw operation/restoration traces and visible DOM screenshots. The seeded corpus uses `0x05e21a11`, with 32 traces of 12 operations. Restored Rust state is compared to restored source state, not assumed to equal the original source state after lossy source serialization.

## Warning callback state reads

Task359 fixes ANSI state restoration after reentrant `console.warn`. The source reads current rows during row-end handling, current length and rows for the trailing-blank fixup, then cursor and finally attributes. `BufferAccess` now has fallible hooks at those points. Native immutable callers keep their supplied context; the actual browser host reads live native/public-view state without holding borrows across callbacks.

Three new source/Rust probes cover reset from a last-column warning, nested serialization before reset, and thrown warning identity. Before the browser fix, both reset probes omitted the final SGR reset in Rust. The negative control matched48/51 cases per engine, with these two defects plus the existing raw-numeric case. After the fix it matches50/51 in Chromium148, Firefox150 and WebKit26.4. The remaining raw-numeric mismatch still makes the runner exit nonzero. All three new probes match in every engine.

The unchanged original SerializeAddon suite passes120/120 cases against source and120/120 against Rust across the three engines. All13 native serialization test binaries pass55 test functions, including5 new late-state regressions; the new tests also cover cursor exclusion and propagation of each late-read failure. Native focused and WASM library warning-denied Clippy and exact `cargo fmt --all -- --check` pass. These are scoped Windows Rust1.85.1 results, not a full workspace or native OS matrix.

Reproduce using the commands above plus `cargo test --locked -j2 -p xterm-core --test serialize_reentry`. `results/serialize-reentry/` retains native/build/lint logs, the baseline report and its three warning probes. `results/serialize-browser/` retains the final complete browser traces and original-suite reports. The first run's stale48-case runner guard is retained separately; it was corrected to51 after adding the three probes, without changing source assertions. Browser proof is on the focused branch before final integration. Overall equivalence remains NOT PROVEN.

## Public cell enumeration repair

Task349 corrects observable buffer enumeration defects. Four unchanged round-trip tests per engine stringify the actual BufferLine, which previously exposed only an allocation-dependent `__wbg_ptr` in Rust. The Rust line now exposes enumerable read-only `_data`, `_combined`, `_extendedAttrs`, `length` and `_isWrapped` captures with actual packed words, combined strings, extended attributes, logical length and wrapping. A separate source/Rust regression verifies those meaningful packed values. Cell snapshots expose live enumerable own fields in source order and hide the internal pointer from enumeration. No `toJSON` special case or assertion adapter is used. Extended snapshots expose the raw `_ext` and `_urlId` fields separately from the computed `ext` getter.

The row capture copies packed words and strings. Its extended handles retain actual shared payloads, but it is not a writable alias to source private storage and does not expose source transient string caches. Cell fields are enumerable accessors over current Rust values, not complete source data descriptors. Arbitrary raw field values, signed packed words, shared extended-object identity, descriptor replacement/deletion and enumeration of a payload newly assigned after construction remain public-API family315 work. Do not upgrade this repair to complete JavaScript object-shape parity.
