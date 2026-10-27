---
name: native-quality
description: Use when running workspace quality gates, verifying native platform provenance, retaining CI artifacts or diagnosing resource-limited builds.
---

# Native quality gates

Run from repository root with Rust, rustfmt, Clippy, Git and Node 24 in PATH. No new runtime dependencies are required. The scripts use Node built-ins for shell-free subprocesses, source hashes and disk observations.

- `node --test scripts/quality-process.test.mjs scripts/quality-policy.test.mjs scripts/quality-snapshot.test.mjs`
- `node scripts/quality-gates.mjs --out results/quality/NEW-ATTEMPT`
- Install the supported WASM target, then add `--wasm-check` for compilation-only evidence.

Every output directory must be new. Defaults include an attempt timestamp. The runner rejects existing directories, retains failed command output, uses finite subprocess deadlines and records exact per-suite Cargo result counts. It checks disk before and after every command and samples during execution. The default reserve is 1024 MiB, configurable with `--min-free-mib` but never below 256 MiB. On low space it stops its process tree and records the observation, without deleting anyone's files.

Honor `.cargo/config.toml`. Do not override debug/test profiles, assertions, release settings or create duplicate target directories. Native commands use the shared Cargo target directory. The normal target-specific WASM build is separate because it has a different ABI.

Source hashes include crate fixtures/glue, scripts/tooling, workflows and build configuration. A changed source hash invalidates one-snapshot gate success. Lexical unsafe matches are only candidates for independent semantic review. Miri availability is not an executed Miri test. Native library passes are not browser/demo/addon equivalence.

`.github/workflows/rust.yml` defines six native OS/toolchain jobs and a WASM compilation check. A workflow alone is not runtime evidence. After an authorized maintainer pushes a complete committed snapshot, observe CI through:

- `node scripts/github-quality.mjs runs`
- `node scripts/github-quality.mjs dispatch REF`
- `node scripts/github-quality.mjs status RUN_ID`
- `node scripts/github-quality.mjs artifacts RUN_ID`

The GitHub script uses private environment tokens or Git's existing credential helper. It never prints credentials or stores them in artifacts. Retrieved run/job receipts and artifact ZIPs go under `results/quality/ci-RUN_ID/`. Verify their source SHA and completed job conclusions before claiming an OS passed. See `docs/acceptance/quality.md` for the acceptance and negative-case matrix.
