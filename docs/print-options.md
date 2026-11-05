# Print option getter phases

Task 504 repairs one print startup contract. It does not complete the parser, browser or all-addon port. Overall equivalence remains NOT PROVEN.

## Source behavior and repair

The supplied `InputHandler.print` captures the charset object before reading `rawOptions.screenReaderMode`. It captures columns, modes, the current attribute object and the active buffer row after that getter. The getter can reset, resize, switch buffers, mutate attributes or modes, invoke a line feed, or throw. Its error stops the print before cell writes. A missing row suppresses printing only after the getter runs.

`terminal/print.rs` previously held mutable terminal state and the active buffer borrowed during that getter. It also captured columns, modes, attributes and the row too early. The repair captures only the charset handle and options service, releases the state borrow, invokes the getter, and then resolves the remaining live state. Cloning the charset preserves its object identity. Replacing the selected charset in the getter must not redirect the current print run, but mutating that captured map must affect character lookup.

## Regression corpus

```sh
XTERM_SOURCE_ROOT=/path/to/untouched/source_repo node scripts/print-option-oracle.mjs
cargo test --locked -p xterm-core --test print_option -j2
```

Generation is explicit with `--generate`. Normal replay rejects fixture drift. The oracle bundles unchanged headless source and verifies all 776 reference file hashes before and after execution. Rust executes its own engine against the retained fixture, not JavaScript.

The 204 cases combine ASCII, wide-character and soft-hyphen runs, both initial buffers, both screen-reader truth values and 17 getter actions. Actions include reset, buffer activation, resize, cursor movement, mode and attribute mutation, charset replacement and mutation, nested line feed, throwing before or after mutation, removing the active row, and an initially absent row with inspect, reset or throw. Missing-row setup explicitly clears the physical slot as well as the logical length. CircularList deliberately retains old physical slots when length shrinks.

Each case compares both buffers, all cells and attributes, cursor, dimensions, margins, terminal modes, callback checkpoints, getter counts, exact original error identity and ordered accessibility, line-feed, scroll, resize, cursor and render events. This is a deterministic regression corpus, not randomized coverage or a complete original InputHandler suite.

## Evidence

Base main is `5088f8fd`. Commands and raw receipts are retained under `results/print-option/`.

- Source generation and drift replay each pass 204 cases, with 776 reference hashes intact before and after.
- Core rustfmt passes. Full `cargo fmt --all -- --check` fails with Windows OS error 206, retained as a failure rather than waived.
- Native fixed and restored runs pass all 204 cases in one test function. The old production file fails all 204 cases. A borrow-free variant that still captures columns, modes, attributes and row before the getter fails 92. Both controls execute the regression, not just a compile failure. Each replacement restores exact source bytes in `finally`, with matching SHA-256 hashes in `negative-summary.json`.
- All-target/all-feature core tests pass 1,567 test functions across 248 result summaries, with zero failed, ignored or filtered tests. Both core doctests pass.
- Warning-denied native workspace Clippy passes all targets and features. Native workspace all-feature library checking and WASM workspace all-feature library Clippy also pass. These are not full workspace runtime tests or browser runtime evidence.
- The first headroom probe could not locate PowerShell on PATH. The next Windows command wrapper had a syntax error before Cargo ran. Using absolute tool paths and invoking Cargo directly resolved these launcher errors. `first-shell-error.log` retains the command failure; neither launcher failure is counted as a negative-control pass.
- Whole-core tests refreshed three existing buffer/serialize result files. Their diffs were inspected and restored only in this owned worktree. The scoped regenerated buffer summary is preserved here, not attributed to this repair.
- No browser, CI, native Linux/macOS, release or performance checks ran for this task. Temporary driver scripts and the live native binary remain available for independent review before owned cleanup. `source-pins.json`, `receipt-pins.json` and `binary-pins.json` identify exact review inputs.

Full family work remains in tasks 313, 327 and 304. Browser raw-option integration and complete original-suite coverage are not established by this native regression corpus.
