# BS and reverse-index cursor restriction

Task 475 continues the same swarm's full Rust port. This repair covers two control handlers, not complete terminal, addon, browser or operating-system equivalence.

## Defect and change

`InputHandler.backspace` and `InputHandler.reverseIndex` call `_restrictCursor`, which clamps Y to scroll margins when DECOM is set. The Rust handlers clamped only to viewport rows. At the handler boundary an above-margin cursor could stay above the margin, while a below-margin cursor could modify the wrong wrapped row or scroll the wrong content.

`terminal/controls.rs` and `terminal/escape.rs` now reuse `restrict_csi_cursor`, the existing equivalent of the source restriction operation. BS keeps the source's temporary `x = cols` allowance only when reverse wrap is enabled, then restricts again after reversing a wrap. RI restricts before deciding whether to shift lines and after its non-scrolling decrement. No dependency, public API or unsafe code was added.

## Reproduction and coverage

```sh
# If this checkout has no source dependencies, point at an existing installation.
XTERM_SOURCE_DEPENDENCIES=/path/to/source_repo node scripts/control-margins-oracle.mjs
cargo test --locked -p xterm-core --test control_margins -j2
```

The optional dependency path supplies esbuild only. The source terminal is bundled from this checkout's untouched `source_repo`. The script removes its own temporary bundle in `finally`. Use `--generate` only to record an intended source fixture update.

The 288 cases cover normal and alternate buffers, DECOM on/off, reverse wrap on/off, X values 0/5/9 and Y values 0/1/2/3/4/9 before BS or ESC M. Dimensions are 5 by 5 with scroll margins 1 through 3. Each screen contains distinct rows with explicit wrapped flags. The shared complete-state projection checks both buffers, all cells and attributes, modes, margins, wrapped flags, cursor and ordered render/cursor events.

Direct cursor mutation intentionally models InputHandler boundary inputs. It does not prove that each input state is reachable through public terminal sequences. Negative JS cursor numbers have no direct representation in the current unsigned Rust buffer fields and are outside this scoped corpus.

## Evidence

Base main was `3a4f5e8f`. Source/native fixture and raw logs are under `results/control-margins/`.

- Source generation and non-generating replay each passed all 288 cases. Both verified the complete 776-file manifest before and after. The original P: reference also passed all 776 hashes after testing.
- Replacing only the two changed production files with their base versions produced 72 mismatches, 216 passes. `negative-old-production.log`, `negative-summary.txt` and `negative-counterexamples.txt` retain the actual failure. A `finally` block restored and byte-checked both production files before the fixed run.
- Fixed native fixture passed all 288 comparisons in one test function.
- `cargo test --locked -p xterm-core --all-targets --all-features -j2` passed 1,564 test functions across 245 summaries, with zero failures and zero ignored tests. `cargo test --locked -p xterm-core --doc --all-features -j2` passed both doctests.
- Strict all-target/all-feature core Clippy passed. Native and WASM workspace all-feature library checks passed. Core rustfmt and `git diff --check` passed.
- Full `cargo fmt --all -- --check` failed with Windows OS error 206. The raw failure is retained, not reported as a pass.
- The first test compile used a nonexistent `CoreTerminal.take_events` method. `first-test-compile-error.log` retains that error. The test now drains events through the existing shared projection and strict Clippy passes.

No browser run, full workspace test/Clippy matrix, Linux/macOS runtime, release/comparison gate or CI was performed for this task. Accepted task 466 is preserved separately and was not replayed. Full tasks 313/304 and overall equivalence remain open and NOT PROVEN.
