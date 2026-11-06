# Task546 pickup

Accepted production bed44a4b and evidence997743d0. Only these two commits plus the later review/cleanup metadata tail belong to546. The final post names that tail hash. Preserve archive/Nefertiti546-complete.

## Integration boundary

Base main0442e357 plus ONLY533 private prerequisites4c1df0b7,86741f25,37cb3699,99e11d08,db4ca508,999f5134,d58c0dfd. EXCLUDE ALL SEVEN from546 pickup. Original519 is already on main, no519/505/492 replay. Urraca owns309 integration/resources. Main advanced to224d1b33 during this task. This producer did not modify main, run canonical merge or take CI action.

Production files font_cff_index.rs, font_cff_address.rs, font_cff2_dict.rs and new font_cff2_fonts.rs add eager CFF2 per-font/private dictionary and local Subrs reads. Shared lexical parsing preserves deferred duplicates, numeric coercion, whole-font private bounds, all-INDEX-before-DICT precedence and private-before-next-font ordering. The test container writer was extracted unchanged. Existing source assertions remain intact; native tests retain the full predecessor chain. No new dependency, public API, unsafe code or JS font-engine delegation.

FDSelect, charstrings, CFF1 interpretation, other342 table families and existing Bounds/Type browser wording remain open. Infinite private-size/NaN-address, negative-blend/string-range and empty-object resource optimizations are explicit safety divergences, not approved equivalence. Overall equivalence NOT PROVEN.

## Evidence

Read docs/font-cff2-fonts.md and results/font-cff2-fonts/checks.json. Seed0x546cff02 defines408 recipes. Generation and immutable replay observed396 full constructors and392 table phases. Twelve Subrs=1 inputs encounter huge count/width0 and exhaust the source128MiB child heap before either result. Exact inputs/stderr are retained. They are12 unrun comparisons, not passing rejections or matches, and are excluded from source browsers. Rust executes them as bounded observations only.

Native Windows addons all-target/all-feature222 functions/52 harnesses passed,0 failed/ignored/measured/filtered/warnings. Strict native-addon/WASM-font all-target/all-feature lint and native/WASM dependent checks passed. Initial test-wiring lint failures are retained and fixed without suppression. Exact workspace format fails OS206, while five full member checks pass. No full-workspace or Linux/macOS claim.

First source396 safe constructors plus4 actual addon loads passed EACH Chromium/Firefox/WebKit. Actual Rust/WASM2 functions passed EACH3,0 failed/ignored/filtered/pageerrors, comparing396 constructors and408 independent recipes with12 explicit observations plus actual error-cache/fallback/recovery/read/refresh/disposal lifecycle. Source/Rust outer times19.08s/434.75s, per-engine deadline600s. Immutable compiler b84d10f553aa60a47c6444480944da230f0c128b9f7bf5c36db71f6cd120f51e is retained under results/acceptance-artifacts.

Negative bypass executed184 constructor mismatches, including36 duplicate0 and all4 designated addon-bad cases; exact source SHA restoration and positive replay passed. The reviewer reran read-only2302 producer pins,18 runtime pins,776 original plus776 physical-copy hashes and cached13 private tests, including392 eager and450 predecessor phase matches. No reviewer compiler/browser/source worker was launched. All24 generation/replay resource signatures are verified. Raw log bytes are base64/SHA-pinned separately from readable whitespace-clean views. Text producer pins normalize CRLF only and retain original raw hashes; binary/runtime hashes stay raw.

Before cleanup, reproduce with REFERENCE_ROOT pointing to the original P: source_repo and node results/font-cff2-fonts/audit.mjs --cached --runtime. After cleanup, cached/native runtime outputs are intentionally absent. A fresh producer checkout can use the same command without those flags and with --original-only. No source oracle worker or browser needs to run to inspect retained proof.

## Cleanup boundary

All heavy/compiler/browser/server resources were released at409 comment4708. Only O:/xterm-rust-swarm-16/tmp/worktrees/NefertitiCffFonts is owned. Initial preflight scanned18689 files/3283 directories with0 links/redirects/owned-root processes,2302 source pins and776x2 reference hashes. Retain compiler/build logs and runtime hash receipts before unlinking18 generated browser outputs. Preserve all other owners, source_repo, archive/source branches, main, credentials and AGENTS.md.

This handoff is NOT an AFTER-removal claim. Require later root absence/unregistration, normal git removal exit0 and original776 hashes/opentype SHA AFTER receipt. That private receipt will be .pi/swarm/work/2/nefertiti546-cleanup.json, named in the final post.
