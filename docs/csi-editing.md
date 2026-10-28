# CSI cursor and editing implementation

`terminal/csi.rs` implements CUU/CUD/CUF/CUB/CNL/CPL, CHA/HPA, CUP/HVP, HPR/VPA/VPR, CHT/CBT/TBC, DECSTBM, cursor save/restore dispatch and REP. `terminal/csi_edits.rs` implements ICH/DCH, IL/DL, SU/SD, SL/SR and DECIC/DECDC. `terminal/csi_erase.rs`, task 111, owns ED/EL/ECH/DECSCA and selective erase. `terminal/commands.rs` registers the source identifiers on the actual Rust parser. REP's callback-safe printing handoff is shared work in task 153, not a JavaScript fallback.

Source names map to snake_case helpers. Intermediate bytes are represented in dispatch strings, such as ` @`, ` A`, `'}`, `'~` and `\"q`; the parser registration still distinguishes prefix, intermediates and final byte. `?J` and `?K` use the selective flag. Unknown commands return false rather than being reported as implemented.

The implementation deliberately preserves source quirks. Relative movement restricts the old cursor before origin-aware placement. CHA/HPA pass the current absolute Y through that same placement. Pending-wrap character edits first clamp X, while column edits retain BufferLine's original position semantics. SU fills with BCE erase attributes, but SD uses default attributes. IL/DL preserve one deletion/insertion event pair per requested row rather than batching away marker observations. List callbacks run without a Buffer borrow; SU/SD read changed margins after observers, matching source reentrancy. Typed errors retain the exact callback cause through TerminalError, BufferError, CircularListError and EmitError.

Ordinary parameter zero defaults to one where the source uses `|| 1`. Negative repeat counts and negative scroll-region values cannot arrive from the parser and return typed errors from direct Rust helper calls rather than entering source non-progress loops. REP converts whole grapheme UTF-16 strings to codepoints without replacing lone surrogates, reserves storage fallibly, and uses the actual printing path.

## Verification

`node scripts/check-csi.mjs` runs the scoped original baseline, deterministic fixture verification, Rust debug/release tests, warning-free scoped Clippy and scoped formatting. Every process has a bounded lifetime and monitored disk headroom. Attempts are retained with before/after dependency hashes. An incomplete/failed receipt cannot reuse a previous success. Full workspace quality remains a separate gate.

The original baseline is 52 cases from InputHandler.test.ts. It retains original assertions and executes real Mocha/Chai. The exact case-to-Rust-function mapping is in `results/csi/baseline.json`.

- 37 source cases are fully mapped to `terminal_csi_cursor.rs` and `terminal_csi_editing.rs`.
- 15 source cases have their positive out-of-range assertions mapped, but their negative direct Buffer cursor mutations are unrepresentable in the current `usize` fields. Those 15 assertions are explicitly unrun. Task 152 owns a faithful numeric representation/adapter decision. No test clamps its input first or labels those missing preconditions passed.
- `terminal_csi_events.rs` adds callback mutation, live-margin and exact error-identity/source-chain checks. Its observed event arrays were checked directly against the source terminal.
- `terminal_csi_differential.rs` reuses the whole-terminal projection and replays 249 traces, 2,515 snapshots, seed `0xc5107`. The corpus includes 37 command families and their default, zero, ordinary and large parameters; Unicode cells, origin mode, margins, pending sequences and chunk boundaries. Both buffers and every projected cell/attribute/state/event field are compared after each input chunk.

The generated fixture intentionally exceeds the handwritten-file line limit. It is retained oracle data, not implementation. Source hashes and raw failures remain in `results/csi/`. The baseline passing against JavaScript is not a Rust pass; read the actual Rust results and current receipt status.

Overall equivalence remains NOT PROVEN. Negative internal cursor assertions, full unported InputHandler families, browser/addon behavior and native Linux/macOS runtime evidence are not established by this scoped work. The controlling checklist is `docs/acceptance/csi-editing.md`.
