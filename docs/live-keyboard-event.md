# Live keyboard event reads

Task 151 closes the actual DOM event snapshot gap left outside task 94's immutable typed-event model. Custom handlers, composition, onKey callbacks and property getters may mutate an event or throw. Reading key/code/modifiers before custom veto is observably wrong. Moving one eager snapshot after the custom callback is still wrong because the source may never read a particular field.

## Required behavior

- Enter keydown resets handled and sets seen before custom handler invocation without reading any DOM key/code/modifier field. A veto returns immediately, even if every event getter throws.
- Read each field only at its source expression and preserve short-circuit order. Do not cache values across callbacks or repeated source getter expressions. A throwing getter must retain its original JsValue identity through report_error and must prevent all later effects.
- Composition keydown reads keyCode only when composition is invoked. Mac Option-as-Meta can bypass composition without reading keyCode there. Dead/AltGraph detection follows composition and may read an event changed by it.
- Encoding must use live event reads too. A legacy Tab/Enter path must not read an unused code getter. Kitty/Win32/legacy fallback and their repeated modifier/key/code reads must match original encoder order, not a browser-side pre-snapshot.
- Cursor-style/selection/scroll/onKey callbacks may mutate later-read event fields. Uppercase workaround, third-level shift, modifier-only user-input flag and final Alt/Ctrl cancellation use the values that exist at those reads.
- Keypress clears its handled state before checking prior keydown-handled. If already handled, it must not read charCode/which/keyCode or invoke the custom handler. Otherwise preserve charCode truthiness, null/undefined which fallback, numeric zero exclusion, modulo UTF-16 conversion and later modifier reads.
- Keyup clears seen before veto; focus occurs before release encoding and later event reads. Modifier-only checks retain repeated keyCode comparisons and the final Meta key-name check.
- Separate task177 must make InputEvent data/inputType/composed getters lazy and source ordered, including nullable second reads and CoreService payload semantics. It is not completed by this keyboard-only refactor.
- Real DOM tests must use Object.defineProperty getters, custom-handler property replacement and synchronous nested dispatch. Compare exact ordered reads, data/effects/return state and error identity against source. Native/static-event regression suites must remain unchanged and green.

## Shared encoder contract, being coordinated

Core remains independent of browser and OS. The fallible KeyboardEventSource trait exposes key/code as borrowed-or-owned UTF-16 data and each numeric/modifier field through Result. Numeric key codes use f64 to preserve fractions, NaN and infinities until source operations. Existing native functions keep their signatures and use an infallible borrowed adapter. New fallible evaluator entrypoints must perform source-ordered getter calls without changing the existing policy for immutable events.

Victor owns the trait, Kitty/Win32 live policy refactors, dispatcher flow, DOM keyboard adapter and regressions. Boleslav owns legacy refactor155. Norman retains engine InputHost services, IME source-read phases and runtime wiring. Tasks184/185 separately close Win32 inherited-object lookup/coercion gaps. Task202 covers dynamically decorated fields outside declared DOM primitive types, which the current DOM adapter validates instead of fully reproducing every JS coercion. Changes to accepted input policies require focused and full differential reruns; preserved source is never edited.

## Evidence status

The source traits, generic Kitty/Win32 policies, lazy dispatcher and DOM adapter are written. Twelve real DOM tests cover custom veto, property replacement, throwing getters, type reads after third-level checks, live onKey cancellation, null/undefined which reads, Kitty unused-field suppression, Win32 repeated code reads, active IME, recursive custom dispatch and keyboard events from an adopted same-origin document. The first managed WASM build stopped on unrelated renderer/service/utility integration errors, so these assertions remain unexecuted. Owners were notified; the work is not marked externally blocked.

The source-only Kitty/Win32 oracle has executed 4,521 successful paths and every reached getter failure boundary, 74,171 injected throws. The reviewed fixture has 78,692 rows, with source/fixture hashes in results/protocol-live/oracle.json. The Rust replay passed all 78,692 rows in the managed native slot at results/build-slots/1793149821671-29876-53e209ee-9d6d-41cc-9779-95a7a44427c7, alongside five Kitty edge tests and two faithful fixture tests. The replay subsequently shares its Source/Sequence mock with legacy task155; that test-only refactor still needs a fresh run. The generated 78,692-row fixture intentionally exceeds 350 lines to retain every complete trace and failure boundary.

The final generic dispatcher passed all 14 existing Rust state/effect tests at results/build-slots/1793150026739-39584-84b3bc72-e5a8-4219-86d2-7cb8fc66bda6. The unchanged Win32 suite and 13,904 ordinary differential cases passed at results/build-slots/1793149970440-34500-98cee0ae-3920-40c5-8da9-a28517b7a4f8. The post-refactor 508,249-case Kitty rerun stopped on the then-incomplete buffer shared-helper export, retained in results/build-slots/1793149930125-47944-0d76c61a-f654-4373-84c4-f857724f10e9; it is not a pass. Clippy was stopped before launching by the 1 GiB disk reserve at results/build-slots/1793150198142-2504-9273a674-f2e6-48a5-b57b-fa63f1cf3812. Task94's 165,328-case immutable dispatcher differential must also be rerun after this refactor.

Actual browser runtime evidence is pending. The failed WASM build is results/browser-host/wasm_keyboard_live/2026-10-28T01-14-30-136Z/wasm_keyboard_live-build.log. A WASM compile alone will not count as a browser pass.

Typed string/number field validation is not proof of arbitrary JS value/coercion parity. Tasks177,184,185,202 retain those concrete work gaps rather than marking them externally blocked. Full browser integration and equivalence remain NOT PROVEN.
