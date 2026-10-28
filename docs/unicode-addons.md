# Unicode 11 and Unicode 15 addons

These modules implement Unicode providers and native addon lifecycle. Full terminal/browser equivalence remains NOT PROVEN. The original Playwright suites have not yet run against Rust-backed browsers, and no native Linux/macOS result is claimed. Alaric executed both original addon suites successfully in all three engines in `results/upstream-browser/runs/2026-10-27T23-59-39-134Z-65e3d213-db85-489a-811d-1d08350d2e8b/`; this is original-baseline evidence, not Rust parity.

## Source and API mapping

| Source under `source_repo` | Rust implementation |
| --- | --- |
| `addons/addon-unicode11/src/UnicodeV11.ts` | `src/unicode11.rs`, generated `unicode11_tables.rs` |
| `addons/addon-unicode11/src/Unicode11Addon.ts` | `unicode_addons::Unicode11Addon` |
| `addons/addon-unicode-graphemes/src/UnicodeGraphemeProvider.ts` | `unicode_graphemes::UnicodeGraphemeProvider` |
| `addons/addon-unicode-graphemes/src/UnicodeGraphemesAddon.ts` | `unicode_addons::UnicodeGraphemesAddon` |
| `addons/addon-unicode-graphemes/src/third-party/UnicodeProperties.ts` | `unicode_properties.rs`, generated `unicode15_tables.rs` |
| bundled `unicode-trie.ts` and `tiny-inflate.ts` | data decoded at generation time, exact native change-point lookup at runtime |

Rust paths are relative to `crates/xterm-core/`. Provider implementations use the shared `UnicodeVersionProvider`, `UnicodeCharWidth` and raw `UnicodeCharProperties` contracts. No JavaScript terminal logic runs inside the Rust implementation.

`UnicodeGraphemeProvider::new(bool)` maps `handleGraphemes`; `Default` enables it. `ambiguous_chars_are_wide: Cell<bool>` preserves mutation after registry insertion. Versions remain exactly `11`, `15`, and `15-graphemes`. Rust UTF-16 helpers accept `&[u16]`, preserving surrogate code units. Codepoint arguments are u32, not `char`. This retains surrogate and out-of-Unicode numeric behavior. Arbitrary negative, fractional or nonfinite JS codepoints remain a binding-conversion concern, not a claim about the typed native interface.

`unicode_properties` exposes the source helper algorithms: `get_info`, `info_to_width_info`, `info_to_width`, `str_width`, `column_to_index_in_context`, `should_join` and `should_join_backwards`. Constants have uppercase descriptive Rust names. The source grapheme-break shift is zero, so masking directly preserves its result. Column values are f64 so negative/NaN/infinite comparison behavior is preserved; starting indices are usize.

The native addons activate against the actual shared `UnicodeService`. `headless::Terminal` implements the host contract, with integration assertions owned by task 30. They also implement the generic shared `TerminalAddon<T>` trait for `T: UnicodeAddonHost`. The host obtains the live service and enforces public proposed-API access before addon activation. `Rc<UnicodeService>` implements this host for direct registry/manager integration. The headless terminal adapter is coordinated with task 30, and browser bindings with task 31. A direct service test is not a completed terminal binding.

Unicode11 disposal intentionally leaves the registered provider unchanged, as upstream. Grapheme disposal restores the saved active version but leaves both providers registered. Repeated direct disposal emits again. The shared AddonManager's managed handle is separately idempotent, matching its source wrapper. Activation and disposal release all internal RefCell borrows before service/event callbacks. Errors propagate with Result and retain committed state in source order. Callbacks should capture Weak references rather than create cycles.

## Data representation

Unicode11 copies the four original interval sets and fill order. Unicode15 uses 2,269 generated change points covering all 1,114,112 codepoints. Generation executes the bundled trie and inflater exactly once; runtime lookup is native Rust binary search. This avoids bringing a general DEFLATE decoder into the engine solely for a fixed Unicode table.

The generator uses the browser's `atob` path. In Node, Buffer pooling gives a nonzero byte offset that the bundled browser trie DataView does not account for. Emulating the browser path is necessary to decode the intended original data, not a change to its values. The generator records hashes for every original provider, property, trie, inflater and test source. Tables retain MIT and xterm.js authors' notices. Generated Unicode data and complete oracle fixtures are cohesive data files allowed over 350 lines; implementation files remain below the limit.

Unicode 6 and 11 share interval searching and zero-width joining helpers in `unicode.rs`. The accepted V6 full-range/property tests are rerun after this extraction. V6's combining-over-wide and V11's wide-over-combining precedence remain distinct.

## Acceptance and verification

The detailed contract is `docs/acceptance/unicode-text.md`, criteria 126 through 130. Run from the repository root:

```sh
node scripts/unicode-addons-oracle.mjs
cargo test -p xterm-core --test unicode_addons --test unicode_addon_manager --test unicode_v6
cargo test -p xterm-core --test unicode_addons --test unicode_addon_manager --test unicode_v6 --release
cargo clippy -p xterm-core --test unicode_addons --test unicode_addon_manager --test unicode_v6 -- -D warnings
```

Use `node scripts/unicode-addons-oracle.mjs --generate` only for deliberate regeneration. Normal verification compares exact committed tables/fixtures and fails on drift. No assertions or source methods are replaced. The generator removes type-only imports/exports, evaluates unchanged provider/helper bodies, and uses the actual `UnicodeService.getStringCellWidth` method for width assertions. It does not claim to execute Playwright or a browser terminal.

Oracle scope recorded in `results/unicode-addons/oracle.json`:

- Five configurations: Unicode11 and all four Unicode15 grapheme/ambiguous-width combinations.
- 1,114,112 codepoints per configuration, including lone surrogate codepoints. Width and six raw preceding states are compared, totaling 5,570,560 widths and 33,423,360 packed-property comparisons.
- 6,144 forward/backward helper pairs, 448 UTF-16 column-position cases, and 256 width-helper property bytes.
- 1,280 persisted 64-step property traces, totaling 81,920 states, seed `0x15cafe`.
- 128 persisted 32-step lifecycle traces run the actual original addons, UnicodeService, Event and Lifecycle. Rust compares 4,096 intermediate active versions, registry orders, emitted events and exact error outcomes.
- All 13 original grapheme width expected values and the Unicode11 emoji width assertion, plus native registration/version assertions. Browser suites remain unrun.

The Rust test files execute only Rust providers, helpers, UnicodeService and AddonManager. An assertion failure names the codepoint/configuration or exact persisted trace case and step. Seven provider/helper/lifecycle tests and three shared-manager tests passed locally in debug and release on native Windows x86_64, Node v24.15.0 and Rust 1.85.1. The approved shared-helper extraction also passed all five V6 regression tests in both profiles. Focused rustfmt and warning-denied Clippy passed. Current logs are retained under `results/unicode-addons/`; earlier failed shared compilation attempts were reported explicitly and do not count as passing gates. Rust/WASM component tests are in `crates/xterm-web/tests/unicode_addons.rs`; attempts under `results/browser-host/unicode_addons/` currently fail shared compile dependencies before browser execution. No result here establishes native Linux/macOS, all WASM addon bindings or real browser terminal integration.
