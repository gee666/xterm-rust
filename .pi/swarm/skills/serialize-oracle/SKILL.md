---
name: serialize-oracle
description: Use when checking ANSI/HTML serialization against the unchanged source, regenerating exact UTF-16 buffer fixtures, or reviewing serializer output and error paths.
---

# Serializer oracle

Run from the repository root after installing `source_repo` dependencies:

```sh
node scripts/serialize-oracle.mjs
cargo test -p xterm-core --test serialize_differential
cargo test -p xterm-core --release --test serialize_differential
cargo clippy -p xterm-core --lib --test serialize_differential -- -D warnings
```

Use the shared Cargo target and profiles. Check disk before builds. The generator checks at least 256 MiB free and places bundles under `tmp/serialize-oracle/`. It bundles unchanged source/assertion bodies with esbuild and runs the original suite with Mocha. An observing subclass preserves each source method's return/throw behavior while recording input snapshots. Chai call wrappers count assertions without changing their behavior.

Expected baseline is 44 passing cases and 60 observed assertions. There are 3,350 exact UTF-16 outputs, including 44 original snapshots, 1,242 directed outputs, 2,048 seeded outputs and 16 raw-header/number cases. Seed is `0x05e21a11`. Counts are asserted in Rust. The fixture stores full raw cells, normal/alternate buffers, attributes, modes, ranges, selections and theme options. Missing palettes exercise the headless default.

See `docs/serialize-oracle.md` for scope and encoding. `results/serialize/oracle.json` records source hashes, fixture hash, versions, outcomes and index-to-source test names. Differential TSVs retain every actual/expected output and failing ID. Do not alter expected outputs to accommodate a mismatch. Source `Invalid array length` errors are expected results, not skipped tests.

This verifies serializer functions. Rust engine round-trips, live addon lifecycle and browser/WASM integration remain separate task 88 checks. Native platform results and configured CI must not be conflated. Run `node scripts/check-oracle-startup.mjs --serialize` for isolated missing-dependency and blocked-temp-directory receipt checks. Require the actual zero exit status and matching current fixture hash, not an old passing report alone.
