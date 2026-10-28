# xterm Rust port

A Rust port of the supplied xterm.js 6.0.0 tree. Terminal state, parsing and buffers run in Rust. Browser bindings use WASM and browser APIs, not the original JavaScript terminal engine.

**Full functional equivalence is NOT PROVEN.** Implementation, integration and verification are still in progress. A working component test is not evidence that every terminal, renderer or addon contract is complete. See [the requirements](PORT_REQUIREMENTS.md), [compatibility inventory](docs/compatibility.md), [verification report](docs/verification.md) and [comparison report](docs/comparison.md).

## Prerequisites

- Rust and Cargo. The workspace declares edition2021 and Rust1.85. Observed local toolchain details are retained with individual test receipts; native Linux/macOS results require their own execution.
- Node.js24.15 or later for verification scripts and local browser hosting.
- For browser builds, `wasm32-unknown-unknown` and wasm-bindgen CLI0.2.100, matching the locked browser ABI.
- For original-source differential tests, the exact supplied `source_repo` tree and its npm dependencies. The reference was supplied privately with this checkout; no public source commit is asserted. Its provenance and hashes are in `docs/baseline/reference-manifest.json`.

From the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100 --locked
npm --prefix source_repo ci --ignore-scripts --no-audit --no-fund
```

The npm command is for oracle/browser test dependencies. It skips lifecycle scripts such as native PTY installation. The native Rust example does not need npm, JavaScript or a PTY. A browser build and static demo do not execute the reference engine. Optional attach echo tests reuse the reference's locked MIT-licensed `ws` development dependency.

Keep the shared build profiles in `.cargo/config.toml`. Do not disable assertions to clear a test. Quality/demo runners reserve1GiB of available disk space and retain failures when a build cannot safely start. During a resource pause, wait for coordinated generated-artifact cleanup rather than deleting source, fixtures, evidence or another build's outputs.

## Rust checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features
cargo test --locked --workspace --doc --all-features
cargo build --locked --workspace --release
```

These commands define required gates, not a claim that the current shared tree passes them. Feature/target combinations, original source counts, actual native CI receipts and remaining failures belong in `docs/verification.md`.

Some focused checks are:

```sh
node scripts/check-mouse-state.mjs
node scripts/terminal-modes-oracle.mjs
cargo test --locked -p xterm-core --test terminal_modes --test terminal_alt_modes --test terminal_modes_differential
node scripts/upstream-baseline.mjs
```

The mouse checker runs source assertions and source-derived corpora, then Rust tests in debug/release, formatting and warning-denying Clippy. Its latest report must match the current implementation hashes before it can establish a pass. The terminal-mode oracle checks the original20 selected assertions and the checked-in2,136 operation corpus. It is not a complete terminal differential run. `upstream-baseline.mjs` validates the reference manifest before and after running the original suites. JavaScript passes are baseline evidence, not Rust coverage.

Never regenerate expected fixtures to hide a discrepancy. Retain the failing seed, input partitions, output and command receipt.

## Native interactive example

```sh
cargo run --locked -p xterm-core --example native -- --plain
```

Omit `--plain` for ANSI-colored inspection derived from parsed Rust cells. The initial sample includes red/green attributes, Unicode and cursor movement. Try:

```text
reset
resize 8 4
write abcdefghijkl\r\n
show
resize 12 4
input hello\r
serialize
quit
```

The wrapped text should reflow from `abcdefgh` plus `ijkl` to `abcdefghijkl`. Input is observed as outbound data, not silently echoed into display cells. The default Unicode6 provider preserves the source's width policy even when the host console draws emoji differently.

Run deterministic state assertions, negative commands and EOF cleanup with:

```sh
node scripts/check-native-demo.mjs
```

See [native demo contracts and current evidence](docs/native-demo.md). The example is a line-oriented terminal inspector, not a native shell or PTY emulator. Final acceptance requires a current successful receipt, not just this command's presence.

## Rust-backed browser demo

Build the real production WASM library and minimal binding JS:

```sh
node scripts/build-demo.mjs
node scripts/serve-demo.mjs --port 3000 --timeout-ms 900000
```

Open `http://127.0.0.1:3000/demo/index.html`. The server stops after15minutes; press Ctrl+C for immediate shutdown. Initialization failures are visible, including missing WASM assets or an unimplemented terminal binding. `browser/xterm.css` is an attributed, unmodified copy of the original stylesheet and is part of the required distribution, not an optional demo decoration.

For an actual local WebSocket echo connection used by the attach addon:

```sh
node scripts/serve-demo.mjs --port 3000 --timeout-ms 900000 --websocket
```

Use `ws://127.0.0.1:3000/socket` in the demo's attach control. The server echoes bytes; it does not parse terminal sequences. Addon input travels through the real terminal data event, socket and Rust write path.

The UI has incoming display writes, separately observed outgoing input, ANSI/Unicode samples, resizing, live options/theme/font/renderer controls, search/serialization and addon load/unload/reload. All controls target the same visible Rust-backed Terminal. An unavailable addon binding reports an error rather than pretending to load. The presence of a control is not proof that its implementation or lifecycle passes. The full [demo acceptance matrix](docs/acceptance/demos.md) requires real state, rendering, selection, round-trip and teardown assertions.

Mandatory build artifact paths and hashes are recorded under `results/demos/build/`. Generated `browser/pkg/` files are not source-controlled; regenerate them with the build command. This distribution currently includes the compiled browser crate's exports and must not be described as a measured core-only download without a matched-feature size analysis.

## Browser tests and measurements

Original and Rust-backed browser checks require Playwright and its actual browser binaries:

```sh
node source_repo/node_modules/playwright/cli.js install chromium firefox webkit
node scripts/browser-host-check.mjs
```

The host checker is a Rust WASM component suite, not a full demo or terminal equivalence test. Demo cases require `scripts/demo-e2e.mjs` with real Chromium/Firefox/WebKit state/render assertions. That runner and full demo receipts remain work in progress; no missing case is counted as passed. The browser requirements include input, selection, clipboard, resizing, links, accessibility, cleanup and every applicable addon.

The full speed/artifact-size comparison runner is not yet delivered. [Comparison requirements](docs/acceptance/comparison.md) require identical validated workloads, release settings, warmup and at least10 measured repetitions with median/p95 and raw results. No speedup or size advantage is claimed. [Current comparison documentation](docs/comparison.md) distinguishes partial source counts from completed performance/size evidence.

## Architecture and licenses

- `crates/xterm-core` owns portable state, parsing, buffers, services and native APIs.
- `crates/xterm-web` owns WASM/browser bindings, DOM/input/rendering adapters and browser addon integration.
- `crates/xterm-addons` contains portable addon logic.
- `source_repo` is the read-only behavioral reference. Never weaken its assertions to obtain green results.

Public API representation changes, UTF-16 contracts, original assertion mappings and gaps are documented by subsystem. Preserve upstream notices and the [MIT license](LICENSE), plus data/dependency notices under `licenses/`. Actual independent reviews and missing checks are part of the verification deliverable, not optional follow-up.
