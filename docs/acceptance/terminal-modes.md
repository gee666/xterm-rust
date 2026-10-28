# Terminal mode dispatch acceptance

Task 102 ports `source_repo/src/common/InputHandler.ts` methods `setMode`, `resetMode`, `setModePrivate`, `resetModePrivate`, `requestMode`, `kittyKeyboardSet`, `kittyKeyboardQuery`, `kittyKeyboardPush`, and `kittyKeyboardPop`. Reference assertions are in `source_repo/src/common/InputHandler.test.ts` under setMode, DECRQM, alt buffer/cursor, and Kitty keyboard. Core native execution is not browser callback-order proof.

## Standard and private transitions

Set/reset process every main parameter in order and ignore subparameters as the source does. Unknown parameters have no side effects and do not prevent later parameters from being applied. Cover repeated parameters, empty/default/zero, unsupported values, multiple parameters and every stream split.

| Mode | Required action and edge checks |
| --- | --- |
| ANSI 4 | Toggle live insert mode; initial false; setting twice is stable |
| ANSI 20 | Set convertEol through OptionsService, retaining sanitization, strict-equality notifications and failures |
| DEC 1 | Application cursor keys toggle |
| DEC 2 | Set only: designate default charset in G0..G3, preserving active invocation; reset is ignored |
| DEC 3 | Only when windowOptions.setWinLines is truthy: resize to132 on set,80 on reset with existing rows, then request reset; verify callback-time dimensions/error state |
| DEC 6 | Toggle origin before moving cursor to0,0 using origin-aware bounds |
| DEC 7 | Wraparound toggle, initial true |
| DEC 12 | Only mutate cursorBlink option when quirks.allowSetCursorBlink is truthy; query live cursorBlink regardless of quirk |
| DEC 25 | Set shows cursor, reset hides; no inversion mistakes |
| DEC 45 | Reverse-wraparound toggle |
| DEC 66 | Emit exact source debug log before application-keypad mutation, then request scrollbar sync |
| DEC 9/1000/1002/1003 | Select X10/VT200/DRAG/ANY mutually exclusively; reset any of them selects NONE even when another protocol was active; protocol-change callbacks receive each activation |
| DEC 1004 | Toggle send-focus; request current focus only on set, including repeated set |
| DEC 1005/1015 | Remain unsupported; preserve exact DECSET/DECRST debug messages rather than inventing UTF8/urxvt encoding |
| DEC 1006/1016 | Select SGR/SGR_PIXELS mutually exclusively; reset either selects DEFAULT regardless of currently active encoding |
| DEC 1048 | Save/restore cursor, attributes, charset designations/invocation and saved origin/wrap modes without changing active buffer |
| DEC 47/1047/1049 | Source buffer activation ordering, Kitty flag swap, cursor initialization, full refresh and scrollbar sync.1049 saves before entering alt and restores after leaving.47/1047 retain current attributes. Existing alt contents/saved cursor semantics and erase background must match, not a simplified clear-every-time implementation |
| DEC 2004 | Bracketed paste toggle |
| DEC 2026 | Synchronized-output toggle; reset requests full refresh even if already reset |
| DEC 2031 | Color-scheme updates toggle unless colorSchemeQuery is explicitly falsy; missing/null property defaults true |
| DEC 9001 | Win32-input mode toggle only with truthy vtExtensions.win32InputMode |

Optional vtExtensions/quirks bags may be null/undefined without throwing. windowOptions uses direct property access and invalid null/undefined must preserve a recoverable property-access error. Structured getters can throw and reenter services; do not hold a mode RefCell borrow across them. Failures stop subsequent parameters and retain already committed changes.

## DECRQM exact replies

Only the first parameter is queried. A reply is exactly `ESC[Ps;Pm$y` for ANSI and `ESC[?Ps;Pm$y` for DEC. Pm meanings are0 unknown,1 set,2 reset,3 permanently set,4 permanently reset. Queries use the CoreService data path, respecting disableStdin and preserving event order.

- ANSI2 always4;4 follows insert;12 always3;20 follows convertEol; everything else0.
- DEC1,6,7,12,25,45,66,1004,2004,2026 follow their live states.25 is inverse cursor-hidden.
- DEC3 is0 when disabled or width neither80 nor132; otherwise80 gives2,132 gives1.
- DEC8 always3;67/1005/1015 always4;1048 always1.
- DEC9/1000/1002/1003 compare exact active protocol;1006/1016 exact active encoding. Inactive mutually exclusive values report2.
- DEC47/1047/1049 all report whether the actual alternate buffer identity is active.
- DEC9001 is0 when extension disabled, otherwise current mode1/2.
- Unsupported DEC2031 query remains0 despite the supported setter. Do not infer query support from a setter.

Run all eight original DECRQM cases with their inner mode loops intact, including quirk/permanent behavior. Compare exact UTF16 reply units and full state, not merely a boolean snapshot.

## Kitty negotiation

- With kittyKeyboard disabled or absent, all four handlers are handled but leave flags/stacks untouched and emit no query reply. This is the source behavior, not a placeholder for an enabled feature.
- `CSI = flags;mode u`: absent/zero mode defaults1;1 replaces,2 ORs specified flags,3 AND-NOT clears; other modes do nothing. Main parameters saturate per Params, not an ad-hoc parser.
- `CSI ? u` replies `ESC[?flagsu`, with no extra delimiter, through the CoreService data path.
- `CSI > flags u` chooses the stack by live buffer identity. At16 entries remove exactly the oldest, push current flags, then replace flags. Repeated switches preserve separate main/alt flags and stacks, including repeated activation of an already active buffer.
- `CSI < count u` defaults/clamps count to at least1; pops at most available stack entries, assigning each popped flag. If the stack is empty afterward, flags become0 even when the last popped value was nonzero. Large hostile counts perform at most16 iterations without altering the result.
- Port the three original Kitty stack-limit/buffer-switch/pop cases and add replace/OR/clear/invalidmode/zero/default/overpop/chunking/disabled checks.

## Integration and evidence

All six original alt-buffer/cursor cases remain intact, including SGR colors, ANSI save/restore and erase-background assertions. If another missing handler causes a failure, retain it and integrate that handler rather than weakening the assertion. `tests/terminal_modes.rs` and `terminal_alt_modes.rs` execute actual CoreTerminal streaming input, not direct helper calls or a second engine.

Record original discovered/mapped/executed/passed/failed/skipped/unrun counts, exact commands/tool versions, source hashes and deterministic trace seeds under `results/terminal-modes/`. Independent review must inspect source branches and run full native mode tests. The public headless and browser terminals subscribe to request-reset. Mode3 reset and browser focus/scrollbar/refresh callbacks must run with original synchronous timing in their integrations; a deferred event queue is not proof of that contract. Keep that integration gap explicit until actual browser traces pass.
