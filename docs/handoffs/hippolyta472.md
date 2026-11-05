# Hippolyta task472 pickup

Task472 is ACCEPTED at152d2725 after resumed independent review. It implements initial CFF INDEX reads only. Full342 and overall equivalence remain NOT PROVEN. `docs/font-cff-index.md` records the contract, commands and remaining gaps. Existing browser Bounds RangeError wording is deliberately retained as a known gap, with raw source and Rust receipts, not claimed equal.

## Exact source ownership

Branch `swarm/HippolytaCff` contains own commits0cf23394 and9af0e34b, then8c0a30ff for final browser import wiring and native source pins. Prerequisite c75d716a is accepted457 plus its private442/453 history. Merge9146490b preserves main3a4f5e8f's accepted font union and cohesive FontError split. Neither prerequisite nor merge history is a new producer contribution to replay. Use only this task's final paths/pins on top of the integrated457 baseline.

The production changes are `font_cff_header.rs`, new `font_cff_index.rs`, `font_hint_deflate.rs`, new `font_deflate_output.rs` and its native test module. No FontError, font.rs or font_bytes producer delta is needed. Header selects the same tables and preserves earlier phases. The shared inflater now dispatches through an output trait, preserving the original prefix sink and adding a bounded32KiB history/range sink. CFF initial indexes retain only their offset bytes, validate object read ranges, and never allocate origLength. Keep all new tests, fixtures and scripts listed in `results/font-cff-index/producer-hashes.json`.

## Evidence

Read the latest `results/font-cff-index/checks.json`, not old logs labelled initial or bounded without final. Source476 full constructors have56 accepts and420 bounds errors, seed0x472cff01. Native all-target/all-feature xterm-addons has187 passing test functions over40 test harnesses, no failed/ignored cases. The focused constructor run also replays520header,792hint,612outline and1571GPOS cases. Shared-window unit checks patterned70k stored/fixed/dynamic output across seven declared limits and five captured ranges. Negative bypass gives420 failures, exact bytes restore and the positive corpus passes again. Scoped native/WASM strict lint/build pass. Fullfmt OS206 remains distinct from five passing packagefmt checks.

Initial independent source/light review found no code defect, reran476 source cases and existing native executables, and verified23 pins. The resumed review accepted the final runtime evidence, verified all seven artifact pins and reran native tests. Final source57b3fe52 and receipt152d2725 retain source476+4addon loads and actual2WASM functions per Chromium/Firefox/WebKit. Both180s Firefox timeouts remain alongside measured finite phases and a final284.14s pass under a360s bound. Assertions were unchanged. checks.json records the final source and runtime status. Original and physical copy776-file hashes were verified after the final runs.

## Temporary ownership

Only `O:/xterm-rust-swarm-16/tmp/worktrees/HippolytaCff` is owned temporary work. It was created with a physical source copy and no dependency junction. No main ref or remote CI operation was performed. Retain final branch, concise receipts and pinned compiler WASM before deleting generated browser directories and normal worktree removal. `cleanup-preflight.mjs` verifies all links/redirects, owned processes, source hashes and producer pins. Cleanup is not complete until a later receipt confirms absence/unregistration and rechecks original hashes after removal. Never infer that planned cleanup ran.
