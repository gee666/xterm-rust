# Independent WriteBuffer review

Blake reviewed task 81 against the complete original `common/input/WriteBuffer.ts`, the 19-case source suite, the Rust queue/drain/types modules, source-case mapping, controlled scheduler tests, error bridge, external/resizable byte-view tests and deterministic oracle.

Independent commands on Windows:

```text
node scripts/write-buffer-oracle.mjs
cargo test -p xterm-core --test write_buffer
cargo clippy -p xterm-core --test write_buffer -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/input/write_buffer.rs crates/xterm-core/src/input/write_buffer_drain.rs crates/xterm-core/src/input/write_buffer_types.rs crates/xterm-core/tests/write_buffer.rs crates/xterm-core/tests/write_buffer_cases/*.rs
```

Observed original source baseline 19/19 passing, 8,192 deterministic queue operations, rejection order and actual mutated/detached/resized Uint8Array assertions passing. Observed final Rust tests 42/42 passing with zero ignored tests, including the same 8,192-operation replay. Scoped Clippy and formatting exited zero. Raw output is retained alongside this review as `blake-source.log`, `blake-tests.log`, `blake-clippy.log` and `blake-fmt.log`.

Review found one integration defect during shared parser API migration: `WriteAction::from_parser` used a wildcard completion arm that silently accepted Failed and PendingFallible. The owner changed it to exhaustive handling returning Result, retaining immediate/future errors, and added three focused bridge tests. Independent reruns include those tests.

Checked strict 50,000,000-unit guard, UTF-16 lengths, empty-text falsiness versus empty byte objects, separate callback queue behavior, sync nesting comparison/reset, partial state after throws, callbacks without held queue borrows, 12ms slices, strict 50/51 compaction, pending boolean propagation, rejected-handler error-before-false-continuation ordering, disposal/cancellation, weak ownership and original typed error identity. Resized views can produce negative pending units; the Rust counter is signed f64 and preserves the observed source result rather than clamping it.

No remaining scoped queue defect found. This acceptance is not evidence that all native/browser scheduler adapters or Terminal async handlers are integrated. Those remain separate tasks and criteria. It provides no native Linux/macOS or browser execution. Workspace-wide formatting/lint and full terminal equivalence remain independent gates.
