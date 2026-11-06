# Negative combining targets during print

Task 579 fixes one native print defect. Full parser, browser, platform and xterm.js/Rust equivalence remain open.

## Source behavior

A custom Unicode provider can return a zero-width first character followed by a joining character. After the first character the cursor is at column 1, but cell 0 has width 0. `InputHandler.print` chooses offset 2 and calls `BufferLine.addCodepointToCell(-1, ...)`. Reading that negative typed-array index yields undefined and the final typed-array write is ignored. The source does not change cell 0. It still writes any width-growth filler cells, advances the cursor and preserves the new preceding-join state.

Rust previously used `x.saturating_sub(offset)` and corrupted visible cell 0. `print_render.rs` now uses `checked_sub` and only omits the impossible cell write. All following filler/cursor/state work is unchanged. The public custom-provider contract permits these packed width/join properties, so a provider's unexpected combination must not be redirected onto a different cell.

## Evidence

Base is main `14139449`. Source commit `8fcf69c9` has no private prerequisites. It does not repeat accepted tasks 555 or 565.

`results/print-negative-target579/source.json` records 64 deterministic custom-provider cases and 160 full-state snapshots. Cases cover initial and joined widths 0 through 3, joining at negative and valid targets, single/split chunks, insert mode, subsequent output and Bell. Both buffers, packed cells, attributes, cursor, modes, ordered events and preceding-join state are compared. All 776 original reference hashes pass before and after source execution. The same runner executes the complete unchanged original `InputHandler print` suite, 8 passed, 0 failed or skipped. These JavaScript checks are not Rust coverage.

The native fixture test passes all 160 snapshots. Running the exact base production module against the same final assertions produced 128 matching snapshots and 32 mismatches, exclusively from negative-target corruption. The fixed module was restored before the successful rerun. Raw logs retain the failure rather than weakening assertions.

Focused native gates passed 21 tests across the new fixture, existing original print/wide/charset ports, print/core differential, print-option differential and full terminal-engine differential, plus 5 print unit tests. Strict `cargo +stable clippy -p xterm-core --all-targets --all-features -j2 -- -D warnings` passed. Aggregate workspace fmt failed Windows OS206. All five complete member `cargo fmt --manifest-path ... -- --check` commands passed, which is not an aggregate-command pass.

The first new native test compile had a u8/u32 mismatch in the test provider. It was corrected before controls. The first all-target Clippy attempt failed because this sparse worktree omitted the tracked `results/windows-wrapping/source.tsv` fixture. Restoring that tracked directory fixed the checkout omission; the strict gate then passed. Both failures are retained. The source runner's final cleanup uses nested finally so reference-verification failure cannot leave its owned bundle behind; that cleanup-only edit passed Node syntax checking after the successful source run.

No browser/WASM runtime, full native workspace suite, workspace strict Clippy, doctests, release matrix or native Linux/macOS execution was run for this producer. Final integration and full family tasks remain responsible for those checks.

## Reproduction

From a complete checkout with source dependencies installed:

```text
node scripts/print-negative-target-oracle.mjs
cargo +stable test -p xterm-core --test print_negative_target -j2
cargo +stable test -p xterm-core --all-features --test input_handler_core_print --test input_handler_core_differential --test print_option --test terminal_engine_differential -j2
cargo +stable test -p xterm-core --lib terminal::print --all-features -j2
cargo +stable clippy -p xterm-core --all-targets --all-features -j2 -- -D warnings
```

The source runner accepts `--source-root PATH` for an unchanged reference outside the worktree. `--generate` deliberately replaces the fixture and requires review. Its bundle lives under the checkout's `tmp/print-negative-target579` and is removed on success or failure. New worktrees/build outputs must follow the user's O: policy.
