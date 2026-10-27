# Independent Kitty keyboard verification

Victor owns task 54, `scripts/verify-kitty-keyboard.mjs`, generated fixtures, this review and `results/kitty-keyboard/`. Wallada owns task 51's encoder, runner and Rust tests. The exact branch checklist is [kitty-acceptance.md](kitty-acceptance.md), criteria 85 through 89. These checks cover the portable encoding policy, not terminal mode negotiation or browser dispatch.

## Scope and method

The oracle loads unchanged `source_repo/src/common/input/KittyKeyboard.ts` and `KittyKeyboard.test.ts` through Node's TypeScript erasure. It executes every test body with inherited beforeEach hooks and counts every original strictEqual assertion. Each evaluate call captures the complete result, including action, cancellation, absent versus empty key and every UTF-16 output unit. Static shouldUseProtocol calls have a separate fixture. Normal runs reject fixture drift; `--generate` requires reference review.

The Rust fixture tests replay 169 evaluation inputs and four protocol-enablement inputs. The differential runner compares full results, not parsed or normalized sequences. The current corpus has 508,249 cases, with exact category counts in `results/kitty-keyboard/verification.json`:

- Every source-mapped name, modifier/lock names, ordinary/empty/malformed text, unmapped F26-F35 names and Object.prototype inherited names across all 16 modifiers, 32 flag combinations and three event types.
- Physical modifier/numpad/Key/Digit codes across modifiers, flags, event types and both Mac Option policies.
- Every BMP unit through direct associated text, shifted physical-key lowercase and Option base-key recovery. Lone surrogates remain units.
- 25,000 seeded synthetic events, 5,000 seeded long decimal numpad suffixes and explicit lexical/parseInt boundary cases, including decimal overflow.

The seed is 1263096916, hexadecimal 0x4b495454. Source hashes, Node/Rust versions, OS and CPU are in the raw JSON. The script retains up to 20 counterexamples and counts every failure; successful reruns do not remove retained failures. No mismatch was found in the current corpus.

## Independent source review

Inspected `kitty_keyboard.rs`, `kitty_keyboard/keys.rs`, `kitty_keyboard/inherited.rs`, both Rust test modules, the shared TSV adapter and original source.

- Modifier and lock suppression precedes all mapping, and release suppression uses the source ordering.
- CSI letters, SS3, tilde parameters and all functional/modifier/media mappings match the source tables. F26-F35 remain absent despite the upstream comment.
- Numpad lexical comparison precedes decimal parseInt, including accepted multidigit suffixes and trailing junk. Floating-point formatting uses the already-declared JavaScript-compatible `ryu-js` dependency. The parser's expect has a valid invariant: this branch guarantees a nonempty ASCII decimal prefix.
- Enter/Tab/Backspace release exceptions and the distinction between cancel=true encoded sequences and cancel=false legacy bytes match.
- CSI-u alternate and associated-text eligibility, empty modifier fields, repeat suffix suppression and release suffix emission match the source.
- Key recovery preserves UTF-16 lengths, ASCII-only folding of textual A-Z, the shared Unicode 17.0 physical lowercase table and first-unit extraction for expanding lowercase such as Keyİ.
- Review found inherited ordinary-object names would differ from direct Rust table lookup. Wallada added inherited-name handling; the expanded full matrix now verifies it. This deliberately pins observed Node 24/V8 native-function strings. Firefox/WebKit native-function strings and mutated JavaScript prototypes are not established by this Node-only policy check.
- The initial Rust numpad iterator did not compile. The owner fixed its double-reference handling; current builds and tests execute that branch.
- No unsafe code or browser/OS dependency was added to this policy. The runner propagates malformed fixture parsing and I/O errors rather than swallowing them.

## Observed commands and results

The resumed run honors `.cargo/config.toml`, which disables dev/test debug symbols and incremental caches without disabling assertions. It uses the shared target directory. Available P: space was 2.7 GiB after focused checks.

On Windows x86-64 with Node 24.15.0, Unicode 17.0 and rustc 1.85.1:

```sh
node scripts/verify-kitty-keyboard.mjs
cargo test -p xterm-core --test kitty_upstream --test kitty_edges
cargo clippy -p xterm-core --example kitty_oracle --test kitty_upstream --test kitty_edges -- -D warnings
rustfmt --check --edition 2021 crates/xterm-core/src/input/kitty_keyboard.rs crates/xterm-core/tests/kitty_edges.rs crates/xterm-core/tests/kitty_upstream.rs crates/xterm-core/tests/support/kitty.rs crates/xterm-core/examples/kitty_oracle.rs
```

- Oracle exit 0. All 165 original tests and 173 assertions passed; all 508,249 differential comparisons passed. Logs are `differential.log` and `verification.json`.
- Focused Rust tests exit 0. Five edge tests and two fixture tests passed. `rust-tests.log` retains a warning from the in-progress buffer reflow module at the observed snapshot.
- Focused formatting exit 0, including child mapping modules, recorded in `own-fmt.log`.
- Focused Clippy currently exits 101 on unrelated `serialize/ansi.rs` comparison-chain and `buffer/reflow.rs` unused Result findings. No Kitty finding appears. `clippy.log` is the current failure, not a passing quality claim. Owners were notified.

Earlier workspace fmt, Clippy and test attempts are retained in `fmt.log`, `workspace-clippy.log` and `workspace-tests.log`. They failed on active addon formatting, parser-table/cell lints, a missing cell fixture and the then-unexported reflow module. They are historical failures, not current passes. Workspace and native/browser gates remain independently required.

Linux/macOS execution, terminal flag query/set/push/pop/reset, actual DOM keydown/keyup selection, legacy/Win32 precedence, cancellation, duplicate-input prevention, listener disposal and Chromium/Firefox/WebKit evidence are outside this portable-policy result. Tasks 28 and 33 own that integration. Full terminal equivalence remains NOT PROVEN.
