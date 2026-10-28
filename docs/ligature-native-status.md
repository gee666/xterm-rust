# Native ligature status

Task119 implements the Rust cmap/GSUB ligature pipeline, graph merging, lookup traversal and LRU cache. It is exported as `xterm_addons::ligatures::font`. Full font-addon equivalence remains NOT PROVEN.

## Checks

- Original three suites: `node scripts/ligature-font-oracle.mjs` passes449 tests and669 assertions. Expected traces contain2,540 font calls,217 directed graphs, five original merge cases and eight range cases.
- `cargo test -p xterm-addons --lib ligatures::font` passed six tests after production integration.
- Current actual source modules compiled with `rustc --test -D warnings` pass seven tests, including six WOFF comparisons. This isolated run was used while unrelated shared-tree edits broke Cargo compilation.
- The same production modules pass `clippy-driver -D warnings`. Whole-package Clippy is still affected by unrelated core edits; no whole-workspace clean claim.
- `node scripts/woff-font-oracle.mjs` verifies the derived Monoid WOFF fixture and six source comparisons.

Current isolated logs are under `results/font-addons/native-probe-2026-10-28T07-48-40.533Z/`. Earlier failures and build-slot output remain retained, but no additional receipt machinery is required to review this change.

## Boundaries

`docs/ligature-shaping-acceptance.md` explains source quirks such as coverage range indices, fixed-start substitution segments, merge endpoints and glyph-position ranges. `docs/ligature-native-design.md` describes the cycle-safe arena and cache identity model.

The reader handles shaping-relevant tables. Matching opentype.js constructor rejection of malformed unrelated metadata is separate task235 and remains required for full equivalence. Browser joiner/style/lifecycle wiring belongs to task40; font discovery is task158. Native Linux/macOS and browser integration are not proved by these Windows component tests.

The fonts are tracked, unchanged fixtures with notices in `licenses/ligature-font-fixtures.md`; the derived Monoid WOFF selects its offered MIT licence. Production Rust does not load oracle-generated lookup trees or execute the JavaScript terminal/font engine.
