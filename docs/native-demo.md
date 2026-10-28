# Native terminal demonstration

The line-oriented example calls `xterm_core::headless::Terminal`. Incoming writes execute the Rust parser, mutate its real buffer cells and attributes, and appear in the viewport inspector. `input` emits outbound data separately. It does not start a shell, PTY, JavaScript engine or background service.

Full xterm.js equivalence is NOT PROVEN. This demonstration does not replace the complete native-platform, browser or addon checks.

## Run it

Requires the workspace Rust toolchain, Rust 1.85 or later, Cargo and the checked-in lockfile. Run from the repository root:

```sh
cargo run --locked -p xterm-core --example native -- --plain
```

Omit `--plain` to render the cells' colors and common style flags using the host console's ANSI support. The initial sample contains red/green text, astral/CJK/combining Unicode and cursor movement that changes `cursor: abc` into `cursor: XYZ`. Colors are derived from the parsed cells, not directly echoed input escape strings. The small native inspector is not the browser renderer or a promise of identical font shaping.

Try these commands inside the example:

```text
reset
resize 8 4
write abcdefghijkl\r\n
show
resize 12 4
input hello\r
serialize
quit
```

Before widening, the first two lines are `abcdefgh` and `ijkl`. Afterwards the first line is `abcdefghijkl`. The cursor was moved below that wrapped group so default upstream reflow participates. The source option `reflowCursorLine` defaults to false, so resizing a group containing the cursor is deliberately different.

`input hello\r` adds `hello\r` to the outbound UTF-16 observation. It does not echo those characters into incoming display cells. `serialize` calls the Rust buffer serializer and displays terminal-safe escaped UTF-16, rather than executing its returned controls on the host. A live fallible stderr sink displays real engine diagnostics with escaped UTF-16 instead of accumulating an inaccessible memory log.

## Commands and errors

- `sample`, `show`, `reset`, `serialize`, `help` and `quit` take no arguments.
- `write TEXT` sends decoded UTF-16 through the terminal's write path.
- `input TEXT` sends decoded UTF-16 through its user-input path.
- `resize COLS ROWS` validates unsigned integers, at least two columns and one row, checked cell-count arithmetic and the configured cell budget before calling the live resize API.
- Text supports `\e`, `\n`, `\r`, `\t`, `\0`, `\\`, `\xHH` and `\uHHHH`. UTF-16 escapes may contain lone surrogates. Unknown/incomplete escapes fail before a write can mutate the buffer.
- `--max-cells N` configures the example's viewport budget, default 1,000,000 cells. This protects the inspection UI from accidental huge dimensions; it is not an engine protocol limit. The budget must fit the initial 40-by-8 viewport.

Unknown commands and malformed input produce useful errors. Interactive mode continues after a rejected command. Script mode fails at that file and line with a nonzero exit status. EOF and `quit` invoke explicit fallible terminal cleanup. Output I/O errors propagate rather than becoming panics. Argument parsing uses `args_os`; non-Unicode option names fail gracefully, while script paths remain lossless OS paths. No sleeps or retries drive terminal behavior.

The engine's default Unicode provider is version 6. Its width for U+1F600 is one, while CJK `界` is two. A modern host console may draw emoji with a different width. Core cell/width assertions, not host glyph spacing, establish this source behavior. Native UTF-8 inspection replaces lone surrogates for display only; script assertions and escaped outbound/serialized observations retain every UTF-16 unit.

## Deterministic checks

```sh
cargo test --locked -p xterm-core --example native
cargo run --locked -p xterm-core --example native -- --plain --script crates/xterm-core/tests/fixtures/native-demo.txt
node scripts/check-native-demo.mjs
```

The script uses the same command executor as interactive mode. `assert-size`, `assert-cursor`, `assert-line`, `assert-cell`, `assert-fg`, `assert-width` and `assert-output` inspect actual Rust cells, attributes, widths, cursor, dimensions and emitted data. Text follows the coordinates without shell-style quoting; preserve the trailing space for an expected empty line.

The Node checker needs Node 24 or later but no npm packages. It reuses the repository's bounded process runner and unique-attempt evidence helper. It runs example tests, warning-denied Clippy, formatting, a clean example build, plain/color script executions, an actual stdin-driven interactive session and EOF cleanup. The interactive session deliberately rejects a command, then verifies line/data/size state and explicit quit cleanup. Expected script failures require a real nonzero exit code without a signal, spawn error, resource failure or Rust panic. Negative cases cover malformed escapes, unknown commands, zero dimensions, excessive viewport requests and deliberately failing state assertions. Their real nonzero exit codes remain recorded, separately from whether each expected-failure check passed.

Results go to immutable `results/demos/native/runs/` attempts with a current `latest.json` pointer. The checker observes a 1 GiB free-space reserve on the root and existing target filesystem, honors `.cargo/config.toml`, validates Cargo's resolved target OS and shared directory, and records source hashes before/after. It launches the executable reported by the current Cargo JSON build artifact, never a hardcoded potentially stale binary. A changing shared tree produces incomplete status and requires a stable rerun. It never rewrites earlier failures into success.

## Current evidence

The original headless JavaScript reference was run read-only to check the sample's Unicode widths and cursor-line reflow behavior. The first Rust demo tests retained two incorrect initial assumptions, emoji width two and reflow of the cursor's group. The unchanged source demonstrated widths `[1,1,2,0,1]` and no cursor-group reflow. The assertions now match that reference, and the resize sample moves below its group.

At this checkpoint, the first test run passed three tests and rejected those two incorrect assumptions. The next integrated build encountered the live core reflow-function migration. These failed attempts are retained rather than represented as a working-demo pass. Independent source pre-review additionally caught non-Unicode argument panic risk, overly broad negative-exit acceptance and missing interactive-loop coverage. The source now has OS-lossless arguments, stricter failure metadata and a real stdin-driven command/error/recovery session. A later leased run passed all ten native example tests, including actual cell/color/Unicode/input/reflow behavior and lossless argument/UTF-16 boundaries. Its full checker stopped on an unused private Buffer helper under warning-denied Clippy; that owner reports the helper wiring fixed. The newest console-sink integration and complete CLI checker still need a controlled rerun. Final example/checker results and independent runtime review are required before task #132 is accepted. Linux/macOS execution is not claimed here.
