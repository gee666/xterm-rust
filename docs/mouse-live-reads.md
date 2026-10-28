# Live mouse reads

Task #172 closes the eager MouseSnapshot/PointerInput gap left by task #110. Source is `source_repo/src/browser/services/MouseService.ts`, `source_repo/src/browser/services/MouseCoordsService.ts` and `source_repo/src/browser/input/Mouse.ts`. Task #110 remains listener ownership and normal typed-event routing; criterion26 cannot be satisfied until both tasks and the touch/mode/coordinator tasks pass.

## Shared contract

`MouseReadHost` exposes fine-grained fallible reads. Its aggregate snapshot defaults are only for immutable test models. The real EngineInputHost must override every fine-grained read. This prevents an irrelevant wheel option getter from running during an inactive protocol change, pointer event or zero wheel path. The existing actual terminal mouse service is still shared; no second service is introduced.

The DOM adapter reads event properties lazily through Reflect::get, catching thrown values as Result rather than allowing a JavaScript exception to skip Rust destructors. Cancellation and getModifierState calls also use caught Reflect::apply. Minimal static JavaScript bindings perform only primitive numeric coercion, bitwise masks and comparisons whose Symbol/BigInt/valueOf exception semantics cannot be reproduced by an unchecked Rust cast. Routing, protocol decisions, accumulation, coordinates and state remain Rust. No JavaScript terminal engine is called and no runtime eval or Function constructor is needed.

Task85's wheel accumulator exposes a shared accumulate_pixels operation so the DOM adapter can preserve getter timing while reusing the exact partial-scroll formula. There must not be a copied second accumulator or a second trackpad threshold constant.

## Acceptance paths

1. Geometry samples relative coordinates before valid-character-size rejection, then samples canvas and cell metrics only when valid. Native field getters use their original relative order. Failures stop before later metrics/events.
2. Read overrideType before type, and type only when overrideType is falsy. Unknown kinds return before buttons, modifiers or wheel callbacks. Preserve missing buttons versus zero; only read fallback button when needed. Repeated bit-mask branches read the actual current buttons property.
3. Wheel report coordinates are checked before custom-wheel veto. Veto returns before delta/modifier/sensitivity reads. The initial strict numeric-zero delta test occurs before the consume function. Its height/DPR argument reads occur before its own delta/Shift short circuit.
4. Consume reads delta and Shift in source order, reads only the selected fast/normal option branch, then deltaMode and the trackpad delta where required. Native wheel callback mutations are observed by subsequent reads. Pixel partial state commits before subsequent direction/modifier/output errors, as source does.
5. Non-wheel Alt gating short-circuits on requireAlt, active protocol and actual Alt. The later strip-Alt computation samples source conditions again. Copy final ctrl/alt/shift only after gating. Wheel preserves Alt; local gating strips only non-wheel Alt.
6. Trigger range checks read cols/rows only at their source branch points before delegating shared validation/encoding/debounce. No borrow crosses restrictions, encoders, output or custom callbacks. Preserve cache commit after output and mutation visibility.
7. Passive wheel reads no custom callback while active wheel reporting is requested. Its veto precedes scrollback/metrics. Normal scrollback reads no cursor mode. A zero amount still cancels in the alternate buffer but emits no data.
8. Touch checks requested wheel before scrollback, cell height before accumulation and coordinates only after whole lines exist. Each wheel output uses live bounds. Cursor sequence is computed once before repeated key output, as source does.
9. Alt controller calls the actual getModifierState method with the actual event receiver. Its false branch, getter errors and noncallable errors do not become a successful no-op. Source callback exceptions retain identity through EmitError::Callback and std::error::Error sources.
10. Actual Chromium/Firefox/WebKit regressions must cover custom-wheel mutations, irrelevant and throwing getters, early veto/zero paths, cancellation failure ordering, reentrant dispatch and partial state. Compare source-method ordered read/effect traces where practical; do not count a typed snapshot model as proof of live DOM behavior.

## Evidence

The planned WASM target now contains seven listener/lifecycle cases, nine additional live-read/coercion cases and one ten-case source-trace replay test. The latter assert unused wheel button fields, truthy override/type short-circuit, zero/veto metric suppression, post-callback three-read delta mutation, missing-buttons fallback order, unrelated host getter suppression and thrown-value identity through the DOM error event. Numeric valueOf cases also check protocol-change reentry and the exact sensitivity-read point before a coercion exception. The test host forbids aggregate snapshot reads, so a return to eager access cannot silently pass.

`node scripts/mouse-live-oracle.mjs` now executes ten directed scenarios through unchanged source methods and verifies their ordered event/host/effect fixture. It records originalSuitesExecuted=0; the complete15-test original baseline is task85. Metadata groups plain canvas/cell values to the Rust host read methods rather than claiming arbitrary render-object proxy parity.

No Rust build or browser runtime pass is claimed yet. Use the controlled build queue from docs/build-slots.md after resource recovery:

```
node scripts/build-slot.mjs --owner NAME --lane wasm --wait-ms 600000 --timeout-ms 300000 -- node scripts/check-mouse-live.mjs
```

The scoped checker records source/fixture and Cargo/profile hashes before and after its build, rejects changed component snapshots and links the immutable browser artifact report. It forces all three browser engines rather than accepting a filtered single-engine run. Rerun the component suite and new live-read suite with the existing browser-host-check skill, full relevant source-policy oracles, targeted Clippy and independent review. Keep exact counts, seeds, current source/implementation hashes and failing traces. Native Linux/macOS, complete Terminal/browser interaction and actual touch recognition remain separate requirements.
