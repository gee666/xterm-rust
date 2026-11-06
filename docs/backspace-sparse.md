# Backspace on a sparse current row

Task 555 fixes one InputHandler boundary behavior. It does not close the complete parser, buffer, browser or platform families. Overall equivalence remains NOT PROVEN.

## Behavior

Upstream `source_repo/src/common/InputHandler.ts`, `backspace`, tests the current row with `lines.get(...)?.isWrapped` before reversing a soft wrap. A missing current row therefore leaves the restricted cursor in place and parsing continues. The previous row is different. Once a real soft wrap has been reversed, upstream requires the previous row and throws if it is absent.

`crates/xterm-core/src/terminal/controls.rs` previously required both rows. The fix makes only the current-row lookup optional. It retains the final cursor restriction and dirty-row update, rather than returning early. Previous-row failures and partial state changes are unchanged. No dependencies, unsafe code, parser registration or public signatures changed.

Sparse internal rows are boundary inputs here. The tests explicitly change internal list slots and cursor positions, just as upstream InputHandler tests directly change buffer coordinates. They do not establish that every such state is reachable from the supported public API.

## Reproduce

Use the repository's documented Rust toolchain and the unchanged reference dependencies. Run from the checkout root. Every command should have a finite execution deadline. Set temporary build outputs and scratch to the owned O: worktree when working in this swarm.

```sh
node scripts/backspace-sparse-oracle.mjs
cargo test --locked -j2 -p xterm-core --test backspace_sparse --test input_handler_core_backspace --test control_margins
cargo test --locked -j2 -p xterm-core --all-targets --all-features
cargo clippy --locked -j2 -p xterm-core --all-targets --all-features -- -D warnings
cargo check --locked -j2 --workspace --all-targets --all-features
cargo check --locked -j2 --target wasm32-unknown-unknown --workspace --lib --all-features
```

The source runner reads its own physical `source_repo` copy. If dependencies are installed elsewhere, set `XTERM_SOURCE_DEPENDENCIES` to the unchanged reference directory containing `node_modules`. It uses those packages without adding junctions or changing the reference. `--generate` regenerates only the owned Rust TSV fixture after source integrity checks. Normal execution rejects fixture drift. Its scratch directory is removed in `finally`.

## Corpus and observations

The deterministic matrix has 480 cases. It crosses normal/alternate buffers, origin mode, reverse wrap, columns 0/5/9, rows 0/2/4/9 and five row patterns. Patterns are missing current row, present hard current row, present soft-wrapped current/previous rows, missing previous row after a soft wrap, and missing previous row with a hard current row. The setup includes colored cells, scrollback and top/bottom margins. Out-of-range cursor coordinates exercise restriction before row lookup.

Each case parses BS followed by BEL. Both complete buffers retain explicit absent slots, packed cell content, UTF-16 text, foreground/background and extended attributes, cursor, scrollback/viewport coordinates and margins. Terminal dimensions, active buffer, current attributes, origin/reverse modes and ordered cursor/render/BEL events are compared. Eight cases intentionally fail on a missing previous row. Their partial cursor/wrap state and absence of following BEL match source. The direct InputHandler oracle bypasses JavaScript's host write queue, so native `WriteParsed` bookkeeping is outside this event projection. Queue suites own that contract. All other unexpected native events fail the test.

The source runner also executes both complete unchanged `BS with reverseWraparound set/unset` suites from `InputHandler.test.ts`. Eight tests pass. The existing Rust `input_handler_core_backspace` port passes all eight corresponding tests. The existing `control_margins` test passes its 576 source cases, including BS and reverse-index restriction controls.

## Retained evidence

Evidence is under `results/backspace-sparse/`. Source execution used Node 24.15.0 on native Windows and verified all 776 reference entries before and after. Native execution used rustc 1.85.1, target `x86_64-pc-windows-msvc`.

- Source matrix, 480 cases passed. Unchanged source backspace suites, 8 passed with no skips or unrun selected tests.
- New native test, 480 cases passed, including all 8 expected previous-row errors.
- Focused native command, 10 test functions passed. This includes the new 480-case function, the existing 576-case margin function and 8 original backspace ports.
- Core all-target/all-feature run, 1,590 tests passed across 256 summaries, zero failed, ignored, measured or filtered tests.
- Strict core Clippy, native workspace all-target/all-feature check and actual WASM workspace all-library/all-feature check passed. Full workspace runtime tests, strict workspace Clippy and doctests were not run on this producer. The integrated quality matrix remains task 304, not a claimed pass here.
- Exact `cargo fmt --all -- --check` failed with Windows OS206. All five complete workspace members passed their separate manifest formatting checks. This is not an aggregate-format pass or waiver.

`input-pins.json` identifies source commit `6e731f13`, 1,912 source/fixture/manifest files and three cached focused executables. Before cleanup, run `node results/backspace-sparse/audit.mjs` for read-only integrity and receipt checks. It requires the producer's source tree, reference copy and cached executables. These O: runtime artifacts become historical after cleanup. `XTERM_SOURCE_ROOT` can point to another byte-identical reference for the integrity checks. Three test-generated tracked reports were copied losslessly into `generated-report-copies/` before restoring just those known paths. No source file was restored in that cleanup step. Archive preflight found two pre-existing baseline inputs with CRLF checkout bytes differing from their LF Git blobs. Their exact compiled-input bytes are preserved under `working-input-copies/`, with hashes and verified CRLF-only differences in `working-input-copies.json`. No production or fixture source was changed by this preservation.

The negative control restored the exact `controls.rs` from main `224d1b33` while retaining the new assertions. It observed 472 matching cases and 8 mismatches, all missing-current-row reverse-wrap cases with `MissingLine` and lost following BEL/render. `negative-counterexamples.txt` preserves them. `negative-restoration.json` confirms the fixed file was restored byte-for-byte in `finally`. The restored focused run passed again. No source assertions were weakened.

Two initial local harness mistakes remain disclosed. The first native attempt rejected the engine's `WriteParsed` bookkeeping, which the direct source parse does not emit. The corrected projection documents this existing host-queue boundary and still rejects other unexpected events. The negative-control command initially expected 12 mismatches, but 4 inputs were outside the scroll margins and correctly never reached the optional lookup. Its observed 8 mismatches are retained without rerunning or altering the negative results. The fixed source file had already been restored by `finally` before that command failed.

This evidence is finite native Windows coverage. WASM compilation is not browser runtime evidence, and no Linux/macOS runtime, full upstream suite, complete parser family or whole-port equivalence is claimed.
