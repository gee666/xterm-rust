# Browser integration progress

Status: unfinished. Full equivalence is NOT PROVEN. The current root integration has known compile and behavior gaps and is not ready for acceptance.

## Task 31

The owned browser manifest, DOM host, UTF-16 helpers and first actual three-browser assertion runner were committed in 697cc20. Engine-backed Terminal construction, queued writes, input, options, parser/buffer views, public event getters, addon management and a small browser facade have since been implemented in the shared tree. Many leaf modules have their own owner commits. The latest root integration is not yet a buildable checkpoint.

Remaining work includes replacing the provisional fixed-DOM-renderer routing with Cornelia's actual RenderService, wiring Norman's real Viewport/scrollable element and central scroll methods, completing early/late open phases and CoreBrowserService integration, and connecting all addon/render/decoration/accessibility lifecycle hooks. Several public bindings and raw JS receiver/coercion cases remain with their scoped owners. GC cycles are tracked separately in task181 and docs/wasm-ownership.md. None of these missing implementations is an external blocker.

The historical wasm_host run passed10 Rust assertions each in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The historical wasm_scheduler run passed3 each after a corrected unsupported-idle restoration test. These are component evidence only, and subsequent source changes mean they are not current whole-crate proof. Per-run records are retained under results/browser-host/.

The controlled acceptance-hooks check in results/build-slots/1793149945156-25500-c61e30eb-1103-4664-8871-594933fc8f1d failed with19 integration errors. Several have since been fixed in source, but no later green root check is claimed. Current source must be checked again after the RenderService/Viewport integration and queue API migrations.

## Task 118

Browser WriteScheduler now uses JS-owned callbacks and source global timer/microtask operations instead of eager cancellation on token drop. Work is incomplete. The latest portable queue contract adds fallible cancellation and retained ParseInputSource ownership; the browser adapter still needs its final cancel Result signature and ByteView dual-trait implementation, plus actual bounded lazy reads after async pauses. Explicit disposal must use the fallible path. Source request/cancel reentry, overwritten timer tokens, callback errors and ownership need fresh runtime tests against the JS oracle.

## Task 183

The native-event router has a distinct InputScroll branch. It refreshes the shared selection service and is intended to queue real viewport synchronization without firing public onScroll. The actual Viewport integration and queue_viewport_sync method are unfinished. ED3 tests must execute the built Rust terminal in all three browsers and compare private refresh/sync calls, cursor/buffers and absence of a public scroll event. Existing native erase traces are not browser proof.

## Evidence policy and next checks

Run scoped commands through scripts/build-slot.mjs with the shared cache/profile policy. The target cache relocation is documented in docs/target-relocation.md. The browser runner is being hardened to pin compiler input, retain post-bindgen executable bytes and hash the served WASM rather than the compiler's pre-bindgen module. Support-crate tests remain component evidence and cannot replace actual Terminal integration tests.

Next acceptance steps are a fresh warning-free root WASM check, actual source/Rust browser integration runs, user demo/e2e checks, and independent review of the current files and exact artifacts. Task lifecycle remains in_progress until that evidence exists; no finished work is being counted without review.
