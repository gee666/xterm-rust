# Addon coverage audit

Full equivalence is NOT PROVEN. This audit inspects integrated main `224d1b339a43eddf899c2944d9dad734d8c44234`, not unintegrated producer branches. Task561 refreshes the all-fourteen-addon part of task328. It does not finish the final source/API/assertion audit, accept an implementation family, or rerun terminal behavior.

## What changed

The older compatibility snapshot marked every addon missing, even when a real exported Rust binding and retained execution existed. It also said that no public browser binding was implemented. Those statements were obsolete. The refreshed input `tools/compatibility/addon-mappings.json` links 74 inspected production/declaration modules, 33 source-qualified API owners and all 50 addon suite/support files to scoped implementation, tests, executable commands and evidence. The mapping is deliberately data, not proof that every declaration member or assertion has equivalent behavior. Its repeated per-record references keep each generated row independently readable; this cohesive machine-readable data exceeds the 350-line code guideline.

Across the complete inventory, 95 of 209 modules and 168 of 770 declaration/member records now have partial mappings. The remaining 114 modules and 602 API records are unmapped, not necessarily absent from Rust. The fourteen-addon subset has 74 partial and 17 unmapped production/declaration modules. Common, headless, browser, CSS and full public declarations still need task328 reconciliation. No mapping is marked equivalent or complete. All 162 fixture rows remain explicitly unrun by this index; that is a missing fixture-runner mapping, not evidence that no fixture tests exist elsewhere.

No reference assertion or source byte changed. All 776 reference hashes are retained. The static inventory still has 2,426 test definitions and 32 benchmark definitions across 120 suite/support files. These are not runtime case, assertion or Rust test-function counts.

## Fourteen-addon findings

The counts below are historical receipts linked by the new mappings, not fresh runs by this audit. No totals are added across different scopes or revisions. `full-suite-checkpoint` uses frozen `e1651049` and predates many repairs. The canonical inventory reads the actual `results/upstream-baseline/latest.json` pointer, while the later full-suite receipt retains its own original unit run. Neither is silently replaced by this refresh.

| Addon | Inspected Rust and browser contract | Retained evidence and remaining work |
| --- | --- | --- |
| attach | `xterm-addons::attach`, `utility_addons/bindings.rs`, browser WebSocket and live queued writes | Utility/source oracle, actual WASM tests and historical 6/6 original cases. Full raw host/reactivation/disposal/GC family323 remains open. |
| clipboard | Native clipboard/base64 plus exported `ClipboardAddon`, `Base64`, `BrowserClipboardProvider` | Utility/codec tests; historical original 20 pass and 10 original skips. Permission/clipboard host behavior, raw callbacks and final matrices remain323/330. Skips are not passes. |
| fit | Native geometry helper plus real DOM `FitAddon.fit/proposeDimensions` and Terminal.resize | Historical 17/27 original cases pass; ten geometry failures remain visible, including in the source baseline. Full raw lifecycle/measurement/final geometry is323. |
| image | Actual exported `ImageAddon`, Rust coordinator, shared storage/rendering and IIP/SIXEL/Kitty protocol producers | `results/image-addon/{verification,browser-host}.json` records directed real-terminal coverage. The old full suite failed before the export existed. The complete unchanged 639-case matrix, img01..48, raw/private/GC and broader seeded coordinator closure remain311/324. |
| ligatures | Portable Rust parser/shaper, browser font discovery and real Terminal joiners through `xterm-font-addons` | `results/font-terminal/summary.json` and `results/font-legacy/legacy-results.json` retain loaded-font and 8/8 legacy assertions. Eight source helper/type modules remain unmapped. No original ligatures Playwright config exists.342/335 retain malformed font/raw joiner closure and the documented empty-pattern safety divergence. |
| program-status | Portable parser/store/addon and Rust WASM status records/events | Source oracle and historical 12/12 original browser cases. Scoped record/clear fixes are not complete family323 or individual-assertion coverage. |
| progress | Portable progress parsing and exported mutable JS event/state bindings | Source oracle and historical 30/30 original browser cases. Full raw setter/event/disposal/reentry and cross-addon closure remain323. |
| search | Portable scan/cache/state plus actual Terminal selection/highlights, timer host and JS RegExp boundary | `results/search-browser/wasm-search-final.json` and historical 81/81 original browser cases.347/527/322 still own arbitrary raw terms/options, disposed-cache liveness and family closure. |
| serialize | Shared portable ANSI/HTML plus live Rust buffer/selection/theme and supported JS ownership subclass | `results/serialize-browser/verification.json` explicitly fails its remaining raw numeric corpus case while retaining 120/120 unchanged source and Rust original-suite cases. Later scoped repairs exist.346/322 retain complete raw/accessor/palette/GC closure. A passing original suite does not erase the corpus failure. |
| unicode-graphemes | Rust Unicode15 tables/properties, joining service and exported addon activation/restoration | Native source corpus and historical 3/3 original browser cases. Compressed JS trie/inflate loaders have no direct Rust module mapping because Rust consumes generated tables. Full generated-data/property, malformed input/provider and assertion closure remains316/328. |
| unicode11 | Rust width tables/service and exported addon; dispose intentionally does not restore the old version | Native source corpus and historical 3/3 original cases. Distinguish this disposal contract from grapheme restoration in demos. Full316/328 and final matrices remain open. |
| web-fonts | Rust exported `WebFontsAddon` and `loadFonts` over FontFace/FontFaceSet promises | `results/font-terminal/playwright.json` retains 15/15 original cases. Raw async mutation, promise/disposal/GC and family342 closure remain open. Native relayout helpers are not a native browser loader. |
| web-links | Portable link provider plus Rust WASM addon, live link registration and JS RegExp/window boundary | Historical 1503/1503 original provider/regex cases. These do not prove live hover/leave/underline, ranges or pointer interaction.321/323/330 remain open. |
| webgl | Individually mapped model/typed-array/cache/atlas/glyph/rectangle/canvas components | No callable `WebglAddon` or complete `WebglRenderer` export on this inspected revision. Those modules and the typings remain missing. Seven source modules remain unmapped.306/558 own live addon/GPU/context/GC closure. The old 96 reported original passes may exercise DOM after the unchanged suite catches a WebGL constructor failure. They are not 96 GPU proofs. |

## Reading the machine-readable mappings

`modules.json` and `apis.json` retain exact source paths and signatures. An owner mapping applies only within its declared `.d.ts` source, avoiding accidental coverage of an identically named type in another package. The existing unqualified internal `IDisposable` mapping remains a deliberate shared fallback. Exact-source entries take precedence. Invalid or duplicate owners and suites are rejected.

Every addon original suite/support file has an explicit suite-to-component guide in `suites.json`. This prevents `test/` files from disappearing merely because their paths differ from `src/`. It does not state that a sibling component test reproduces every assertion. All Rust suite runtime counters remain null until a separate exact execution reconciliation certifies them. Original unit counters still come only from the validated baseline receipt, not static registrations or similarly named Rust files.

`addons.json` derives partial status from mapped components rather than hardcoding every addon missing. In particular, partial WebGL helpers never establish a constructor or GPU implementation. Original source proposed-API gating remains separately indexed; this addon refresh does not change public Terminal option/runtime-contract status.

The generator now hashes both mapping inputs and the behavior-rule input in `evidence-manifest.json`. A source, mapping or evidence hash proves identity at generation time, not that old test results executed current implementation bytes. Changed final integration sources require reviewed regeneration and fresh behavior gates. No pass is manufactured by refreshing hashes.

## Reproduce the audit

With the exact supplied reference and locked TypeScript tooling installed, run from the repository root:

```sh
npm ci --prefix tools/compatibility --ignore-scripts --no-audit --no-fund
node scripts/compatibility-inventory.mjs
node scripts/compatibility-inventory.mjs --check
node --test scripts/compatibility-inventory.test.mjs scripts/compatibility-baseline-links.test.mjs scripts/compatibility-addon-mappings.test.mjs
```

The tests check source/signature inventories, hashes, baseline validity and path/junction guards. New controls check exact-source owner isolation, duplicate/unknown owners and suites, unchanged missing WebGL APIs, explicit partial addon status and null uncertified Rust runtime counts. They do not compile or run a terminal. Commands in individual mapping rows require their documented Rust/WASM/browser prerequisites; the ligature legacy command additionally requires a freshly built Node WASM package as documented in `font-legacy-and-shaping.md`.

No cargo, browser matrix, quality, native OS, benchmark or demo execution is claimed by task561. Current native quality receipts, accepted but not integrated producers, unsafe/license review and final README/verification/comparison reconciliation remain task328 and their implementation owners' work. This audit is not a waiver of any of them.
