# Clean WASM quality checks

Task 383 repairs test compilation and warning-denied WASM linting from a clean checkout. It does not change the terminal engine or any original assertion. Overall equivalence remains NOT PROVEN.

## Failure and repair

On integrated source `ff0bec8b5f9d82c0a3df64a541a9dbc2ae407524`, the full WASM target check failed even though the library-only CI check passed:

```sh
cargo check --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown -j2
```

`core_browser_service_wasm.rs` imported an ignored, absent `tmp/core-browser-source/reference.mjs`. The paired original CoreBrowserService oracle is now a tracked test fixture under `crates/xterm-web/tests/reference/`. Cargo can compile every target without Node dependencies or prior oracle-generation side effects. The fixture contains only the unchanged original component and its imported helpers. No production Rust or browser binding imports it.

The existing `scripts/prepare-core-browser-source.mjs` now regenerates that fixture and a source/output SHA-256 manifest. Its `--check` mode regenerates in memory and rejects stale bytes or provenance. The manifest records esbuild 0.28.1 and all six source/license hashes. Original source files and assertions remain untouched. Regeneration needs the original reference and its existing esbuild dependency; ordinary Cargo checks need neither. Copyright and MIT attribution accompany the fixture, and the original license hash is recorded.

The all-target compile also found two unused scrollbar-test imports. Warning-denied WASM Clippy then exposed three test-only diagnostics. The repair names the scrollbar fixture tuple and uses existing borrowed numeric/string comparisons instead of allocating comparison operands. Assertions, test discovery and input values remain unchanged. No lint suppression or new dependency was added.

The WASM CI job now installs Clippy and checks/lints all targets and all features, not just libraries. This workflow change does not itself prove a future hosted run passed.

## Observed checks

Commands ran sequentially with Cargo `-j2`, Rust 1.85.1, Node 24.15.0, wasm-bindgen 0.2.100 and the existing Playwright installation. The owned physical worktree is `O:/xterm-rust-swarm-16/tmp/Q`; TEMP, TMP and TMPDIR all point to its `tmp/`.

- Exact `cargo fmt --all -- --check` passed.
- Clean workspace WASM `cargo check --all-targets --all-features` passed.
- Workspace WASM `cargo clippy --all-targets --all-features --keep-going -- -D warnings` passed.
- Oracle regeneration and `--check` passed. A deliberately stale owned fixture caused `--check` to fail, then the original bytes were restored and verification passed.
- Retained browser assertions passed in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4: CoreBrowserService 4, scrollbars 6, Unicode public API 5, image lifecycle 13 tests per engine. Zero failed or ignored in the final runs.

The first image-lifecycle invocation mistakenly used the generic browser runner without its documented source-oracle prelude. It failed 5 of 13 cases per engine with `SourceImageRenderer is not defined`. The complete failed reports remain preserved. Running the existing documented `WASM_TEST=image_lifecycle_browser node scripts/image-canvas-check.mjs` wrapper supplied the unchanged source comparator and passed all 13 cases per engine. No Rust or source assertion changed between those attempts.

Raw compile/lint failures, successful logs, browser reports and the negative-control receipt are under `results/quality/wasm-repair383/`. Browser reports retain source/artifact hashes and exact commands. These are scoped target/component checks, not the complete original/browser/demo matrix or the final stable-release task 304.
