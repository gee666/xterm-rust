# Rendering task 32 progress

Task 32 is not complete. Overall rendering equivalence is NOT PROVEN. This file records the remaining work, rather than treating implemented modules or earlier component checks as acceptance.

## Retained implementation

- Core Color task 4 was independently accepted after 350 original assertions, 32 Rust cases and 28,738 differential operations. The later raw UTF-16 additions belong to task 170 and need their own evidence.
- Theme palettes, tri-state contrast caching, shared ColorRef restoration, selection blending, raw theme parsing and live getter commit ordering are implemented. Native theme/cache checks passed before the later raw-path patch. Task 170, now Octavia-owned, covers raw CSS/font coercion and original error ordering. Max's task 230 owns the shared callback-safe width driver.
- Native row planning and actual WASM span construction are implemented in `renderer/row*.rs`. An unchanged-source runner passed all 38 original RowFactory cases and captured 565 row calls. The Rust three-browser replay is written but has not yet demonstrated a current pass. CharacterJoinerService's 24 original cases passed and generated 27 captured calls; its new Rust replay also still needs execution and independent review.
- Actual DOM rendering, owner-window/dpr/focus injection, selection geometry, link underlining and cursor/text blink lifecycle are implemented under `renderer/dom/`. These are real Rust implementations calling DOM APIs, not a JavaScript terminal engine or a headless substitute.
- Actual RenderService scheduling, backend replacement, synchronized-output buffering and safety timeout, visibility observation, paused idle resizing, redraw-only event distinctions and disposal are implemented in `renderer/service*.rs`. The new implementation is not independently reviewed or proven by a passing current runtime suite.

## Remaining implementation and integration

1. Complete task 230's width-cache API migration in RowFactory and DOM callers. Release every cache guard before host coercion/measurement callbacks and preserve source constructor/options ordering.
2. Complete task 170's lazy raw CSS/font path and its boxed-string, array-hole, property getter, coercion and reentrancy followups. Do not substitute native well-formed String helpers for JS-facing behavior.
3. Finish and verify shared CoreBrowserService injection and RenderService integration in Xavier's task 31. The root must not retain its provisional duplicate debouncer/event routing. Backend setter aliases for image/WebGL must use the actual facade, not unconditional native pre/post hooks.
4. Verify current-buffer option/geometry reads and reentrant rendering against the source. The current native geometry helper alone is not proof of live getter or callback ordering.
5. Integrate Isaac's actual BufferDecorationRenderer DOM/microtask work, finish OverviewRulerRenderer and verify ColorZoneStore through it. Source render callbacks must receive real owned elements. ColorZoneStore task 146 still needs independent acceptance.
6. Run the complete RowFactory, CharacterJoinerService, ThemeService, WidthCache and renderer-support assertion mappings, source-differential traces, directed error/disposal/reentrancy cases and seeded cases. Run actual Chromium, Firefox and WebKit component and terminal tests, then the working demo/e2e matrix. Retain screenshots and pixel/computed-style assertions where required by criteria 189 and 192-199.
7. Obtain independent code and evidence review; fix current fmt/Clippy/build failures rather than suppressing them. Criteria 412 and 433 require the canonical stylesheet matrix and correct public dimensions snapshots. The mistaken live-public-dimensions claim was retracted after reading CoreBrowserTerminal's copying getter.

## Current execution evidence and failures

Controlled builds use `scripts/build-slot.mjs`, the unchanged shared profile settings and the single relocated target store. Logs are retained under `results/build-slots/`.

- `1793171497199-45640-de2e71b3-3a26-411a-9c36-31b542492f00` reached a core NativeWriteScheduler cancellation-signature migration failure.
- `1793171805947-49268-d1ab7ddb-36d3-4dc7-a094-7367c8d3af22` reached a concurrent SortedList API migration failure.
- `1793172337573-15920-f210f74a-17da-43a2-9a61-56210bd90eae` reached current width-driver migration, write scheduler and root assembly errors. The relevant owners were notified. These are unfinished integration work, not external blockers or successful renderer checks.

No request for final review of task 32 is justified yet. Cornelia retains active ownership of RenderService, DOM integration and the associated tests. Scoped tasks have concrete owners and are not silently deferred.
