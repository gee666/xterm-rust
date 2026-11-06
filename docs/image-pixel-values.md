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

## Verification

The final source is9a694fe0, based directly on0442 without private prerequisites. There are thirteen new browser functions and all64 inherited functions remain intact.

- Initial and restored-final addon matrices pass77/77 in each Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. All have zero page errors, ignored or filtered functions.
- The exact two-file87ab821d context control passes76 functions and fails only the new nested-activation case with the intended current-storage mismatch. Both fixed files are restored by SHA in finally.
- The exact fourteen-file0442 production control passes all64 inherited functions and fails exactly the thirteen new functions. New modules remain unlinked under the baseline's module declarations. All fourteen fixed files are restored by SHA in finally.
- Unchanged typed producer suites pass IIP12, SIXEL15 and Kitty19 functions in each of the three engines, with zero page errors, ignored or filtered functions.
- Current native focused execution passes17 functions, including four new ownership, payload and layer-phase tests. Full native addon execution passes214 functions across51 result summaries, with no failures, ignored, measured or filtered tests.
- Native addon all-target/all-feature Clippy and actual WASM library plus all four image-test-target Clippy pass with warnings denied.
- All five whole-member formatting checks pass. Exact aggregate formatting fails Windows OS206 and remains a failure, not a waiver.

The first native compile failed for a missing import after splitting the cohesive end module. The next execution caught an incorrect new-test assumption about nested BASE64 visibility. An experiment with the unchanged dependency confirmed `data8` is empty until that nested decoder calls `end`. The test now checks that the outer continuation does not finalize the nested transfer. Both failures and the source observation are retained. There were no failed runtime launches or retries in the scoped final browser work.

The read-only audit verifies2298 source files,785 phase-specific Rust input pins,879 comparator input pins, both776-file reference manifests, seven actual fresh=false compiler records, seven compiler/WASM/binding artifact sets and three native executables. Initial and restored-final compiler hashes are identical. Exact comparator entries and source bundles are retained per phase. These bundles are generated upstream comparator fixtures, retained verbatim for identity checks rather than split into350-line modules or shipped as the Rust implementation. The context control is a real misplaced-image regression, not a missing-method failure.

The direct and shipped bindings both exercise retained constructor identities, completing public write paths, real IIP/Kitty pixels and graphics replies. Small BigInt geometry exceptions use the synchronous parser comparison because the original asynchronous write callback does not run after that throw. They are not converted into successful public-write cases.

Re-run on this producer revision in an owned O: worktree with a private target and current resource allocation:

```sh
node results/image-pixel-limit/run.mjs native
node results/image-pixel-limit/run.mjs initial
node results/image-pixel-limit/negative-control.mjs context
node results/image-pixel-limit/negative-control.mjs baseline
node results/image-pixel-limit/run.mjs final
node results/image-pixel-limit/run.mjs iip
node results/image-pixel-limit/run.mjs sixel
node results/image-pixel-limit/run.mjs kitty
node results/image-pixel-limit/run.mjs format
node results/image-pixel-limit/audit.mjs
```

Set existing `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE` and matching0.2.100 `WASM_BINDGEN`; keep TEMP/TMP/TMPDIR and Cargo target under owned O:. The audit's retained runtime paths identify historical artifacts and must not be mistaken for reusable builds after cleanup. Full original639/img01..48, complete private/default/prototype records, GC and native Windows/Linux/macOS final matrices remain311/324/304/327. Overall equivalence is NOT PROVEN.
