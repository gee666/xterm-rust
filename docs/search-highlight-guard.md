# Highlight search engine term guard

Task493 follows accepted480. Full347/322 and full-port equivalence remain NOT PROVEN.

The original SearchAddon validates a raw term before clearing highlights. Each subsequent SearchEngine.find separately checks raw `length === 0`. Rust previously converted the raw term to UTF16 before clearing decorations, omitted every highlight engine guard and retained a single matching-option snapshot across highlight calls.

The highlight path now retains the SearchTermHandle. Each engine entry captures its host, reads the existing strict-zero guard, clears the captured host on an empty guard, and skips cache, matching flags and text. Throwing guards propagate unchanged after old decoration cleanup, before selection clear or matching work. Nonempty guards initialize the cache before flags and matching text. Subsequent highlight calls repeat this order.

No dependency, unsafe code or JavaScript matching implementation is added. Ordinary engine guard480 is unchanged. Nonempty guard callbacks, including captured engine/cache ownership across reactivation or resizing, successful boxed/overridden matching methods, matching per-line short circuits and full SearchAddon suites remain347/322.

## Tests and scope

Three native functions expand18 scenarios over real terminal, selection, decorations and virtual clock services. They cover empty/error guards, reactivation, repeated guard calls after one or two matches, exact enum error cutoff and cache initialization before option errors. The retained59 functions are unchanged.

The shared fixture has84 directed cases inside one WASM test function per engine. It covers both search directions, positive/negative zero, activation during guard, six exact thrown identities, late highlight eligibility/argument failures, twelve nonnumeric strict guards stopped by an intentional regex getter sentinel and primitive hit/miss controls. These cases do not claim raw matching equivalence after that sentinel. Repeated raw-guard calls after actual matching are currently native-only because successful browser raw matching remains unimplemented.

`results/search-highlight-guard/inheritance.json` verifies all43 immutable480 archive pins and both776-file reference manifests. Four inherited files are intentionally modified by493; verification uses immutable Git objects rather than claiming unchanged working bytes.

Observed native Windows results so far:

- All62 native search functions passed, none failed, ignored or filtered. The initial new test had a host-field typo; its compile diagnostic is retained and the test was corrected before these passes.
- Replacing only `search/find.rs` with accepted480 bytes failed all three new native functions, with59 filtered. A `finally` restored the exact fixed SHA-256, followed by62 passes.
- Scoped native library/search-test Clippy and WASM library/search-test Clippy passed with `-D warnings`.
- Changed Rust formatting passed. Exact `cargo fmt --all -- --check` failed with Windows OS206; it is not a full formatting pass.
- Accepted495 HTTP cleanup is an explicit private runner prerequisite at1c1fbbce, originalf08773fa, approved409/3623. Its seven Node tests passed. It is excluded from493 producer pickup.

The shared84 cases passed against unchanged original source classes in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4,252 cases total. Actual Rust/WASM passed19 test functions in each engine,57 total, including all84 cases inside one function per engine. They are not84 additional functions.

Replacing only `search/find.rs` with accepted480 bytes made the new WASM guard function fail in Chromium while the retained18 functions passed. The `finally` restored the exact fixed SHA-256 before all19 functions passed in every engine. Both swapped variants require actual Cargo `fresh=false`; their WASM hashes differ, and both compiler artifacts contain the exact shared fixture bytes. Compiler/WASM/binding and native executable hashes are retained for independent review. Temporary binaries are provenance, not distribution deliverables.

Full workspace Clippy/tests, unchanged original Search E2E, seeded corpus, demo, GC and native Linux/macOS were not run for493. Scoped proof does not establish full SearchAddon or full-port parity.

## Reproduce

With the matching external wasm-bindgen0.2.100 CLI and installed Playwright engines, use an owned private O: target and explicit O: TEMP/TMP. Set SOURCE_DEPENDENCIES and PLAYWRIGHT_MODULE for the external original dependency tree rather than installing it into the physical reference copy.

```sh
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
SEARCH_CASES=search-highlight-guard node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
```

Control programs, exact commands, raw logs, restorations, source hashes and count summaries are under `results/search-highlight-guard/`. Source grant409/3642 triggered after497 actualrelease3714. Native compiler and source/browser grants were released after the corresponding commands ended. Main and the independent original reference remain untouched.

Isolated review accepted frozen59ea6155 with no defects. The reviewer independently hash-checked and reran the native executable,62 passes, and verified all53 then-current pins, all43 inherited480 pins, both776-file manifests, both compiler/WASM/binding sets, exact embedded fixture, fresh=false, distinct hashes and exact restores. It inspected retained browser/Clippy/fmt evidence but did not rerun heavy jobs. The later review receipt adds one pin. Actual cleanup is separate from that verdict.
