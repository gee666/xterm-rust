---
name: charset-oracle
description: Use when regenerating charset fixtures, running the original CharsetService suite, or checking Rust designation state and VT100 maps against xterm.js.
---

# Charset oracle

From the repository root, with Node.js 24.15.0 or newer and the workspace Rust toolchain:

```sh
node scripts/charset-oracle.mjs
cargo test -p xterm-core --test charset
```

The first command preserves `source_repo` and resolves interface-only imports in temporary copies under `tmp/charset-oracle/`. It runs all four original CharsetService test bodies with Node strict assertions replacing Chai. It writes reference hashes and per-test outcomes to `results/charset/oracle.json`.

Generated fixtures cover all 16 exported charset designators and unknown `X` against all 128 ASCII inputs, giving 2,176 map rows. Another fixture records 4,107 designation/invocation/reset operations, including 4,096 seeded operations. Seed is `0x0c4a25e7`. Both fixtures are under `crates/xterm-core/tests/fixtures/charset-*.tsv`.

See `docs/charset-oracle.md` for the exact six-column state and three-column mapping formats. `-` means undefined. Alias identity uses the smallest lexical designator. Keep complete multi-unit replacements such as Dutch `ij`.

Never edit expected values to hide disagreements. Preserve a counterexample and fix the Rust implementation or demonstrate an oracle error.
