# Diponegoro input continuation

## Accepted scoped work

Task292 fixes real DOM InputEvent lazy reads. `TextInputEventSource` and `KeyboardDispatch::input_source` share the source-ordered guard policy with native immutable callers. The first data read tests truthiness only; admitted events clear dead-key state before rereading exact text. Reflect exceptions and synchronous getter reentry are preserved. See docs/input-event-dispatch.md and results/input-event/README.md.

Fresh checks passed20 native tests, both source-fixture integrity checks, InputEvent4/input7/callback3/keyboard12 test functions per Chromium/Firefox/WebKit, plus scoped native/WASM warning-denied Clippy and changed-file rustfmt. The InputEvent guard test expands288 combinations per engine against unchanged source. A restored old DOM binder failed3/4 new tests per engine before the fixed binder passed4/4. Independent review accepted292 after rerunning source2/native20/InputEvent4x3.

Two port-test fixtures were corrected with source/browser evidence, not relaxed. Firefox's synthetic ClipboardEvent constructor discards DataTransfer initializer content, so the helper supplies the prescribed clipboardData property explicitly. The changing230-to-Shift IME case now compares the unchanged CompositionHelper.keydown body and asserts its actual three reads instead of a mistaken literal two. Both original method fixtures have source-integrity checks and retained upstream notices.

## Unfinished input work

Task319/criterion471 still require complete input/accessibility behavior and unchanged full suites. A specific pending case is an admitted InputEvent whose second data read returns null, undefined, an object or another non-string. Upstream passes that raw value through CoreService, but the Rust browser/core contract currently expects UTF-16 text. Do not coerce raw values to empty text. Preserve disableStdin, scroll, user-input, log, callback and exception ordering when extending this boundary.

Task343 is a concrete recovery opportunity for the next input owner. Retained commits `swarm/Aisha`6fd9fa9 and16b12b2 implement raw Kitty/Win32 map semantics and per-realm adapters. Neither commit is an ancestor of current integrated main, and main lacks the production input/keyboard_lookup and corresponding raw realm modules. Check patch equivalence and current code before copying anything.

- `6fd9fa9` includes default-hint conversion, inherited/raw Kitty lookups, generic native policies, generated realm table adapters, DOM tests and negative controls. Its81-file patch includes substantial existing evidence; do not replay unrelated old branch history.
- `16b12b2` implements source-ordered raw Win32 map conversion and DOM lookup adapters. Later Aisha commits1922c37/21d23f9 provide source/cross-browser observations and a6f69be contains the test-host fixes already recovered by292.
- Read the retained docs with `git show swarm/Aisha:docs/kitty-realm-lookup.md`, `docs/keyboard-map-lookup.md`, `docs/kitty-verification.md` and relevant result reports before deciding what to recover. Earlier build-slot instructions are retired, not prerequisites.
- Keep the new InputEvent policy and fixture fixes when adapting old keyboard.rs/key_event.rs hunks. The old branch predates several real main integrations and must not replace those files wholesale.

## Integration and cleanup

The accepted producer is preserved on archive/Diponegoro16-reviewed. Its canonical batch also includes owner-confirmed accepted search290, wheel334 and CI-recovery336 commits, avoiding separate cold gates. Final gate status and exact integrated revision must be read from the canonical receipt rather than inferred from this planning note.

New temporary work is owned O:/xterm-rust-swarm-16/tmp/worktrees/Diponegoro16 only, including its regular target, tmp and physical reference copy. No dependency junction was created. Preserve committed source/evidence and inspect dirty/untracked files, process ownership and reparse points before removing this new worktree. Do not delete the original reference or historical worktrees. No Windows/Linux/macOS final matrix or full equivalence claim is made here.
