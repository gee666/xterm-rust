# ED2 option getter reentry

Task 513 continues the same swarm's full Rust port. It does not close full parser task 313, original-suite task 327, quality task 304 or overall equivalence.

## Source contract and repair

`InputHandler.eraseInDisplay` clamps the cursor before branching on the erase parameter. For ED2 and selective ED2, it reads `rawOptions.scrollOnEraseInDisplay` before capturing the row count, scanning lines or clearing rows. The getter can reset, resize, activate another buffer, change attributes or cursor, and throw. Subsequent operations use the live state. Truthy raw values select scrolling; falsy values select row reset. Getter errors retain their original identity and any committed cursor clamp or callback mutations.

The Rust `terminal/csi_erase.rs` implementation retained an immutable `TerminalState` borrow while calling that getter through `option_bool`. Reentrant callbacks needing mutable state therefore failed with a borrow error. The repair captures the shared options service, releases state ownership, then reads the option. The row count and erase receiver remain live lookups after the getter. No new dependency, public API, unsafe code, rendering policy or duplicate terminal state was added.

## Reproduction

```sh
XTERM_SOURCE_ROOT=/path/to/untouched/source_repo node scripts/erase-option-oracle.mjs
cargo test --locked -p xterm-core --test erase_option -j2
```

`XTERM_SOURCE_ROOT` is optional when the current checkout contains the reference and its installed esbuild dependency. The source script bundles the unchanged headless terminal, verifies all 776 baseline reference hashes before and after, and removes the owned temporary bundle in `finally`. Normal execution rejects fixture drift. `--generate` deliberately captures the fixture; `--no-receipt` replays without rewriting the source receipt.

The 396 deterministic cases cover both initial buffers, selective and ordinary ED2, nine raw values and eleven actions. Values include false, true, zero, NaN, empty and nonempty strings, an object, null and undefined. Actions cover inspection, reset, activation of either buffer, resize, cursor or attribute mutation, getter errors before and after mutation, and scroll listeners that reset or throw. Each case starts beyond the allowed cursor column to verify the pre-getter clamp, including on error paths.

The comparison includes both buffers and every cell's text, width and attributes, cursor, scrollback, margins, dimensions, modes, getter counts, callback checkpoint states, original callback-error identity and ordered scroll, resize, cursor and render events. The fixture contains 84 expected thrown results. Source baseline execution is not counted as a Rust test pass. Native Rust tests execute Rust, not the JavaScript engine.

## Verification status

Base main is `e64f4870`. Raw logs and receipts are under `results/erase-option/`.

Source generation and non-generating replay each pass all 396 cases and preserve all 776 original hashes before and after. Native execution, negative controls and quality checks are pending while waiting for the team's compiler allocation. No native pass is claimed yet.

These cases are finite, deterministic, source-derived regressions. They are not randomized coverage, a complete original InputHandler suite, or browser binding evidence. No Linux/macOS runtime, CI, release/performance/size comparison or full browser/original-suite matrix has run for this task. Full equivalence remains NOT PROVEN.
