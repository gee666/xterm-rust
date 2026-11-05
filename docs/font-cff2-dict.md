# CFF2 top dictionary operand reads

Task492 adds the missing CFF2 top dictionary lexical phase after accepted472's initial INDEX reads. It validates escaped operators, two/four-byte integer operands, compact operands and nibble-terminated real operands. Invalid bytes31 and255 produce the source's exact `Error` message. The source completes lexical reads before interpreting dictionary entries or rejecting duplicates. The Rust code deliberately preserves that order.

CFF2 takes its dictionary start from the header size and its length from the header's unsigned16-bit topDictLength. The source first reads the global subroutine INDEX, then copies the dictionary into a distinct DataView. An operand cannot consume bytes after that dictionary, even if those bytes exist in the font. Compressed reads reuse472's bounded-window inflater and retain at most the protocol-limited65535 dictionary bytes, never an untrusted whole WOFF origLength.

## Evidence and commands

Run from the producer worktree, with an explicit resource grant, a private O: target and Cargo-j2. Use `SOURCE_PACKAGE_JSON` and `PLAYWRIGHT_MODULE` to reuse installed read-only reference dependencies. Use matching wasm-bindgen0.2.100.

```sh
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-dict-oracle.mjs
cargo test -j2 -p xterm-addons --all-targets --all-features
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_cff2_dict_browser -- -D warnings
node scripts/font-cff2-dict-negative.mjs
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-dict-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff2_dict_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs
```

The read-only Node oracle currently passes1208 unchanged full constructors. It captures4 unchanged CFF1 controls,1100 invalid operands,100 bounds failures and4 earlier missing-name failures. Of the bounds cases,16 fail in the earlier global INDEX and84 fail on dictionary operands. Cases cover both CFF directory tags, SFNT/WOFF, every possible first byte, integer and real payload bytes that resemble invalid operands, escape consumption, global-before-dictionary, name-before-dictionary, dictionary-before-hmtx, duplicate-before-later-lexical-failure and32/64KiB prefix-crossing reads. The source, font, fixture and reconstructed input hashes are retained. Native and browser tests reconstruct all complete font bytes independently and compare them before loading the real Rust font implementation.

The actual addon test uses the existing source-host lifecycle scenario. It checks failed-load caching, recovery on a different family, refresh counts and the exact previous invalid31 Error. Both invalid31 and255 are also checked through full Rust/WASM constructors. It does not pretend that a browser helper is the source terminal engine.

Final source68f8db11 passes198 native test functions across44 harnesses in `cargo test -p xterm-addons --all-targets --all-features`, with no failures or ignored cases. Native and WASM test Clippy deny warnings. Native workspace library checks and WASM workspace all-feature library Clippy also pass. The initial native import failure is retained; fixing the nested module import resolves it. Five individual package format checks pass. The exact full-workspace fmt command still fails with Windows OS error206, retained separately rather than reported green.

The initial full-corpus negative bypass exceeded its180s bound because bypassing errors reaches expensive complete valid-font construction repeatedly. Its error and restoration receipt remain. An additive directed28-case test provides a bounded negative control without changing the original1208-case assertion. Bypassing the lexical hook gives28 expected failures, including31,255 and operand bounds; exact source-byte restoration then passes. Full1208 positive coverage remains unchanged and passes again in the final native and browser runs.

Unchanged source constructors1208 plus4 actual addon loads pass in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The real Rust/WASM suite's2 test functions pass in each engine, including the1208-case constructor corpus, independent byte reconstruction,28 directed cases,520 existing header cases and actual addon lifecycle. Raw exact `Error: Invalid b0 31` survives the actual addon diagnostic path in every engine. WASM durations were20.68s,261.23s and29.76s. `results/font-cff2-dict/rust-browser.json` retains the compiler artifact pin and raw outputs. Source and actual browser logs are separate; no actual error message was dropped or normalized to obtain equality.

The accepted495 artifact-server fix is a distinct private runner prerequisite at7c00c646. Its7 Node tests pass; it closes only its own clients after the body settles. No495 changes belong to492's producer contribution. All compiler/browser/server grants were released after execution. `results/font-cff2-dict/preflight.json` verifies all776 manifest files against the original and physical reference copy. All temporary paths are confined to O:/xterm-rust-swarm-16/tmp/worktrees/ShiminCffDict. Cleanup remains pending until an explicit absence/unregistration receipt.

## Limits

This is not a complete CFF2 dictionary implementation. Dictionary values, duplicate-key interpretation, FDArray/FDSelect, private dictionaries, CFF1 dictionaries and charstrings remain task342. The existing Bounds and missing-name browser error wording differences also remain. Nothing here claims to close342, the full original suites, native OS matrices or overall equivalence. Overall status remains NOT PROVEN.

The only private prerequisite is4795d345, a byte-pinned copy of472's sixteen accepted source files. Do not replay it as a new producer contribution or pick472's earlier private442/453/457 history. Integration must preserve the accepted font union and FontError split already on main.
