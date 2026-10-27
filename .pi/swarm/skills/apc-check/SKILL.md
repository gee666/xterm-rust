---
name: apc-check
description: Use when validating APC dispatch, regenerating upstream oracle traces, or checking ApcParser.test.ts assertion coverage.
---

# APC checks

Run from repository root with Node.js 24.15 or newer and Rust 1.85.1 or newer.

```sh
node scripts/check-apc.mjs
cargo test -p xterm-core --test apc --test apc_edges --test apc_differential
cargo clippy -p xterm-core --test apc --test apc_edges --test apc_differential -- -D warnings
```

Use `node scripts/check-apc.mjs --generate` only to regenerate the committed oracle fixture after an intentional source-reference change. The script erases TypeScript and replaces imports before evaluating unchanged source bodies and all 20 original assertions in a Node VM. It supports nested beforeEach/afterEach hooks and awaits async tests. It does not modify source_repo.

`results/apc/oracle.json` records exact baseline names, outcomes, Node version, source hashes, seed, trace counts and fixture hash. `crates/xterm-core/tests/fixtures/apc-oracle.tsv` holds 256 traces of 80 operations using seed `0xa9c2025`. The Rust differential test compares complete callback/fallback logs for registration, explicit disposal, clear, restart, reset, completion, idle calls, empty chunks and UTF-32 inputs including lone surrogates and supplementary characters. A failing row retains the operation trace and seed for reproduction.

See `docs/apc-port.md` for representation differences and the mapping between 20 upstream cases and parameterized Rust assertions. These checks do not cover the outer escape parser or prove complete terminal equivalence.
