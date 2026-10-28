---
name: decoration-oracle
description: Use when changing DecorationService registration, lookup, marker lifecycle, colors or deferred line-cache maintenance.
---

# Decoration oracle

Run from repository root with Node24.15 or newer, reference dependencies installed, and Rust1.85 or newer. Supply explicit command timeouts. Check free disk space before builds, preserve shared .cargo/config.toml and do not create alternate target directories.

```sh
node scripts/decoration-oracle.mjs
node scripts/decoration-raw-oracle.mjs
cargo test -p xterm-core --test decoration_raw --test decoration_precision --test decoration_review_edges
cargo test -p xterm-core --test decoration_upstream --test decoration_differential --test decoration_edges
cargo test -p xterm-core --release --test decoration_upstream --test decoration_differential --test decoration_edges
cargo clippy -p xterm-core --test decoration_upstream --test decoration_differential --test decoration_edges -- -D warnings
```

Default execution verifies deterministic fixture bytes without rewriting them. Use `node scripts/decoration-oracle.mjs --generate` explicitly to regenerate expected fixtures. A mismatch writes actual generated rows to tmp/decoration-oracle/actual.tsv and fails.

The oracle bundles read-only original sources through the installed upstream esbuild and executes every original DecorationService.test.ts assertion unchanged under actual Mocha. It records ten original outcomes, source hashes, Node/platform metadata and fixture hashes in results/decoration/oracle.json. Temporary generated bundles are confined to tmp/decoration-oracle.

Seed0xdec0a710 generates1024 ordered operations from the actual JavaScript service and buffer marker implementation. It compares every retained marker/decoration disposal state, sorted decoration sequence and240 ordered cell/layer lookups after every operation. Insert/delete actions wait for a real JavaScript microtask boundary. The Rust replay flushes the explicit portable microtask queue at the same boundary. Operations register zero/multi-height decorations, dispose, trim, insert, delete, reset and mutate layers. Never edit fixture expected states by hand to hide a mismatch.

The raw oracle adds81 directed/random Number/String cases,405 snapshots and118 ordered queries per snapshot, using seed0xdec064ab and the actual Buffer listeners. It verifies `decoration-raw.tsv`; `--generate` is required to rewrite the fixture. Set `XTERM_SOURCE_ROOT` to the supplied reference directory if dependencies are outside the worktree. Source non-progress spans are documented separately, not counted as passing observations. Run `decoration_raw` to replay these snapshots and test key identity, NaN merges and native non-progress errors.

Read docs/decoration-acceptance.md for complete branch requirements. Passing portable tests does not establish browser microtask/DOM rendering or full terminal equivalence. Preserve failed logs and report shared compile failures separately.
