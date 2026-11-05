# Browser input source mapping

Criterion 24: source_repo/src/browser/Clipboard.ts, source_repo/src/browser/Clipboard.test.ts, clipboard wiring in source_repo/src/browser/CoreBrowserTerminal.ts.

Criterion 25: source_repo/src/browser/input/CompositionHelper.ts, source_repo/src/browser/input/CompositionHelper.test.ts, composition wiring in source_repo/src/browser/CoreBrowserTerminal.ts.

Criterion 26: source_repo/src/browser/input/Mouse.ts, source_repo/src/browser/input/Mouse.test.ts, source_repo/src/browser/services/MouseCoordsService.ts, source_repo/src/browser/services/MouseService.ts, source_repo/src/browser/services/MouseService.test.ts, source_repo/src/browser/scrollable/touch.ts.

Criterion 27: source_repo/src/browser/services/KeyboardService.ts, input methods and bindings in source_repo/src/browser/CoreBrowserTerminal.ts. Core input encoders have separate owners.

The acceptance checklist is docs/browser-input-acceptance.md. These are source mappings, not claims of implemented coverage or passing evidence.

## Current input contributions

The criterion numbers above are historical. Active criterion471 and full-family task319 retain complete input/accessibility closure.

- CoreBrowserTerminal._inputEvent maps to input/keyboard_dispatch.rs and input/dom/keyboard.rs. Task356 adds admitted raw second-read delivery through the real core/browser data path. See docs/raw-input-data.md and results/raw-input356.
- CoreBrowserTerminal._keyPress and _isThirdLevelShift map to input/keyboard_dispatch.rs, input/keyboard_event.rs and input/dom/key_event.rs. Task378 preserves raw charCode/which and the keypress-only keyCode fallback, strict guards, delayed conversion, throws and reentry. See docs/keypress-raw-values.md and results/raw-keypress378.
- Task343 supplies the raw Kitty/Win32 lookup policies and realm adapters. See docs/keyboard-recovery343.md.

These scoped contributions do not close the full original input/accessibility suites, every decorated event field, native-platform proof or overall parity.
