---
name: core-service-check
description: Use when changing CoreService modes/input events or LogService thresholds, routing and lazy values, or reviewing their upstream trace evidence.
---

# Core and log service checks

Run from the repository root with Node 24.15+, Rust 1.85+ and `source_repo` present:

```sh
node scripts/check-core-services.mjs
cargo test -p xterm-core --test core_service --test log_service
cargo clippy -p xterm-core --lib --test core_service --test log_service --example core_service_oracle -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/core_service.rs crates/xterm-core/src/log_service.rs crates/xterm-core/tests/core_service.rs crates/xterm-core/tests/log_service.rs crates/xterm-core/examples/core_service_oracle.rs
```

The script transforms original TypeScript in memory. It removes import/export syntax, erases types, and removes the three constructor dependency decorators. Original service implementation and all three CoreService test bodies run unchanged. Constructor-only source tests consume equivalent minimal mocks with the same option defaults. See the exact mock limitation in `results/core-services/summary.json`. There is no source LogService test suite.

The Rust example implements a TSV protocol. The script compares ordered data, binary, scroll, user-input, console/custom logs and complete core mode/cursor/Kitty snapshots after each command. Seed `0x434f5245` adds 20,000 operations after boolean/viewport/UTF-16/log-threshold matrices. Live option coercion errors and disposal are checked too. On mismatch it retains both a counterexample and the full failing operation prefix under `results/core-services/`.

The summary records source and implementation hashes, seed, commands, versions, platform and operation counts. Native Windows results do not prove Linux/macOS/browser execution. Terminal integration is separate task 68; utility results cannot close that requirement. The native console adapter must implement `LogSink`; custom loggers go in `OptionValue::object(CustomLogger(...))`. Use weak references in self-retaining callbacks.

Observe `.cargo/config.toml` disk-saving debug/test settings. Do not override profiles or create separate target trees. Check disk space before broad builds. Temporary files belong under root `tmp/`.
