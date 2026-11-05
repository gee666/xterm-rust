# Abigail task457 handoff

Task457 is accepted atf9a29280. Final source/light review verified all47 producer pins, six browser artifacts and native2600 cases, original/copy776 hashes, and source520+4 addon plus actual Rust and hint regressions in all three engines. Full342 and overall port equivalence remain NOT PROVEN.

## Exact pickup

Owned source commits are1a08d4b4 and7e2eef24. Private prerequisite merges are not independent source contributions to replay. The verified first-parent chain is4425f01a73e → own1a08d4b4 → merge-main dab1cf68 → own7e2eef24 → accepted453 private merge79b1ff05 → receipt3b8b5477 → browser receiptf9a29280. Task463/Harry already assembles accepted442/447/453 separately.

For that composed loader, take only the457 changes below. `results/font-cff-header/pickup.json` pins the exact owned source/support/evidence paths and classifies the two composed files.

- Add public FontError::CffVersion major/minor and its exact source Display message.
- Add font_cff_header module. Select final CFF before final CFF2 only in the non-glyf/loca branch, after name parsing and before hmtx. Preserve453 loca/glyf/metric helper and447 GPOS.
- Move shared font_hint_deflate module to font.rs under font_deflate. font_hint_bytes reuses that sibling. The extended307-line walker retains a caller-provided bounded prefix while validating the entire compressed stream. Hints still discard output. The CFF caller keeps at most five zero-filled bytes, allowing source short/overflow/stored/history semantics without full-table allocation.
- The420 eager hint hook must use current refreshed payload bytes and the original fixed directory descriptor. The private union explicitly repairs this after442; do not restore the stale original payload reader.
- The signature reader changes from byte indexing to equivalent u32.to_be_bytes. Current privatefont_bytes348 lines; adding447's errors will need the semantic FontError separation Harry already planned. Do not introduce a nominal line-count suppression.
- Add only new CFF tests, fixtures, helper/scenario JS and three scripts, plus docs/intent, this handoff and results/font-cff-header receipts. Do not replay copied442/453 or unrelated main files wholesale.

## Proof and commands

`docs/font-cff-header.md` records contract, commands, exact scope and remaining gaps. Source520 cases have66 accepted/446 errors/8 warning throws, seed0x457cff01. Four actual-addon loads cover rejection, cached failure, recovery and prior error. Native4 functions cover520+792+612+676=2600 constructors. Warning-denied scoped native/WASM lint passes. ExactfullfmtOS206 is retained; all five memberfmt checks pass.

`results/font-cff-header/source-browser.json`, rust-browser.json and hint-browser.json plus their logs bind Chromium148/Firefox150/WebKit26.4 receipts. Source520+4 addon each passes; actualRust2 functions per engine compare520 inputs byte-for-byte and execute the four loads. Shared inflater2 functions per engine replay792 constructors/actual-addon loads and the separate safe termination guard. The old strict miniz output guard fails retained finite cases, restores exact bytes in finally and current520 passes.

The isolated reviewer reran both Node oracles and hash-pinned native executables and verified all final browser artifacts without heavy jobs. `independent-review.md` retains its acceptance. Generated browser binaries are temporary, not release distributions, and may be removed after review. Concise receipts and hashes survive.

## Cleanup

Only O:/xterm-rust-swarm-16/tmp/worktrees/Abigail457 is owned temporary work. No dependency junction was created. All compiler/browser/server grants were released on post409/3102 before final review. No main-ref or remote CI operation was performed. Preserve this branch and concise evidence, scan every link/redirect/process, remove only own generated untracked browser directories, then normal git worktree remove. The subsequent absence/unregistration and original776-source check is reported on409 and588; do not assume planned cleanup happened without that receipt.
