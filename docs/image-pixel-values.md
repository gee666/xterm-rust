# Raw image pixel limits

Task539 repairs `pixelLimit` on the browser ImageAddon. The constructor copies options twice without converting the value. Its first copy remains authoritative through reset and disposal. The second copy still runs with its original getters and exception precedence, but reset never restores `pixelLimit` from it.

Each consumer evaluates its own source expression.

- Graphics READ compares the captured canvas area with the raw limit. If rejected, it evaluates `Math.floor(Math.sqrt(limit))`, converting again. READ_MAX evaluates only the square expression. Other controls do not read the limit.
- Enabled SIXEL evaluates `limit * 4` synchronously during handler construction, before async decoder loading and registration. The resulting memory number is captured once. A throwing multiplication leaves prior parent registrations intact and skips SIXEL, IIP and Kitty registration. Disabled SIXEL does not convert the value.
- IIP compares original dimensions only after successful BASE64 finalization and supported-image metadata with nonzero dimensions. It evaluates the raw limit again after resize and flooring, only for nonzero output dimensions. First-check rejection warns before releasing the live payload. Second-check rejection releases without a warning. A thrown comparison is not a caught bitmap error and retains the payload.
- Kitty checks placement dimensions after bitmap decode/crop and geometry. It samples the current command's layer after that comparison and saved-cursor reads, before resize awaits. If an offset bitmap is created, its actual dimensions are checked again after replacement and old-bitmap cleanup. Relational exceptions use the source's caught display-failure path. Current bitmap ownership and cleanup remain unchanged.

Relational comparisons intentionally support BigInt. SIXEL multiplication and graphics square conversion intentionally reject BigInt. A NaN limit rejects IIP's less-than checks but accepts Kitty's greater-than checks. No single numeric projection can preserve these contracts.

Rust owns decoding, geometry, storage, parser dispatch and resource cleanup. Small JS expressions perform language conversions and preserve exact host exceptions. Native IIP tokens separate original/resize checks from their continuations, retaining captured dimensions but reading the live header or decoder after callbacks. Tokens reject foreign controller ownership before mutations. Existing numeric IIP and Kitty convenience APIs compose the same stages.

The source producer is based directly on main `0442e357`, without private prerequisites or a copy of528. Future integration with528 must preserve its palette conversion after the synchronous pixel multiplication and before asynchronous initialization. Complete writable private option/prototype state, full original image639/img01..48 and GC remain311/324, not claimed by this task.

## Current verification

The first native checkpoint passes16 phase tests, including three new ownership/reentry tests, eleven retained IIP tests and two Kitty geometry tests. Full native addon execution passes213 functions in51 result summaries. Native addon all-target/all-feature Clippy and actual WASM library/image-addon-test Clippy pass with warnings denied at that checkpoint. Later source review added late Kitty layer sampling after the raw comparison and a native mutation test. That update and the final shipped-protocol browser test await a current compile and runtime gate.

The first native compile failed for a missing import after splitting the cohesive end module. The next execution caught an incorrect new-test assumption about nested BASE64 payload visibility. An experiment with the unchanged dependency confirmed `data8` is empty until that nested decoder calls `end`. The test now checks that the outer continuation does not finalize the nested transfer. Both failures and the source observation are retained.

All five whole-member formatting checks pass. Exact aggregate formatting fails Windows OS206 and remains a failure, not a waiver. Browser execution and negative-control evidence are pending the resource allocation. The twelve new browser functions preserve all64 inherited assertions and compare constructor identity, graphics, protocol checks, exact errors, callbacks and real pixels. No browser pass is claimed here yet.

Run `node results/image-pixel-limit/run.mjs native`, `format`, `initial`, `final`, `iip`, `sixel` and `kitty` only in an owned O: worktree with current resource allocation and existing reference Node/Playwright/bindgen tools. `negative-control.mjs` swaps only this worktree's modified production files to exact0442 originals and restores every fixed byte in finally. New production modules stay unlinked under that baseline. Complete Windows/Linux/macOS quality and browser/demo matrices remain open. Overall equivalence is NOT PROVEN.
