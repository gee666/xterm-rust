# IND and NEL callbacks

Task 466 repairs the terminal handlers for ESC D, ESC E, C1 IND and C1 NEL. The previous handlers held a mutable TerminalState borrow while BufferService.scroll synchronously notified marker and scroll listeners. Public state reads, option updates and terminal reset from those listeners could fail with a borrow error. LF already used a shared-state callback boundary.

The new index_state handler releases terminal and buffer borrows before scrolling. After observers return, it restricts the cursor on the current active buffer, which may have changed during reset. ESC and C1 forms share the same implementation. NEL resets the column before the source's initial cursor restriction. The unused borrow-holding index and scroll methods are removed.

## Source and native checks

`scripts/index-callback-oracle.mjs` bundles the untouched headless reference and executes InputHandler.parse directly. This keeps synchronous observer exceptions visible instead of passing them through the asynchronous write queue. The fixture has 24 cases, four command forms times two callback origins times three callback actions. Each captures both complete buffers and packed cells, cursor, attributes, modes, callback snapshots, final marker disposal and render ranges. Error callbacks must throw the exact supplied object. Reset callbacks are explicitly one-shot because reset can itself emit scroll notifications.

`crates/xterm-core/tests/index_callbacks.rs` replays the fixture against the real Rust public headless Terminal. It also checks that both native entry points produce the same initial buffer state and that error propagation retains the callback error's identity through the Rust error chain.

Run from an owned O: worktree:

```sh
# XTERM_SOURCE_ROOT is optional and only locates installed source build dependencies.
# It does not change which source files are compiled.
XTERM_SOURCE_ROOT=/path/to/reference node scripts/index-callback-oracle.mjs
cargo test --locked -j2 -p xterm-core --test index_callbacks
```

The oracle removes its owned tmp/index-callback-oracle output in finally, including after a failed assertion. The source input hashes and fixture SHA-256 are in results/index-callbacks/source.json. This is scoped parser/headless evidence, not browser execution, native OS matrix evidence or full equivalence proof. Overall equivalence remains NOT PROVEN.

## Execution receipts

At base main 32bf954467fe3389e9cfe42fdd8c34d425975b62, under Harry's compiler-only grant 3142:

- The unchanged source oracle passed 24 of 24 cases and then passed fixture drift verification. All 776 reference manifest entries matched in both the private physical reference and original P: reference.
- Restoring only the three old production files made the new native regression fail at case 0 with a callback Borrowed error. The runner restored the fixed files in finally and verified their exact SHA-256 values. The fixed regression passed all 24 cases in one Rust test function.
- `cargo test --locked -j2 -p xterm-core --all-targets --all-features` passed 1,559 tests across 244 successful test summaries, including empty example targets. These are Rust test-function counts, not source assertion counts. No tests were ignored or filtered.
- `cargo test --locked -j2 -p xterm-core --doc` passed two compile-fail doctests.
- `cargo clippy --locked -j2 -p xterm-core --all-targets --all-features -- -D warnings` passed.
- `cargo check --locked -j2 --workspace --all-features` and the corresponding `--lib --target wasm32-unknown-unknown` check passed. These are compile checks, not browser runtime tests.
- Exact `cargo fmt --all -- --check` failed with Windows OS error 206. The core manifest formatting check passed. This task does not claim the required full workspace quality gate passed.

Raw command logs, negative-control restoration hashes, tool versions and producer input pins are in results/index-callbacks/. The broad native test run regenerated unrelated buffer-terminal and HTML reports. Only the fresh buffer summary is retained here for context; those tracked reports were restored rather than folded into this scoped fix. Their historical main receipts are not retroactively changed.

No CI, remote push or main integration is performed by this task. Browser runtime, Linux/macOS execution and full-family acceptance remain open.
