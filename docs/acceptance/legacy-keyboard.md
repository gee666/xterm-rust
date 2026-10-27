# Legacy keyboard acceptance matrix

Scope is `source_repo/src/common/input/Keyboard.ts`, `Keyboard.test.ts`, and the keyboard data/result definitions in `common/Types.ts`. Implementation and evidence are described in `docs/keyboard.md`. These checks are separate from actual browser event delivery. Browser routing must also satisfy criterion 27 and `docs/browser-input-acceptance.md`.

Every listed behavior is required, even when the original suite lacks a direct assertion. Unless a row says otherwise, result kind is SendKey and cancel is false. `undefined` maps to Rust None; empty text must never substitute for it. Result comparisons include kind, cancel and every UTF-16 unit, not only printable text.

## Special keys and actions

| ID | Input path or boundary | Required behavior and edge cases | Evidence |
| --- | --- | --- | --- |
| KB01 | Backspace, all modifiers | Control chooses BS rather than DEL; Alt prefixes ESC; Shift/Meta do not change this rule. | Original assertions, complete modifier matrix |
| KB02 | Tab and Shift+Tab | Plain Tab sends HT and requests cancel; Shift wins over all other modifiers, sends CSI Z and does not request cancel. | Original assertion, cancellation regression, modifier matrix |
| KB03 | Enter | Sends CR, Alt prefixes ESC, cancel true. Other modifiers do not change it except the iOS path KB18. | Original assertions, full-result matrix |
| KB04 | Escape | Sends ESC or ESC ESC with Alt, cancel true for every mask. | Original assertion, cancellation regression, matrix |
| KB05 | Arrows 37 through 40 | Meta suppresses output for all eight Meta masks. Otherwise any modifiers generate CSI 1;mask+1 plus D/A/C/B. Zero modifiers choose CSI or SS3 by application cursor mode. | Original assertions, meta regression, all 16 masks and both modes |
| KB06 | Insert | Shift or Control suppress output; Alt/Meta alone are ignored and CSI 2~ is sent. | Original assertion, insert regression, matrix |
| KB07 | Delete | CSI 3~ with zero modifiers, CSI 3;mask+1~ otherwise, including Meta. | Original assertions and modifier matrix |
| KB08 | Home/End | Unlike arrows, Meta is encoded. Zero modifiers select CSI or SS3 by application cursor mode; modified sequences use H/F and mask+1. | Original assertions, Home/Meta regression, matrix |
| KB09 | PageUp/PageDown | Shift produces PageUp/PageDown action with no key and no cancel even with other modifiers. Otherwise Control encodes the whole mask. Without Control, Alt/Meta are ignored. | Original assertions, page-priority regression, page modifier regression |
| KB10 | F1 through F4 | Unmodified uses SS3 P/Q/R/S regardless of application cursor mode. Any modifier uses CSI 1;mask+1 plus final. | Original assertions and all code/mask/options matrix |
| KB11 | F5 through F12 | Exact parameters 15,17,18,19,20,21,23,24. Modifiers add ;mask+1. No invented F13 mapping. | Original assertions, key codes 111/124 and complete matrix |

## Control, Alt and text branch priority

| ID | Input path or boundary | Required behavior and edge cases | Evidence |
| --- | --- | --- | --- |
| KB12 | Control-only A through Z | Emit values 1 through 26. Adjacent key codes 64/91 do not enter this path. Shift/Alt/Meta exclude this branch. | Every code and mask matrix |
| KB13 | Control-only punctuation | Space=NUL, 3 through 7=ESC/FS/GS/RS/US, 8=DEL; key '/'=US is checked before legacy bracket codes 219/220/221. Unknown codes produce no key. | Control punctuation regression, original Ctrl+Shift assertions, matrix |
| KB14 | Alt digit and punctuation map | Exact US-layout unshifted/shifted mapping for 48..57,186..192,219..222. Control is ignored for these mapped characters; Meta excludes this Alt branch. | All original mapping assertions and full mask matrix |
| KB15 | Alt A through Z and Space | Alt letters lowercase, Shift uppercase, Control chooses values1..26 even when Shift set. Space becomes ESC Space or ESC NUL under Control. | Original assertions and all code/mask/options matrix |
| KB16 | Printable key fallback | Requires no Control/Alt/Meta, keyCode>=48 and exactly one UTF-16 unit. Shift is allowed. Empty text, named keys and two-unit astral keys are rejected. | Original lower/uppercase tests, UTF-16 identity regression, all BMP single keys |
| KB17 | Ctrl+Shift physical punctuation | With nonempty key, Minus=US, Digit2=NUL, Digit6=RS. Preserve earlier Alt branch priority, Mac Meta-select branch priority and all unmatched None outcomes. | Original assertions, modifier/options matrix plus Dead-code matrix |

## Platform-specific paths

| ID | Input path or boundary | Required behavior and edge cases | Evidence |
| --- | --- | --- | --- |
| KB18 | iOS Ctrl+C | keyCode13, key exactly lowercase c and Control set emits ETX with cancel true; Alt does not prefix ESC. Uppercase C does not trigger the special case. | Explicit iOS regression |
| KB19 | keyCode0 mobile arrows | Exact UIKeyInputUp/Left/Right/DownArrow names map to CSI/SS3 A/D/C/B. Modifiers are ignored. Other key names at code0 produce no key. | Eight original assertions, all mobile names/masks/options |
| KB20 | Mac Option policy | Normal Mac Option letter is not an Alt escape. macOptionIsMeta enables the Alt branch. Special keys processed before this branch retain their independent Alt behavior. | Original Mac assertions and all options/masks matrix |
| KB21 | Mac Command+A | Only isMac plus Meta with no Shift/Alt/Control and keyCode65 produces SelectAll with no payload/cancel. Non-Mac and extra-modifier variants must not select all. | Explicit action-priority regression and matrix |

## UTF-16 and lowercase data

| ID | Input path or boundary | Required behavior and edge cases | Evidence |
| --- | --- | --- | --- |
| KB22 | Single-unit key identity | Preserve NUL, every BMP unit and unpaired surrogates exactly. Never use lossy UTF-8 conversion. Two-unit astral keys are not single JS characters here. | All65536 single-unit differential cases and Rust regression |
| KB23 | Dead-key routing | Only eligible Alt branch plus key exactly Dead and code starting Key. Code Key with no fourth unit emits ESC alone and cancel true. Non-Key codes and ordinary named keys are unmatched. | Dead-code/modifier/options matrix and empty suffix regression |
| KB24 | Dead-key slice | Slice only code unit3 even when it is a high surrogate, and discard subsequent units. Shift preserves it unchanged. | Exhaustive BMP shifted cases, non-BMP/lone-surrogate regressions |
| KB25 | Lowercase mapping | Unshifted dead-key suffix uses Unicode17.0 JavaScript full lowercase on one unit. U+0130 expands to i+dot; U+A7CE becomes U+A7CF. Unpaired surrogates remain unchanged. No Rust compiler Unicode dependency. | Exhaustive65536 lowercase cases, retained counterexample and regressions |
| KB26 | Numeric/interface limits | Borrowed key/code slices permit empty and malformed UTF-16. keyCode is DOM u32, with0,47,48,255 and u32::MAX tested. Browser WebIDL coercion is a separate adapter check. No panic on those inputs. | Differential matrix and seeded large-u32 cases |

## Evidence and integration gates

KB27. Run every original test body unchanged after TypeScript-only erasure and import/export relocation. Load result discriminants and escape constants from the original source, not independent hardcoded copies. Discover61 tests and156 assertions for this supplied reference. Zero failures, skipped or unrun checks are allowed within this module's suite. Preserve all156 original expected payload assertions in the checked-in Rust fixture and compare kind/cancel too. Detect fixture drift without overwriting it during ordinary verification.

KB28. Compare every result field for256 key codes x16 modifier masks x8 option sets, all mobile arrow/mask/options cases, all BMP single-unit and both dead-key cases, and20000 seeded cases. Seed1262836051, current total251452. Record iterations, exact Node/rustc/Unicode versions and source hashes in `results/keyboard/verification.json`. Save any mismatch including input/options/expected/actual. Do not delete the fixed U+A7CE counterexample. A failed runner or partial case count fails the gate.

KB29. Run the ten Rust upstream/regression tests, warning-free compilation, rustfmt and Clippy. Review source against every matrix row independently and retain review evidence. Port files keep upstream copyright; generated lowercase data retains Unicode notice. Changes to a table or assertion count require reviewed provenance, never silent expected-output updates.

KB30. Browser delivery is not established by the pure evaluator. Browser task33 must prove keydown routing priority Win32 before Kitty before legacy, repeat and release paths, custom-handler veto, composition/IME gating, PageUp/Down scrolling, SelectAll actions, key/cancel/data ordering, screen-reader branches, textarea resets and listener disposal. Actual Rust-backed Chromium/Firefox/WebKit scenarios and UTF-16 WebIDL transfer are required under criterion27. Task31 supplies the WASM bindings; headless evaluator tests cannot close that criterion.

## Reproduction and current status

```sh
node scripts/verify-keyboard.mjs
cargo test -p xterm-core --test keyboard_upstream --test keyboard_edges
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Task19 is independently accepted by Wallada. `results/reviews/wallada-keyboard-review.json` records her rerun of61 original tests,156 assertions,251452 differential cases and eight Rust tests in the initial snapshot. Task97 adds two directed boundary/precedence regressions; all ten current Rust tests pass in debug and release. This establishes module evidence only. Browser paths KB30, other native platforms and full terminal equivalence remain unproven. The detailed criteria linked to this matrix must not be replaced by a claim that a broad umbrella criterion is complete.
