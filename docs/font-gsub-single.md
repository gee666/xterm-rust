# GSUB single-substitution format read order

Task 530 fixes the type 1 parser's validation order. The source reads substFormat and rejects values other than 1 or 2 before touching coverage. Rust previously dereferenced coverage first. A missing, truncated or invalid coverage pointer could therefore replace the earlier format error. Type 7 extension lookups call the same single parser and had the same defect.

The shared oracle adds 608 deterministic SFNT/WOFF cases. These vary direct/type 7 lookups, selected/unselected calt, mark-filter flags, later invalid META, formats 0/3/65535, absent/half/out-of-bounds coverage pointers, invalid/valid coverage and valid/absent/half-format controls. Source parser and FontImpl each accept 32 and reject 576. The actual source addon checks loading fallback, one buffer read, cache reuse and refresh count for every case.

The fixtures pin opentype.js SHA256737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484 and UbuntuMono font SHA256b35dd9d2131d5d83a9b87fe9ad22c6288fa3d17688d43302c14da29812417d63. Independent Node and reused browser/Rust container writers compare exact input bytes and hashes. Initial fixture generation and subsequent read-only replay are separate retained commands. Existing expected values and assertions are not changed to pass.

Native errors retain the established `FontError::UnsupportedFormat("GSUB single substitution", value)` representation. Source errors include the absolute byte offset in prose. This task fixes parser read/error order and addon rejection behavior, not that existing diagnostic representation difference. General nested null coverage/class handling, shaping, full original suites and full 342/325/OS/comparison proof remain open. Full equivalence is NOT PROVEN.

## Commands

Run from an owned O: worktree, with Node 24.15.0, Rust 1.85.1, installed source dependencies via SOURCE_PACKAGE_JSON, matching wasm-bindgen 0.2.100 and Playwright. Set TMP/TEMP to the owned tmp directory, CARGO_BUILD_JOBS=2, and obtain a resource allocation before heavy jobs.

- `GSUB_CORPUS=single node scripts/font-gsub-lookup-oracle.mjs`
- `cargo test --locked -j2 -p xterm-addons --test font_gsub_single`
- `cargo test --locked -j2 -p xterm-addons --all-targets --all-features`
- `cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings`
- `cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gsub_single_browser -- -D warnings`
- `GSUB_CORPUS=single node scripts/font-gsub-header-browser-source.mjs`
- `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_single_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs`

## Evidence status

Source Node generation and immutable replay passed 608 parser, 608 FontImpl and 608 actual-addon cases. Before applying the production fix, the unchanged native test failed with 384 mismatches. After the fix, the same test passed, including 1,384 retained lookup cases. All-target/all-feature addon tests passed 207 functions across 49 summaries, with no failures, ignored, measured or filtered tests and no warnings. Strict native addon lint, strict lint of the actual new WASM browser target, and dependent font/web WASM library checks passed.

The exact aggregate `cargo fmt --all -- --check` failed with Windows error 206. All five whole-package formatting checks passed. These package checks do not replace the failed aggregate gate. Both 776-file original and physical-copy reference manifests matched.

Unchanged source passed 608 parser, 608 FontImpl and 608 actual-addon cases in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Actual Rust passed both browser functions in each engine, with zero failures, ignored or filtered tests. The synchronous Rust function also replays all 1,384 retained lookup cases. The async function exercises actual Rust addon loading, fallback/cache/refresh behavior across the 608 new cases. These are component checks with a terminal host fixture, not complete Terminal rendering or original-suite proof.

Rust assertion times were 9.01s in Chromium, 126.71s in Firefox and 9.96s in WebKit. The original runner exited normally with status 0, and a subsequent process inspection found zero owned compiler/browser/server processes. The browser/compiler allocation was released in post409 comment4268. Source and Rust ran sequentially under the bounded 900s outer command; Rust used the original runner with a 360s per-engine assertion deadline. There were no browser retries, source assertion changes or recompilation after a passing browser run.

The compiler WASM SHA256 is a13bf200656b30d392e77abb611a0aae1560951908d0bef1fbaa9dd1aae63bdc; served WASM is f0156ad6736c56aa6840e7e196d21cef8b1a5ffbc22610878a62484235f9f557. The report separately pins binding JS, CSS, Cargo.lock and the browser test. Twenty-one additional producer/input hashes are in producer-hashes.json. The current fixed native executable has its own native-artifact.json pin and can be run directly for independent review without compiling.

Commands, raw logs and machine-readable counts are in results/font-gsub-single. Original runner reports/logs were copied byte-for-byte there; generated binaries remain only for independent review and will be removed with the owned temporary worktree afterward. Code/script/docs diff checks passed. The receipt whitespace check reports the original browser logs' final blank line and PowerShell's CRLF output. Those raw receipts remain unchanged, and receipt-whitespace.log retains the findings. Independent review accepted task 530 at 1992832b with no findings. The reviewer inspected the code, ran the existing pinned native executable and read-only source oracle, verified all 21 producer hashes, both 776-file references and live artifact pins, and checked the retained browser/native counts and pre-fix failures. No compiler, browser, server or cleanup jobs ran during review. Scoped passing tests are not full equivalence.
