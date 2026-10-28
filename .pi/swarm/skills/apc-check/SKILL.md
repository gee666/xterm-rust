---
name: apc-check
description: Use when validating APC dispatch, regenerating upstream oracle traces, or checking ApcParser.test.ts assertion coverage.
---

# APC checks

Run from repository root with Node.js 24.15 or newer and Rust 1.85.1 or newer.

`node scripts/verify-apc.mjs` runs the source oracles, all APC Rust suites, formatting and Clippy through the shared bounded process runner. Each run has a separate evidence directory and starts with an incomplete false summary, so interruption cannot retain a stale green report. It records before/after input hashes and rejects changed-input runs. It uses the checked-in Cargo profile and existing target directory.

```sh
node scripts/check-apc.mjs
cargo test -p xterm-core --test apc --test apc_edges --test apc_differential --test apc_registration
cargo clippy -p xterm-core --test apc --test apc_edges --test apc_differential --test apc_registration -- -D warnings
```

Use `node scripts/check-apc.mjs --generate` only to regenerate the committed oracle fixture after an intentional source-reference change. The script erases TypeScript and replaces imports before evaluating unchanged source bodies and all 20 original assertions in a Node VM. It supports nested beforeEach/afterEach hooks and awaits async tests. It does not modify source_repo.

`results/apc/oracle.json` records exact baseline names, outcomes, Node version, source hashes, seed, trace counts and fixture hash. `crates/xterm-core/tests/fixtures/apc-oracle.tsv` holds 256 traces of 80 operations using seed `0xa9c2025`. The Rust differential test compares complete callback/fallback logs for registration, explicit disposal, clear, restart, reset, completion, idle calls, empty chunks and UTF-32 inputs including lone surrogates and supplementary characters. A failing row retains the operation trace and seed for reproduction.

The mutable-list fixture retains five source cases, covering abrupt errors in start/put/end/reset and successful index-shift reexecution. Start, put, end and reset return `Result` to preserve source errors without panics.

See `docs/apc-port.md` and `docs/criteria/apc.md` for representation differences and the mapping between 20 upstream cases and parameterized Rust assertions. For actual outer-parser integration, run `node scripts/apc-escape-oracle.mjs` and `cargo test -p xterm-core --test apc_escape`. The 280 trace corpus compares exact callbacks, state, collect, Params and join state; three retained control cases cover sync/rejected callback errors and reset during a pending end. See `docs/apc-escape.md` and `results/apc-escape`. Only intentional source-reference changes justify `--generate`.

Neither the subparser nor escape-parser checks prove terminal/browser integration or complete terminal equivalence.
