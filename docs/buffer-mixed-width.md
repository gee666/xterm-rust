# Mixed-width Buffer resize comparison

Task372 extends the existing Buffer resize oracle. It found no production mismatch in this corpus, so no runtime code changed. Full Buffer parity and overall equivalence remain NOT PROVEN.

## Inputs and retained assertions

`scripts/buffer-resize-oracle.mjs` still executes all61 unchanged `Buffer.test.ts` cases through actual Mocha/Chai. The only instrumentation wraps `Buffer.resize` to observe its input, output and ordered events. The source tree is not modified. Its original86 resize observations,1024 width-one random observations and directed savedY regression all remain. Comparing complete fixture case blocks against the parent revision found all1111 byte-identical, except the final directed case's id changes from1110 to1622 because new cases precede it.

The new seed is `0x372b0ff`, independent of the existing `0xb0ff67` stream. It generates64 traces with8 resizes each. Before every resize it fills rows with ASCII, CJK, emoji, combining UTF-16 strings, spaces and lone surrogates. Each width-two head has an adjacent empty width-zero placeholder with matching attributes. A head that would cross the row end becomes a blank width-one cell instead. These are constructed Buffer cells, not parser-generated input.

Options vary normal/alternate ownership, scrollback, reflowCursorLine, Windows PTY backend/build and whether live host dimensions change before the operation. Each step adds a marker and varies cursor, display/base and saved positions. The512 new observations contain368 normal and144 alternate states,237 narrowing,237 widening and38 equal-width resizes. Before-state cells include4593 wide heads with4593 placeholders,5119 combined cells and11144 cells with active extended attributes. Source hashes and individual case descriptions remain in `results/buffer/resize-oracle.json`.

The existing snapshot helper checks full packed cells, foreground/background, extended attributes, hyperlink ids, combined UTF-16, full/trimmed translations, row identity and wrapped flags, dimensions, cursor, saved positions, scroll region/capacity, markers and ordered insert/delete/trim/disposal events. The Rust test now executes all1623 observations through both `Buffer::resize` and `Buffer::resize_shared`,3246 comparisons per build profile. No assertion was removed or relaxed.

## Observed checks

Windows x64, Node24.15.0, Rust/Cargo1.85.1, base331f0ee:

- Source baseline61/61 passes, no skipped or unrun cases. Default oracle invocation checks fixture equality rather than silently rewriting it. Only `--generate` updates it.
- The five-target focused debug command passes52 Rust test functions, including all3246 resize comparisons and existing reentry, upstream Buffer and reflow tests.
- The release differential target passes its one test function and all3246 comparisons.
- Workspace formatting and warning-denied Clippy for the changed integration test pass.

The initial longer private checkout path caused Windows OS206 in `cargo fmt --all`. Moving the same owned worktree to the shorter `O:/xterm-rust-swarm-16/tmp/K` path allowed the full command to pass. No formatting exclusions or lint suppressions were added.

Raw logs and input hashes are in `results/buffer-mixed-width/`. The helper `scripts/check-buffer.mjs` now carries the new seed/count fields and removes its obsolete width-one-only limitation. That broader helper was not run for this task. The directly executed commands are below.

```sh
# Optional dependency override for an isolated worktree without node_modules:
# XT_NODE_MODULES=/path/to/existing/source/node_modules
node scripts/buffer-resize-oracle.mjs
cargo test --locked -j2 -p xterm-core --test buffer_resize_differential --test buffer_resize_reentry --test buffer_screen_upstream --test buffer_reflow_upstream --test buffer_reflow_edges
cargo test --locked -j2 --release -p xterm-core --test buffer_resize_differential
cargo clippy --locked -j2 -p xterm-core --test buffer_resize_differential -- -D warnings
cargo fmt --all -- --check
```

## Independent review

The isolated task372 reviewer accepted commit `9b68fcc` with no findings or tracked edits. It independently checked preservation of all1111 prior case blocks, valid wide-placeholder layout and matching attributes in all512 new before-states, and exact coverage counts. It reran the uninstrumented61-case source baseline, the61-case instrumented oracle with exact fixture equality, all52 focused debug functions, the release differential, scoped warning-denied Clippy and full workspace formatting. Review logs are retained beside the implementation logs under `results/buffer-mixed-width/review-*.log`.

Each native transition hydrates its before-state independently. This comparison does not prove preceding native parser operations, shared extended-object aliasing, browser callbacks or host scheduling. It supplies no new browser or native Linux/macOS result, and it is not the complete final workspace feature/platform matrix. Full family314 and original-suite327 remain open.
