# Browser keyboard dispatch

Task 94 owns the Rust state/effect ordering in CoreBrowserTerminal's `_keyDown`, `_keyUp`, `_keyPress`, `_inputEvent`, `_isThirdLevelShift` and `wasModifierKeyOnlyEvent`. Task 33 owns actual DOM bindings and browser tests. The shared encoding policies remain separate.

## Acceptance paths

1. Keydown clears handled and marks seen before custom veto. Veto skips composition and every effect. Mac Option-as-Meta with Alt bypasses composition and dead-key tracking. Composition rejection optionally scrolls to bottom only when configured and scrolled away.
2. Dead/AltGraph mark pending state before encoding. Cursor-style update follows encoding. PageUp/Down scroll rows minus one and cancel before returning false. SelectAll precedes third-level bypass. Encoder cancellation precedes missing-key/uppercase/dead-key exits and may be repeated after a sent key.
3. Third-level uses Mac Alt without Ctrl/Meta and without option-as-meta, Windows Alt+Ctrl without Meta, or Windows AltGraph. Keypress bypasses keyCode filtering; other event categories require zero or greater than 47. Check 0, 16, 37, 47, 48, 81 and all modifier combinations.
4. Uppercase A-Z defers to keypress only for legacy mode without Ctrl/Alt/Meta. Kitty/Win32 skip that workaround. Pending dead key suppresses exactly the next encoded nonempty key and clears before return. Enter and Ctrl-C clear textarea before onKey.
5. onKey precedes showCursor then triggerData. Only Win32 modifier-only events use user_input=false. The modifier test includes codes 16/17/18/91/92/93/224 and name Meta. Screen-reader mode leaves handled=true and undefined return only when Alt/Ctrl are absent; other paths cancel and return false.
6. Keyup clears seen before veto. Nonmodifier focus precedes release evaluation. Only nonempty releases emit data, without onKey/showCursor. Cursor-style update and keypress-handled reset happen after data. Veto and callback errors retain exactly the partial source state.
7. Keypress clears its handled flag before checking keydown-handled and custom veto. Character selection preserves absent versus zero charCode/which and String.fromCharCode modulo-65536 semantics. Alt/Ctrl/Meta require third-level. Successful onKey/showCursor/data precede handled=true and dead=false.
8. Input requires nonempty data, insertText, not composed while keydown seen, screen-reader disabled and keypress not handled. Clear dead state before data, preserve UTF-16 units and emit no onKey/cursor/cancel effects.
9. Every callback/effect error propagates immediately. No later state/effect is committed after failure. Synchronous reentry must not encounter a held dispatcher or adapter RefCell borrow. Snapshot options/modes again after callbacks because source reads live values.
10. Differential checks compare ordered effects, tri-state keydown returns and all four state fields on deterministic and seeded mixed event sequences. Preserve counterexamples and exact source hashes/counts. Replay source third-level/custom-handler/keypress expected cases in Rust. Real browser event dispatch, disabled-stdin effects inside CoreService, focus and disposal still require task 33 integration evidence.

## Contracts

`KeyboardDispatch` stores four booleans in Cells and all entrypoints take `&self`. `KeyboardReadHost` provides fallible fine-grained reads; `KeyboardHost` adds shared-reference hooks and fallible synchronous actions. An adapter must release any RefCell borrow before invoking custom handlers, composition, onKey or core data callbacks. Action payloads are borrowed only during that call. Callback errors stop dispatch and retain source partial state.

The dispatcher invokes individual read methods at the source's live read points, including short-circuiting getters that can throw. Defaults derive values from `KeyboardSnapshot` for immutable test models; the actual DOM adapter overrides each individual read rather than eagerly taking one snapshot. Its `is_mac` is the browser-service override; `modes.is_mac` is the encoding platform imported by KeyboardService. These can differ in upstream tests. `DispatchKeyEvent` carries DOM category, repeat, AltGraph, charCode and which beside the existing UTF-16 protocol event. `TextInputEvent` represents data/inputType/composed. DOM WebIDL conversion remains the adapter's responsibility; numeric fields use unsigned-long values.

## Evidence

Observed on Windows x86-64, Node 24.15.0 and rustc 1.85.1:

```sh
cargo test -p xterm-web --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters
node scripts/keyboard-dispatch-oracle.mjs
node scripts/keyboard-dispatch-oracle.mjs --focused
```

A full Cargo run passed all 14 Rust tests and 165,328 ordered source-method comparisons with seed 1145656144, hexadecimal 0x44495350. `results/keyboard-dispatch/oracle.json` records that Cargo-built oracle result. Later unrelated shared edits again prevented a complete Cargo rebuild; `rust-tests.log` and `build-failure.log` record the latest attempt, not a current green workspace.

The approved `--focused` mode compiles the actual core keyboard/Kitty/Win32 and web keyboard/dispatch source modules through small rustc wrappers under root `tmp/`. It adjusts only Kitty's two submodule path declarations in a temporary copy because Rust's #[path] inclusion changes submodule resolution. No behavior or assertion changes. It builds the already-declared ryu-js dependency in the shared Cargo target, creates no second Cargo target and removes its own binaries/rlibs afterward. This mode independently passes the current 14 tests, Clippy-driver with all lints and warnings denied on the web policy/tests, and the same 165,328 comparisons. It does not compile unrelated terminal/addon/browser modules. `focused-commands.json`, `focused-build.log`, `focused-tests.log`, `focused-differential.log` and `focused-oracle.json` retain exact commands and scope. Set CLIPPY_DRIVER to select an explicit Clippy driver; RYU_JS_RLIB disambiguates multiple cached ryu-js rlibs if needed.

The source-derived corpus has 10,400 operations in each named keydown, following keypress, following input and following keyup category; 36,864 third-level keypress boundaries; 36,864 third-level keydown boundaries; and 50,000 seeded mixed operations. It compares every return, all four state flags and all ordered effects. Original and production module hashes appear in the reports. Failed comparisons retain up to 20 counterexamples with preceding operations; none occurred. Historical build failures remain retained after successful runs.

Rust source-case mapping includes Terminal.test.ts's custom-handler six return assertions, both Mac/Windows printable @/backslash/pipe event families, and additional explicit tests for all dispatcher routes, UTF-16 modulo, throwing getters, partial callback failures and synchronous reentry. This runner does not execute the original complete Terminal browser suite. It records originalBrowserSuite.executed=0 instead of presenting source-method traces as upstream-suite execution. Terminal reset, actual DOM onKey event identity, CoreService disabled-stdin/scroll composition and full browser suites remain owned by the engine/browser integration tasks.

Focused rustfmt passes. Full Cargo Clippy attempts currently stop in unrelated headless/move-to-cell work, retained in `clippy.log`; that is not a passing workspace gate. Native Linux/macOS and actual Chromium/Firefox/WebKit input/disposal tests remain unrun here. These results do not establish a complete working Rust-backed browser terminal or full equivalence.
