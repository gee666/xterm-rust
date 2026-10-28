# MoveToCell port

Task #129 owns the platform-independent cursor-sequence algorithm in `crates/xterm-core/src/input/move_to_cell.rs`. Its behavioral reference is `source_repo/src/browser/input/MoveToCell.ts`. Task #70 owns actual selection alt-click routing and task #33 owns browser input integration. This helper does not send input or move the terminal cursor itself.

## Source paths and edge cases

User criteria #375 and #376 have separate executable cases:

| Case | Paths and checks |
| --- | --- |
| move-01 | Normal buffer, same-row left/right/no movement; targets on rows above/below across row ends; only horizontal arrows; CSI versus SS3 application-cursor prefixes; targetX equal to cols and targetY equal to rows retained |
| move-02 | Alternate buffer, reset wrapped starting row, skip wrapped rows in vertical movement, horizontal direction after reset, same-position empty result; wide cells, width-zero trailing cells, combining UTF-16 strings, non-BMP characters, lone surrogate cells and initially wrapped first rows |

The reference has no reverse-wraparound argument. That mode must not acquire invented effects in this helper. Its alt-buffer backwards slices are asymmetric: moving left within one row can produce an empty sequence because the final translated slice has start greater than end. The Rust implementation preserves this behavior rather than correcting it.

The Rust entry point accepts signed integer coordinates and returns `Result<String, MoveToCellError>`. Coordinates are not clamped; the original tests explicitly include the outer viewport border. Nonfinite/fractional JavaScript numbers are not represented by this native signature. Arithmetic/allocation and outstanding buffer/line borrows are reported as recoverable errors. Output consists only of ASCII escape sequences. Character counts are UTF-16 code units, not Rust scalar values, byte lengths or occupied cells.

## Assertion and trace mapping

All four unchanged original Mocha tests execute after the upstream core esbuild build. The four Rust `source_*` tests retain their 31 assertions:

- Four same-row normal-buffer directional results.
- Twenty normal-buffer cross-row results.
- Six application-cursor results.
- One alternate-buffer down/right result.

`tests/fixtures/move-to-cell.tsv` contains 1,024 deterministic traces, 512 per case, from seed `0x6d6f7665`. Each trace uses actual BufferService, OptionsService and BufferLine implementations, not a translated JavaScript engine behind Rust. The corpus covers 2-8 columns, 2-6 rows, random cursor/target positions, application-cursor selection, wrapped-row flags and five repeating cell patterns. Directed source assertions supplement the corpus.

Source-derived fixture distribution is explicit. Move-01 has 258 application-cursor cases, 22 unchanged locations, 127 targets above, 267 below and 184 wrapped first rows. Move-02 has 260 application-cursor cases, 21 unchanged locations, 158 above, 242 below and 164 wrapped first rows. The reference returns empty sequences in 26 normal and 57 alternate cases, including the asymmetric-slice behavior. These are fixture counts, not passing Rust execution counts.

The adapter records independent JS and Rust returns, errors, post-state and ordered events. A canonical state string includes cursor, scrollback/display base, line count, every wrapped flag and every cell's width, foreground, background, codepoint and complete UTF-16 string. Rust additionally asserts buffer identity retention and zero resize/scroll events. Capture arrays are produced by the actual Rust test executable, not copied from the reference.

Verification never rewrites golden fixtures. Regeneration requires explicit `MOVE_GENERATE=1`. A changed source result fails against the checked-in fixture until reviewed. Zero cases fail in both the Rust test and shared runner.

## Commands and evidence

Install the original locked npm dependencies, then run:

```sh
node scripts/acceptance-oracle.mjs --case move-01
node scripts/acceptance-oracle.mjs --case move-02
cargo test -p xterm-core --test move_to_cell
```

`scripts/acceptance-adapters/move-to-cell.mjs` implements the shared runner contract. It hashes the exact reference sources, Rust native test executable and corpus, and returns nonzero assertion counts with full capture domains. The shared runner owns immutable attempts, process-tree cancellation and receipt validation. The adapter honors the shared target directory and 1024 MiB disk reserve. It neither runs a browser terminal nor claims real alt-click integration.

Standalone development attempts are retained under `results/move-to-cell/`. Original tests passed before initial Rust compilation was interrupted by an unrelated Buffer resize API migration. Final passing counts, independent review and user-case receipts must be observed before marking task #129 accepted. Full xterm.js equivalence remains NOT PROVEN.
