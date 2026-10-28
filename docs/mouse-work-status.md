# Mouse work status

Both tasks remain unfinished and are released for a fresh integration validator. Implementation is committed through0ec2973, with checker simplification69e5055. Neither task is marked accepted or externally blocked.

## Task110

Listener ownership, global drag/release cleanup, Alt subscriptions, option/protocol subscriptions, reentrant fallible DOM callbacks and gesture consumption are implemented. Main implementation checkpoint is33a54f3; live-reader checkpoint aa6b160 follows it. Later review corrections are in the shared working tree and will be committed with their evidence.

Remaining work is actual Rust WASM runtime validation in Chromium, Firefox and WebKit, targeted warning-denied Clippy and independent review. The initial seven listener/lifecycle cases are included in the expanded planned24-test wasm_mouse target. Xavier reports exported Terminal reset/RIS now call reset_browser_state/reset_mouse before selection/decorations/draw, but actual public/RIS tests remain unrun here. The real raw mouse-service callback receiver remains task171; neither is proven by this component.

## Task172

Fine-grained required host readers, caught event/DOM geometry operations, source-order raw sensitivity operands, captured-old touch subtraction and primitive numeric button preservation are implemented. The new MouseButton::RawNumber preserves fractional/NaN comparisons and delays ToInt32 to bit encoding. A26-row JavaScript numeric fixture is generated and verified. Its Rust replay now passes in managed native slot1793172445611-32312-edb78234-c1e6-42b6-bced-0d69f74398cd.

The unchanged-source observer script executes10 directed method traces and rejects fixture drift. Those source-only checks pass. The planned WASM target has24 tests after source review added operand-order, touch reentry, numeric range/debounce and geometry method/getter cases. No browser pass is claimed. Raw non-number event identity/prototype compatibility remains task171, not a parity waiver.

Remaining work is compile/type-check current browser changes, execute all24 WASM tests in all three engines, fix actual failures, run targeted Clippy, refresh stable source/artifact evidence and obtain independent review. Static review findings are retained in docs/reviews/mouse-live-read-findings.md; correction tests are in tests/cases/mouse_operands.rs.

## Latest observed execution

- Source-only `node scripts/mouse-live-oracle.mjs`:10 source traces and26 primitive-number expectations generated/verified. Results/mouse-live/source.json records exact source hashes.
- Native numeric lease1793171556452-48420-1ca5f616-903a-4e4e-84d0-9ca7d84e29e3 failed compilation in the then-stale NativeWriteScheduler.cancel adapter. That adapter has since changed; no later native pass is asserted.
- Native numeric lease1793172445611-32312-edb78234-c1e6-42b6-bced-0d69f74398cd passed one Rust test covering all26 generated numeric expectations.
- WASM lease1793172445669-42664-5245da78-602c-44a4-87ef-0cd64f2baca0 failed current renderer WidthMeasurer, browser WriteScheduler.cancel and viewport service API callers. Browser tests remain unrun; no image-error failure is counted as current after that code changed.
- WASM lease1793171964414-15092-785db59d-a84d-41af-91b9-8890959be1f0 reached source stage success but failed current image/storage render/resize Result callers. Browser tests were unrun. Immutable build logs are under results/browser-host/wasm_mouse/2026-10-28T07-19-24-602Z/.
- The controlled target relocation restored disk capacity while preserving the logical shared target and profiles. All further compiler/browser commands use build-slot.mjs. No source, private state or failed evidence was deleted.

Next action is a scoped current native/WASM lease after the shared image caller fix, then resolve this module's actual compile/runtime failures. Missing implementation and unrun validation are work, not external blockers. Full equivalence remains NOT PROVEN.
