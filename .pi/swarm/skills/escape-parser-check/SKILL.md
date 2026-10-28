---
name: escape-parser-check
description: Use when validating the outer escape parser, regenerating full-state upstream traces, checking async assertion mapping or reviewing parser continuation and registration changes.
---

# Escape parser checks

Run from repository root with Node 24, Rust 1.85.1 or newer, rustfmt and Clippy in PATH. The unchanged reference must have its npm development dependencies installed. The oracle bundles directly from TypeScript using the reference's esbuild dependency; it never delegates Rust parsing to JavaScript.

Check available disk space before builds. Use the existing shared Cargo target and `.cargo/config.toml`; never create a second target tree to bypass another worker's lock. The generic native-quality skill provides resource-aware full-workspace execution. For parser-only command logs, before/after source hashes and disk monitoring, run `node scripts/escape-check.mjs results/escape-parser/NEW-ATTEMPT`. The destination must not exist.

```sh
node scripts/escape-parser-oracle.mjs
node scripts/escape-async-mapping.mjs
node scripts/escape-example-fixtures.mjs
node scripts/escape-test-mapping.mjs
cargo test -p xterm-core --test escape_differential --test escape_async --test escape_edges --test escape_upstream
cargo clippy -p xterm-core --test escape_differential --test escape_async --test escape_edges --test escape_upstream -- -D warnings
```

Use `node scripts/escape-parser-oracle.mjs --generate` only after an intentional source-reference change and independent review of fixture differences. The committed fixture contains 3,334 traces and 92,668 parse calls with seed `0xe5ca9e01`. It compares full state/collect/parameters/subparameters/join state and every ordered fallback/error callback after each operation. It includes each state/class combination and randomized complete/split streams.

To rerun the original suite after generating the upstream esbuild unit outputs:

```sh
node source_repo/bin/test_unit.js out-esbuild/common/parser/EscapeSequenceParser.test.js --reporter=json
```

Expect 161 original tests with zero failed/pending. Exact names remain in the unchanged source and the raw baseline JSON. `scripts/escape-test-mapping.mjs` accounts for 147 synchronous cases; `scripts/escape-async-mapping.mjs` accounts for 14 async cases. A mapping row is not itself execution proof. Run the actual Rust tests and inspect the assertions, especially pause positions, disposal and true/false bubbling.

Dedicated protocol integration checks are owned separately: `escape_osc`, `escape_dcs` and `apc_escape`. Follow their scripts/docs for fixtures and executor cleanup turns. Full terminal/browser/native-platform equivalence is not established by these scoped tests.
