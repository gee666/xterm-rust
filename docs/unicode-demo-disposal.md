# Unicode demo disposal assertion

Task541 corrects a custom demo assertion. It does not change the Rust engine, addon implementation or any assertion in source_repo. Full327/330 and overall equivalence remain NOT PROVEN.

## Source contract

The original `addons/addon-unicode11/src/Unicode11Addon.ts` registers its provider during activation and has an empty dispose method. Selecting version11, disposing, reloading, selecting11 and disposing again leaves version11 active and registered. The demo does not impose a separate provider-reset policy.

The original `addons/addon-unicode-graphemes/src/UnicodeGraphemesAddon.ts` instead captures the old active version during activation and restores it during disposal. Reload first disposes the previous addon, so both disposal cycles restore the old version6 in the tested initial state.

The custom demo06 previously required both addons to restore the version captured before loading. That rejects the Unicode11 source behavior. The historical full checkpoint retained three such failures at e1651049. See `docs/full-suite-checkpoint.md` and its unchanged raw demo06 reports. Those failures must not be erased or reclassified as original passes.

## Corrected check

The custom demo now requires version11 to remain active and registered after Unicode11 unload. It also writes the monocle emoji U+1F9D0 and requires cursorX2 and cell width2. Restoring the default version6 would fail both the version assertion and this operational width check. Grapheme unload still must restore the previous version.

The source replayer uses the actual original browser Terminal and addon distributions in Node, without opening a DOM terminal or replacing the Unicode host. Two load/select/dispose cycles per addon verify the version and actual cells. The old restoration assertion is explicitly rejected only for Unicode11. Its successful rejection is a negative control, not a passing browser test.

```sh
SOURCE_ROOT=/path/to/original/source_repo node results/reese327/reproduce-unicode.mjs
```

The original library/addon distributions must have been built first. This command writes only `results/reese327/unicode-disposal-source-assertions.json` and records hashes of the consumed distributions and original TypeScript lifecycle files. The supplied original source remains read-only.

## Verification status

The source replay passed on Windows with Node24.15.0. Unicode11 stayed11 and produced width2 in both cycles. Graphemes returned6 and produced width1. `node --check` and the targeted Git whitespace check passed for the corrected custom test. Source pins and the exact rejected old assertion are in the replay receipt.

The corrected Unicode11 and unchanged Graphemes demo lifecycle cases passed in Chromium, Firefox and WebKit, six cases total. The full demo06 attempt was still unsuccessful:48 discovered,44 attempted,39 passed,5 failed and4 unrun after its180s global deadline. The failures were links, missing WebGL and WebKit clipboard behavior. All six Unicode cases completed before that deadline. The complete report remains unfiltered at `results/demos/browser/2026-11-06T00-48-58-448Z-58f99517-d4e8-4d41-9b31-b258a7c3de9c/`.

The unchanged original Unicode11 and Unicode-graphemes suites also passed3/3 cases each against Rust on all three original projects. The broad matrix executed every one of the14 original configs, with4116 discovered,3258 passed,113 failed,290 original skips and455 unrun. Setup fallout is not counted as a body pass. `results/reese327/browser-summary.json` recomputes these counts from raw reports and preserves every failing record. None of those broader failures is excused by541 acceptance.

These runs use the verified copied531 debug package with1866-input digest `b9d19e7a3d09fdc5dffa4cc1232b56cf6bf96a905fb111f935452d80decc857f`. All75 package files and build artifacts were checked before and after copying, and the existing no-build verifier checks the source and WASM/binding identity for each command. See `results/reese327/package-copy.json`. No Rust, original assertion, timeout or retry setting changed for this correction. Broader327 baseline execution and application repair remain in progress.
