# Nonnumeric ANSI scrollback allocation

Tasks521 and526 repair two parts of raw serialization346. Full346/322 and overall equivalence remain NOT PROVEN.

## Source contract and repair

The original `_serializeBufferByScrollback` reads buffer length and terminal rows, evaluates raw `scrollback + rows`, clamps that result and computes start/end coordinates. ANSI allocates `new Array(end - start)` before traversal. Boxed primitives, strings, arrays and objects can produce NaN, fractional or zero-retained counts. Those counts must reach the browser's allocation check, with its native RangeError message, before late addon option getters run. An undefined scrollback still traverses the entire buffer.

Accepted498 fixed primitive numbers but left nonnumeric inputs on an integral-coordinate wrapper. Accepted507 factored the raw addition/clamp into `scrollback_row` for HTML without changing ANSI. This repair uses that same raw arithmetic for ANSI, then calls existing `line_range` allocation validation for every input type. It deletes the now-unused integral/fixed-message wrapper. HTML traversal and native serializers are unchanged. No dependency, unsafe, source-reference or AGENTS changes.

The branch starts on main1106d480, which already contains498. Gordon ACKed private copies of507's exact five commits. Those copies are prerequisites only, not part of521's producer chain. Their original commits remain canonical. The union preserves all85 inherited cases, including498's three and507's four. The new runner has90 cases,58 directed and32 seeded.

## Regressions and the missing-line repair

Four new allocation probes cover21 raw value kinds in both normal and alternate buffers, default-hint/getter ordering, nested serialization and reset/resize during coercion, seven thrown-sentinel phases, and full public-cell snapshots. The successful reentry control grows the reset buffer so its captured endpoints remain traversable.

Preliminary independent review found the original successful reentry control also shrank the buffer below its captured endpoints. Source traversal then read `isWrapped` from an absent next line. Rust constructed a V8-specific TypeError message in `crates/xterm-core/src/serialize/mod.rs`, exposed through `serialize_addon/view.rs`. Rather than omit that failure, task526 repairs this separate browser diagnostic contract. The dedicated `ansi-scrollback-captured-missing-line` case retains the original reentry counterexample and adds two explicit past-buffer ranges. The initial test remains in commit ec2b22ac; the reviewer executed it against the actual original headless implementation.

Task526 changes only browser error construction. The Rust `MissingWrappedLine` variant still selects the failure at the same traversal point. A small WASM host helper deliberately evaluates `nextLine.isWrapped` with `nextLine` undefined and returns the caught native error object. Keeping the original expression matters because Firefox/WebKit include it in diagnostics. This is host error adaptation, not JS terminal-engine delegation. Native Rust errors and traversal remain unchanged. The baseline must expose the Firefox/WebKit message mismatch; fixed must match every raw case without filtering.

`results/serialize-ansi-coercion/reproduce.mjs` reuses the existing actual source/Rust browser runner. It swaps only the three owned production files for521/526, stages baseline/fixed packages, verifies on-disk build identities, checks exact failure names, compares all raw traces and unchanged-suite reports, and restores fixed files/package state in finally paths. Baseline production is ec2b22ac. Baseline/fixed builds and browser evidence are pending explicit resource grants. No runtime pass is claimed yet.

## Commands and quality scope

Use an owned O: worktree, private target/TEMP/TMP/TMPDIR, Cargo -j2, Rust1.85.1 and wasm-bindgen0.2.100. Set SOURCE_ROOT to the unchanged reference with installed Playwright dependencies. Obtain compiler and browser allocations before execution.

```sh
node results/serialize-ansi-coercion/reproduce.mjs quality
node results/serialize-ansi-coercion/reproduce.mjs build
node results/serialize-ansi-coercion/reproduce.mjs proof
node results/serialize-ansi-coercion/reproduce.mjs audit
```

Quality runs all16 native serializer executables, strict scoped native and actual WASM library Clippy, the required exact workspace format check and each package's format check. An OS206 formatting failure remains a failure, not a substituted workspace pass. This work does not claim full workspace/all-features/release/native Linux/macOS or complete browser equivalence.
