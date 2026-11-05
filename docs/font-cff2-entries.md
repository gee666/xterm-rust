# CFF2 dictionary duplicate entries

Task505 follows accepted492's operand reads. It adds the missing `entriesToObject` error for repeated dictionary operators, including unknown and escaped operators. The error is `Object [object Object] already has key N`. The prior value determines whether a duplicate is rejected. An empty operand array converts to zero, one operand keeps its numeric value, and multiple operands convert to NaN. A prior NaN permits replacement. A later numeric replacement can make a subsequent occurrence fail.

Operand reads still finish before the duplicate error can escape. The first duplicate is retained while parsing continues. Later invalid operands or truncated reads take precedence. The global INDEX remains earlier than the dictionary, missing-name rejection remains earlier than CFF parsing, and hmtx remains later. The change does not interpret FDArray, private dictionaries, charstrings or CFF1 entries.

The parser decodes the source's unsigned short and signed long operands, compact integers and real nibble strings. Real values use JavaScript parseFloat prefix rules, including malformed suffixes, incomplete exponents, NaN, signed zero and overflow. The unescaped CFF2 blend operator pops its count and retains the first operands while their length exceeds that count. Escaped23 is an ordinary dictionary key. Work and storage remain bounded by the unsigned16-bit topDictLength. Every operand is pushed and popped at most once.

## Safety divergence

A negative blend count causes the reference `while (operands.length > numberOfBlends) operands.pop()` to loop forever after the stack empties. Rust rejects this with `CFF2 blend count cannot be negative`. It never emulates that hang. The corpus deliberately does not execute such an input against source constructors. Native unit tests cover the bounded rejection. This is a documented divergence, not evidence of equivalence or user approval of a compatibility change. An invalid later lexical byte can still supersede this deferred Rust rejection. The source would already have hung.

## Commands and evidence

Run in an isolated O: worktree with an explicit resource grant, a private target, Cargo-j2 and the locked wasm-bindgen0.2.100. Reuse installed dependencies via `SOURCE_PACKAGE_JSON` and `PLAYWRIGHT_MODULE`.

```sh
node scripts/font-cff2-entries-oracle.mjs
cargo test -j2 -p xterm-addons --all-targets --all-features
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_cff2_entries_browser -- -D warnings
node scripts/font-cff2-entries-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff2_entries_browser WASM_TEST_TIMEOUT_MS=600000 node scripts/browser-host-check.mjs
```

The source-pinned corpus has898 full constructors, six unchanged CFF1 controls, all282 ordinary/escaped operator keys, numeric/NaN/array/blend cases, first-duplicate precedence, later lexical errors, prior name/global failures, later hmtx failures, both directory tags, SFNT/WOFF and32/64KiB dictionary reads. Seed0x505cff02 contributes64 directed random recipes. Input SHA256 and FNV32 are retained. Native and browser JS reconstruct font bytes independently before actual Rust loading. The new corpus shares the existing test-only container reconstruction rather than copying that writer. The browser lifecycle test reuses the earlier source-host scenario to check actual addon failed-load caching, recovery and exact error messages.

Final tested source8cf497cc passes898 unchanged full source constructors and4 actual source addon loads in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The actual Rust/WASM suite passes2 test functions in each engine, including the898 new cases, all1208 earlier492 cases, directed predecessor checks and header regressions. The exact duplicate Error and actual addon caching/recovery are checked in all engines. Native xterm-addons has204 passing functions across46 harnesses. The full native Windows workspace all-target/all-feature gate passes2170 functions across487 harnesses, zero failed or ignored. Strict native workspace all-target/all-feature Clippy and strict WASM font-addon all-target/all-feature Clippy pass. Native and WASM workspace library checks pass. All five individual package formatting checks pass; the exact full workspace format command still fails with Windows OS206. That is not a full formatting pass.

A bounded duplicate-bypass negative control executes16 expected native assertion failures and restores the source SHA256 exactly before the final positive replay. Initial source browser attempts exposed two test-input setup errors. Missing clean duplicate inputs and one overlong byte-array selector were corrected additively without changing assertions. Those failures remain in source-browser-initial logs. The first actual WASM run passed Chromium/WebKit, but Firefox failed while loading the module before discovering tests. A single bounded diagnostic loaded the same generated artifact successfully with no request failures, then a final full three-engine run passed. The initial failure and diagnostic remain retained, not relabeled as passing. An extra full native workspace attempt exceeded240 seconds after compiling; no surviving owned process remained. The justified warm900-second final gate passed. See the raw receipts and checks.json. Do not upgrade these results to full342, all original suites, native Linux/macOS or overall equivalence. Overall equivalence remains NOT PROVEN.

Private prerequisites a1d93b93 and2d423179 reproduce accepted492's source and tests on main5088. Privateb51eede2 reproduces accepted495's HTTP cleanup helper. They are not new505 contributions and must not be picked again. The505 source commits are1d271191 and8cf497cc, separated by the private495 copy. Only those own commits and later own receipts are eligible for integration after the accepted492 union. Do not pick a continuous range acrossb51eede2. This owner does not move main or run CI.
