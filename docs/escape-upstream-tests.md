# Original escape parser assertion migration

Task 90 owns the synchronous assertion migration from `source_repo/src/common/parser/EscapeSequenceParser.test.ts`. Task 26 owns the Rust parser, randomized/full-state differential comparison and the separate 14-case asynchronous part of the original suite.

The checked reference contains 161 original `it` cases. The original official unit runner passed all 161 according to `results/upstream-baseline/summary.json`. That is JavaScript baseline evidence, not Rust coverage.

`results/escape-upstream/mapping.json` records all 161 source case names and line numbers. The TypeScript compiler API locates each original test and its static assertion sites without interpreting strings as source. The source hash is pinned so an upstream edit cannot silently shift the mapping's section boundaries. There are 268 direct static `assert.*` call sites in the whole original file and 60 calls to the assertion helpers `testTerminal.compare` or `test`. The synchronous portion has 147 cases, 224 direct static assertion sites and all 60 helper-expectation sites. These are static sites, not dynamically executed assertion counts. Original loops execute many sites repeatedly.

All 147 synchronous case identities map to Rust tests in `tests/escape_upstream_support/`. The native suite has 43 test functions. Related state ranges and the five command families share loops and setup instead of copying each original test body. The case map points to the exact Rust function so a reviewer can compare every original expected result. The 14 asynchronous cases are explicitly marked `async-owned-separately`, not counted as passing Rust tests by this migration.

## Layout and assertions

- `states.rs` covers construction, reset, executable/printable rules, global cancellation, entry clearing, ESC dispatch and ignore rules.
- `parameters.rs` covers collect ranges, decimal/colon/semicolon parameters, prefixes, CSI/DCS ignore entry, CSI dispatch and ignore exit, including the additional DCS intermediate-space assertion.
- `strings.rs` covers DCS/APC start/put/end, OSC raw put/ignore behavior through the actual OSC dispatcher, and the original Unicode/error coverage cases.
- `registrations.rs` covers true/false bubbling for all five command families, newest-first order, original `[3, 2, 1]` order assertions, disposal and repeated disposal, and CSI/OSC clear behavior. Payloads and parameter arrays are compared, not merely callback counts.
- `basic_handlers.rs` retains PRINT/EXECUTE/ESC set/clear behavior, raw DCS/APC split payload callbacks and ERROR diagnostics/clear behavior.
- `identifiers.rs` retains every original valid byte range, exact source error text, prefix/intermediate/final packing order and ESC/CSI/DCS/APC invocation/disposal cases.
- `examples.rs` is generated directly from all 21 original escape-sequence examples. Its 23 parse steps retain all 69 literal expected events, including complete/split DCS and APC, C1 variants, print/executable ordering, OSC success/abort, swallowed ST, error recovery and colon subparameters. This generated upstream fixture exceeds 350 lines because rustfmt expands the original expectation arrays. It is data-shaped test code, not production logic.

`escape-example-fixtures.mjs` captures the original example inputs and literal expectations. It does not run a substitute terminal, alter an assertion, or derive expectations from Rust. Normal verification checks the committed fixture exactly. Explicit `--generate` regenerates it after review. Actual original assertions run through the separately maintained official baseline runner.

## Deliberate test-representation changes

The source exposes protected fields through a test subclass. Rust's typed state cannot store the private invalid integer 124 used by the original reset fixture. The Rust reset assertion instead starts from a parsed non-default state with parameters 123 and collect `#`, then preserves the original expected Ground/zero/empty result. Arbitrary invalid private-state mutation is not a public Rust contract. Independent review found that this initially omitted the source mock's OSC reset assertion. The corrected test starts a real `OSC 0;#` payload, resets, verifies that ending the old dispatcher produces no event, then completes `OSC 1;after` and asserts that no stale `#` survives.

The original raw OSC routing tests replace the OSC parser with a permissive mock. The Rust parser owns a concrete real OSC implementation. Rust initializes a valid `OSC 0;` payload before applying each original put/ignore input and compares the same raw payload expectation. This exercises actual dispatcher integration instead of replacing it with a fake. It does not claim to expose the JS mock's private payload field as a production API.

DCS/APC raw callback arrays contain constant placeholder strings and a constant zero. Rust uses typed event variants for those labels while preserving every payload, parameter, callback order and success expectation. The ERROR callback's state-object return with `abort: false` becomes `false`; its diagnostic position, code, state, collect and parameters remain asserted. These adaptations are listed in the machine-readable mapping rather than hidden in a suite count.

## Commands and observed results

Run from the repository root:

```text
node scripts/escape-example-fixtures.mjs
node scripts/escape-test-mapping.mjs
cargo test -p xterm-core --test escape_upstream
cargo test -p xterm-core --release --test escape_upstream
cargo clippy -p xterm-core --test escape_upstream -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/tests/escape_upstream.rs
```

Node 24 or later is required for the TypeScript transformation. Mapping uses the TypeScript dependency installed by the original `source_repo` package setup. Fixture formatting uses rustfmt from the normal Cargo home, or the explicit `RUSTFMT` environment variable. Node subprocesses have timeouts and no extra build directories are created.

Observed on native Windows with Rust 1.85.1 and Node v24.15.0: all 43 native tests pass in debug and release, focused Clippy passes with `-D warnings`, fixture checking and case mapping pass. Exact outputs are in `results/escape-upstream/{debug,release,clippy}.txt`; `examples.json` records source hash and literal fixture counts. The build honors repository `.cargo/config.toml` with dev/test debug information disabled and incremental compilation off. No assertions are disabled.

Earlier attempts exposed concurrent core warnings in queue/buffer/terminal/public adapter code. Those were reported to the owners and fixed before the final focused Clippy run. The passing result is module-scoped, not a claim that the entire workspace, browser or native Linux/macOS gates passed. Full terminal equivalence remains NOT PROVEN while the remaining project criteria are incomplete.
