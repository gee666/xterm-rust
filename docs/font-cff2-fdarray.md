# CFF2 missing FDArray rejection

Task519 follows accepted492 and505. Production changes only `crates/xterm-addons/src/ligatures/font_cff2_dict.rs`. The source is unchanged `source_repo/node_modules/opentype.js/dist/opentype.js`, SHA-256 `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`. See `licenses/opentype-js-MIT.txt`.

## Contract and scope

`parseCFFTable` reads its global subroutine INDEX, lexes the whole bounded top dictionary, applies duplicate-key decisions and then interprets the FDArray entry. Absent entries become null. Null, zero, negative zero and NaN cause the literal `This is a CFF2 font, but FDArray information is missing`. Numeric nonzero values, including infinity, are truthy. Zero or multiple operands remain JavaScript arrays, which are truthy even when empty. An earlier replaceable NaN or multi-element entry can be replaced by a later zero. Duplicate or later lexical errors must still precede the missing-FDArray error.

The parser now records this truthiness during the existing operand pass and checks it after dictionary errors. No second lexer, public API, dependency, unsafe code or FontError variant was added. CFF1 is untouched. Existing unit assertions still test the original lexical/entries phase through a test-only adapter. New assertions test the later constructor guard directly.

This is NOT complete FDArray support. Later FDArray INDEX traversal, per-font/private dictionaries, FDSelect, charstrings, CFF1 dictionary interpretation and existing Bounds/Type browser wording remain task342. The previously documented negative-blend source hang and Rust rejection are unchanged, not approved or called equivalent here. Full equivalence is NOT PROVEN.

## Regression corpus

`scripts/font-cff2-fdarray-oracle.mjs` calls the complete unchanged source constructor and retains its outcome and warnings for all 860 cases, seed `0x519cff02`. It is read-only unless `--generate` is explicitly requested. Both CFF directory tags, SFNT/WOFF, 32/64KiB inflater boundaries, operand encodings, NaN replacement, blend retention, lexical/duplicate/global INDEX ordering, required-table phases and CFF1 controls are included. Every reconstructed input has SHA-256 and FNV-32 pins.

- 852 cases compare the complete constructor result in Rust, including 188 exact missing-FDArray errors.
- 8 separately marked `presenceOnly` cases retain a source Bounds error from the later unported INDEX phase. Rust checks only that this guard does not incorrectly reject their truthy array/infinity value. They are NOT reported as full constructor matches. Multi-element truthiness also has a direct Rust dictionary test.

Actual WASM tests reconstruct each input independently in JavaScript, compare bytes with the Rust recipe, run the Rust corpus and predecessor assertions, and load bad/good inputs through the actual `LigaturesAddon`. The existing asynchronous source-addon scenario tests error logging, cached rejection, fallback ranges, good-font recovery, reads, refreshes and disposal. Source and Rust use the same scenario, not a JavaScript font parser in production.

## Reproduction

Use an owned O: worktree and explicit team resource grant. Set `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR` to its regular target and TEMP/TMP to its owned tmp directory. Reuse the installed original source dependency path and matching wasm-bindgen0.2.100. No package installation or source change is needed.

```sh
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-fdarray-oracle.mjs
cargo test -j2 -p xterm-addons --all-targets --all-features
node scripts/font-cff2-fdarray-negative.mjs
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-font-addons --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-fdarray-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff2_fdarray_browser WASM_TEST_TIMEOUT_MS=600000 node scripts/browser-host-check.mjs
```

The negative control bypasses only the presence guard, requires 188 actual runtime mismatches, restores the exact source SHA in `finally` and reruns the positive test. Compiler-only failure is rejected. Existing source fixtures/assertions are not changed.

## Verification status

Source Node passed 860 cases, with 852 complete outcomes and 8 presence observations. Predecessor505's 898 source recipes replay unchanged. Its 14 original producer pins were verified against `archive/Ian505-complete`, plus 776 original and 776 owned physical-copy reference hashes. Running the old505 audit directly after modifying its parser pin correctly failed at that pin, and was not called a current pass.

Native Windows addon all-target/all-feature tests passed 208 functions across 48 harnesses, with zero failed or ignored. Strict all-target/all-feature Clippy passed for native xterm-addons and WASM xterm-font-addons. Affected workspace native library checks and strict WASM library Clippy passed. The new WASM test compiled. Five member format checks passed; exact `cargo fmt --all -- --check` still failed with Windows OS206 and is not reported green.

The negative guard bypass executed all 188 required mismatches, consisting of 128 accepted inputs and 60 later hmtx errors. Exact source SHA restoration and the positive replay passed. An initial helper module-inception lint failure was fixed by renaming the owned test module, with no suppression. The initial negative counter wrongly expected all mismatches to be successes rather than including those later hmtx errors. It was corrected to require the exact 188/128/60 split, without changing the corpus or any assertion. Both initial failures and successful reruns are retained.

Three-browser execution is pending. Raw results are under `results/font-cff2-fdarray/`. These are focused gates, not a new full workspace test pass. No Linux/macOS runtime result, full original-suite result or overall equivalence is claimed.

## Integration boundary

Base main1106d480. Accepted505 was copied privately as `3bbe523f`, `683bbdd9`, `d338987d`, `8950b8c0`. Exclude all four from519 pickup and apply519 only after original505. No492/495 prerequisites were replayed, no main ref or CI action was performed. Gordon owns integration309/520.

Only `O:/xterm-rust-swarm-16/tmp/worktrees/DhanvantariCffFdArray` is owned temporary work. All generated outputs and process trees must be inspected before removal. Retain accepted commits, compiler pins and raw receipts before unlinking generated files. Verify original reference hashes after normal worktree removal; never traverse a reference junction or clean another owner's paths.
