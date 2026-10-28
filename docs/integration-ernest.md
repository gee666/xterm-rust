# Native and browser workspace checkpoint

Task60 integrated the inspected native producers from Lydia3491c77 with Xavierb956eeb's coherent browser Root on mainb6ed3ff. Selection fixture task273 was independently accepted before integration. The original producer history remains at `swarm/Ernest-stacked-checkpoint`3a4bace, and all original producer branches remain intact. The dirty primary master, source reference and AGENTS.md were untouched.

The canonical helper initially flattened merged histories during rebase and stopped at an old WIP commit. That rebase was aborted. The combined snapshot was squashed onto the explicit original561b705 parent, then rebased onto concurrent main1e62bc5. This preserves Sandra's newly integrated E2E scripts and results. A first gate correctly refused its fast-forward because main advanced during testing. The final unchanged canonical gate passed and fast-forwarded main to b6ed3ff, then removed only Ernest's regular private target.

The three merge conflicts were additive native/browser exports in lib.rs and scroll/mod.rs, plus two equivalent String formatting repairs in theme/color_set.rs. The integration adds no target feature or profile override. Dev/test artifact policy and release settings remain unchanged. Disk checks reported27GiB free on C and1.1GiB on P after cleanup.

The only new runtime logic for this integration fixes the default-off acceptance snapshot after Marker::line was removed. Snapshot construction now captures authoritative MarkerLine primitives and restores them after JSON parsing and after all engine borrows are released. JSON alone cannot preserve nonfinite Numbers, negative zero or unpaired UTF-16 units. The actual-engine regression verifies Number/String distinction, -0, NaN, both infinities, a Number beyond2^53 and unpaired surrogates in both buffers. It does not add a numeric shadow or claim public marker bindings are finished.

## Executed checks

- Canonical `node scripts/worktree-merge.mjs --main main`: PASS. Builds and all-target tests of xterm-core, xterm-addons, xterm-wasm-support and xterm-web, dependent all-target checks, native workspace check, and affected WASM library checks. Native result1934 passed,0 failed,0 ignored across332 test targets, including zero-test examples and target-gated suites. Those zero-test WASM targets are not browser coverage.
- `cargo check --locked -p xterm-web --target wasm32-unknown-unknown --features acceptance-hooks`: PASS without warnings.
- `WASM_TEST=lib CARGO_FEATURES=acceptance-hooks node scripts/browser-host-check.mjs`: PASS6 Rust assertions each in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. This includes the new actual-engine marker snapshot regression and five retained accessibility callback tests.
- Earlier native `cargo test --workspace --all-targets --all-features` on the Lydia+fixture checkpoint: PASS. The final canonical gate uses the actual helper's default-feature commands, not an invented all-feature pass for the later combined Root.
- Selection task273 retained all26 unchanged source tests,4,297 current source cases and the restored historical case.31 native test functions pass debug and release. See `results/selection-service/fixture-repair.json`.

Raw canonical output and browser report/logs are in `results/integration/ernest/`. Browser test compiler artifacts remain in the private untracked run directory. No background browser or server remains.

## Independent review

The clean task60 reviewer accepted this integration checkpoint. It independently passed locked native all-target/all-feature tests, doctests, all-feature release build, no-default check, the acceptance-hooks WASM check, all six library tests in each browser and six supporting JavaScript tests. It found no integration-specific correctness blocker. It also reproduced existing core-test Clippy findings and producer formatting differences, with workspace rustfmt still hitting the Windows command-length limit. Those global quality checks remain open. Later mainacee593 includes Leonard's separately reviewed core lint repairs and Windows integration.

## Limits

Full xterm.js equivalence is NOT PROVEN. Browser/API/render/input/addon behavior remains governed by its existing feature tasks, including31/32/193. Native Linux/macOS runtime, final feature/MSRV/release/format/Clippy/safety checks remain task47 and the final audit. Existing theme format lint was repaired by the included producer, but no full workspace Clippy pass is claimed. Separate accepted producer branches such as Athena's mutable options and Aristides's raw-font/cache work still need their own tested integration. Compilation and these18 browser assertion executions are not a substitute for original complete browser suites or E2E.
