# Native quality checkpoint at 3aa04692

Overall equivalence remains NOT PROVEN. Task 444 observed immutable integrated source `3aa04692a409bc147670d7c61a9e2ac37b5ea9f8`, not later producers or final task 304. Run [42090157228](https://github.com/gee666/xterm-rust/actions/runs/42090157228) completed with **failure**. All six native jobs and focused Miri passed; all-target WASM compilation passed, but its warning-denied Clippy failed.

## Source and execution

The coordinator authorized the checkpoint in post 409 comment 2795 after task 438's canonical gate, independent acceptance and ref/compiler release. A normal push created unique ref `ci/saadia444-3aa04692` at exactly the accepted SHA. No duplicate dispatch or moving-main run was used. Launch/status receipts are under `results/quality/saadia444-launch/`.

All eight original artifact ZIPs and all eight completed job logs are retained under `results/quality/ci-42090157228/`. `observed-artifacts.json` binds source, platform, host, actual tool versions, command statuses and per-harness counts. All 116 ZIP-entry CRCs passed, and raw Cargo log counts match the uploaded summaries. `SHA256SUMS` binds the ZIPs, logs and derived observations. Windows and POSIX retain distinct checkout hashes, so cross-platform bytes are not assumed identical. Each job records its own unchanged before/after source hash and the same exact Git SHA. Recorded initial Git status shows only newly generated results, not tracked source changes.

## Observed native results

| Executing OS and architecture | Rust 1.85.1 | Actual stable Rust 1.99.1 |
| --- | --- | --- |
| Windows x64, x86_64-pc-windows-msvc | Passed | Passed |
| Linux x64, x86_64-unknown-linux-gnu | Passed | Passed |
| macOS arm64, aarch64-apple-darwin | Passed | Passed |

Each of the six native jobs passed the exact full cargo-fmt gate, all-target/all-feature warning-denied Clippy, no-default Clippy, all-target/all-feature release build and every required test configuration. Every job reported:

- All-feature, default and no-default configurations each passed 2,094 functions across 444 summaries, with zero failed, ignored or filtered-out cases.
- All-feature doctests passed two functions across five summaries.
- The separately inventoried `xterm-web/acceptance-hooks` feature passed 360 functions across 148 summaries with defaults disabled.

These repeated configurations are not distinct-function totals and must not be summed as unique source coverage. Native browser-target harnesses can contain zero executed tests. Their compilation does not establish browser runtime behavior. Raw commands, durations, tool output and source stability are in the original ZIPs.

## Hosted Windows full formatting

Both Windows jobs ran at physical `D:/a/_temp/q`. Their actual metadata inventory contained 444 edition-2021 roots and 28,872 quoted file-argument characters including the canonical-prefix allowance. Both exact `cargo fmt --all -- --check` commands passed. The 55 Node verifier cases passed without failure or skip, and both owned-worktree cleanup steps completed successfully after upload.

This is observed hosted validation of task 433's physical-path repair, not a configuration-only claim. Historical local OS206 failures and direct-root/package checks remain preserved with their original source scope in [the Windows path-budget report](quality-windows-path-budget.md). This checkpoint does not claim that later inventories or longer local paths fit Windows' process limit.

## Miri and WASM outcomes

Focused Miri passed 21 functions across two summaries on Linux using actual `rustc 1.101.0-nightly`, commit `2d20b42b8` dated 2026-11-04, and Miri 0.1.0 from that toolchain. This covers the existing lifecycle and cell suites only. The six stable/MSRV native jobs' optional Miri-availability probes failed as expected for toolchains without Miri; the separate executed Miri job supplies the scoped result. No full-workspace sanitizer or browser safety result is inferred.

The WASM job used actual stable Rust 1.99.1. Its full workspace locked all-target/all-feature check passed. The subsequent warning-denied Clippy failed on `crates/xterm-web/tests/wasm_webgl_atlas_page.rs:217`, where `&[page.clone()]` triggered `cloned_ref_to_slice_refs`. Original diagnostics and the failed job conclusion remain intact. No retry or assertion change turned this run green.

[Task 452's accepted repair](quality-atlas-page-lint452.md) replaces that one test argument with `std::slice::from_ref(&page)`. It has scoped MSRV WASM lint evidence and independent source review, but repaired actual-stable integrated CI remains unrun here. The repair is outside this immutable failed checkpoint. Later integration task 448 and all other later producer changes require their own source-bound checks.

## Reproduction and remaining work

The workflow's native command was:

```sh
node scripts/quality-gates.mjs --out results/quality/OS-TOOLCHAIN --timeout-ms 600000
```

The WASM command pair was:

```sh
cargo check --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown
cargo clippy --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown -- -D warnings
```

Use `scripts/github-quality.mjs status|artifacts|logs 42090157228` for authenticated read-only retrieval without exposing credentials. The retained artifact audit can be rerun with Node and an existing unzip executable, with `XTERM_UNZIP` set to its absolute path. It performs no compiler or browser work.

Final task 304 remains open for the stabilized integrated release revision. Complete unchanged original suites, Rust-backed Chromium/Firefox/WebKit matrices, demos, source/API/assertion closure and matched comparisons remain separate obligations. Seven successful jobs do not erase the WASM lint failure or establish full functionality.
