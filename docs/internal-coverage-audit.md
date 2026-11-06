# Common and browser coverage audit

Full equivalence is NOT PROVEN. Task573 continues parent328 on canonical main `14139449bae2792afb019f42e8438b074938cfe7`, with accepted task561 and567 mapping prerequisites. It changes no Rust implementation or original assertion. Later producer branches are excluded. Final328 remains open for the remaining contracts, assertion-level reconciliation and final integrated evidence.

## What changed

`tools/compatibility/internal-mappings.json` adds 86 source-module guides, 29 exact-source public type-owner guides and 15 explicit suite/support guides. The index now links all common, headless and browser modules, the canonical stylesheet and all120 suite/support paths. It has192 partial and17 missing modules,733 partial and37 missing declaration/member records. This adds202 partial API records, not202 newly implemented or passing APIs.

The guides were reconciled with Rust module declarations, public signatures, service ownership and retained subsystem reports. They identify owning implementations and commands, not each branch or assertion's implementation. Related source modules intentionally repeat their owning family's implementation/test/evidence links so each generated row is readable without chasing another row. Mixed `Types.ts` and `Services.ts` files do not have a single corresponding runtime module. Their guides identify owning structs, enums and traits, and explicitly preserve incomplete field/behavior reconciliation. The cohesive machine-readable mapping data exceeds350 lines for this reason.

| Source family | Rust contract and important limit |
| --- | --- |
| CoreTerminal, InputHandler, Async and WriteBuffer | Rust CoreTerminal owns parsing, decoders, buffers, handlers and state. Typed UTF-16/byte writes return Result/Pending; native hosts drive futures through queue adapters. Native polling is not JavaScript event-loop equivalence. |
| Buffer, BufferLine, BufferSet, reflow, range and markers | Shared Rust line/marker handles and numeric adapters replace JS arrays/Numbers. Cell helpers do not prove reentrant resize, cleanup or arbitrary sparse access. |
| Escape parser/constants/types | Rust tables, HandlerResult and explicit continuation are live implementations. The synchronous case map does not certify all async callback errors or full InputHandler integration. |
| Core, log, Unicode, OSC link, mouse, injection, task and platform services | Shared Rust service owners replace decorators/globals. Result/error channels, host scheduling, raw registry and callback identity retain explicit language and runtime gaps. |
| Browser root, DOM, input, clipboard and accessibility | Actual Rust WASM Terminal services create DOM and route data through the Rust core. Native helpers do not prove trusted keyboard/IME/clipboard/screen-reader behavior. |
| DOM rows, width/font cache, theme, rendering and dimensions | Rust renderer/service modules own real DOM/canvas operations. Public dimension snapshots and internal renderer views are distinct. Canonical CSS delivery is mapped, not a pixel/computed-style pass. |
| Selection, links, decorations and overview ruler | These use actual shared buffers, markers, input/render services and DOM ownership. Raw Number/getter/error/reentry and GC contracts remain separately required. |
| Viewport and all scroll helpers | Rust ScrollableElement, oriented bars, visibility, pointer/touch and host schedulers are callable components. Typed geometry does not close raw JS model arithmetic, trusted pointer capture or complete Viewport behavior. |
| Public record families | Source-qualified maps cover logger, theme, Windows PTY, VT/window options, decorations, browser localization/links and dimensions. These are type/owner guides, not member-by-member raw argument or callback certification. |

## Renderer guide correction after review

Independent573 review caught two wrong test/document ownership links. RendererUtils and renderer SelectionRenderModel had been grouped with RenderDebouncer callback tests and the unrelated selection service model tests. Those links were removed, not treated as indirect evidence. The original review found no source/hash/count errors and independently passed all15 controls, but correctly requested changes for these two guides.

RendererUtils now points to `crates/xterm-web/tests/webgl_cases/renderer_utils.rs`, included by the `webgl` target. Its five direct tests cover glyph classification boundaries, rescaling exclusions/thresholds, JavaScript rounding/remainder/signed-zero variant offsets, initial dimensions and FalsyValue text. Run `cargo test --locked -p xterm-web --test webgl`. That does not certify the WASM throwIfFalsy truthiness boundary or browser rendering.

The renderer SelectionRenderModel now points to `crates/xterm-web/tests/webgl_renderer.rs` and `docs/webgl-renderer-update.md`. The test calls `renderer::selection_model::SelectionRenderModel::update` and checks downstream source-derived renderer traces. Run `cargo test --locked -p xterm-web --test webgl_renderer`. This is indirect renderer selection coverage, not a dedicated full SelectionRenderModel assertion migration, and it is not `selection::model::SelectionModel` or DOM SelectionService evidence. Regression assertions in the inventory controls preserve the exact test/implementation distinction. No Rust test was rerun or changed during this guide correction.

## What remains missing

The exact outstanding rows are retained in `results/internal-coverage-audit/index.json` and generated inventories. They are not silently assigned a nearby implementation.

- Seventeen addon modules remain unmapped. Eight are ligatures type/processor/index layers, two are grapheme inflate/trie implementation layers, and seven are WebGL constants/types/full renderer/addon/declaration layers. Rust font/glyph/atlas components exist, but those alone do not establish each upstream layer or a callable WebglAddon. No WebglAddon constructor export was found in the inspected Rust library.
- Thirty-seven API records remain missing. These include the WebGL addon declaration/options, uninspected viewport-range and buffer-element-provider shapes, headless range/cell-position shapes, headless localization and the declared-but-runtime-absent headless Terminal.strings. Browser localization is separately callable. Similar names must not erase differing coordinate bases or runtime availability.
- Four modules,261 API records and690 static test definitions still lack focused acceptance mappings. This update does not substitute umbrella inventory criteria for missing behavior checklists.
- All162 fixture/support rows remain unrun by this index. A parser fixture or captured trace is not proof that every original escape input, expected output, disabled `.in_` case, Python helper or note has a mapped Rust runner.
- All120 Rust full-suite execution counters remain null. A guide to a suite is not an assertion migration ledger. The reference still has776 files,209 production modules,770 API records,2426 static test definitions and32 static benchmark definitions. Static definitions, expanded runtime cases, dynamic assertions, traces and Rust test functions are different units.

## Source-bound execution observations

This audit did not execute terminal behavior, Cargo, browser drivers, original suites, Miri, native OS CI, demo or comparison workloads. The only new executions are source/hash inspection, inventory generation/checking and15 lightweight Node controls. Their logs and hash pins are under `results/internal-coverage-audit/`.

The retained terminal-engine oracle has684 traces and9306 snapshots, seed1129599565. The escape parser source-case map has161 discovered original cases,147 synchronous mappings and14 async cases tracked separately, plus268 static assertion sites and60 helper expectations. These are scoped historical records, not full current-suite counters. No totals from them are summed into the inventory's Rust runtime fields.

Canonical141 includes integration554's source union. Its retained quality summary records2264 native test functions,2 doctests, native all-target/all-feature warning-denied Clippy and WASM library Clippy passing. Aggregate `cargo fmt --all -- --check` records Windows OS206 failure, while five member checks pass. Its browser summary records531 discovered component functions,530 passing and one Firefox exact TypeError-message failure. Original decoration records33 case-projects with27 passes and6 original WebKit skips. Source and Rust original serializer runs each record120 passes. These observations remain source-bound and do not prove the final all-target WASM, native OS, full original/demo or addon matrix. No failed assertion was normalized or weakened here. The later accepted554 review/cleanup receipts and current native CI checkpoint are separate artifacts, not inferred from these earlier summaries.

## Architecture, safety and attribution inspection

The portable xterm-core crate owns terminal state/parser/buffers and does not import a browser engine. Browser `xterm.mjs` imports only its generated Rust WASM module and manages ABI/ownership facades. The production Rust/JS source search found no `source_repo`, `@xterm/xterm` or `@xterm/headless` dependency except a charset attribution comment. This is a source inspection, not a dynamic dependency/security proof.

The reused quality snapshot scanner records lexical `unsafe` candidates with exact source lines. The inspected occurrences are comments about avoiding unsafe code/views and source decoder behavior, not unsafe operations. No unsafe operation was added by573. This is not Miri or sanitizer execution, nor a semantic review of every dependency/FFI boundary.

Workspace policy remains Rust2021/MSRV1.85, MIT and `unsafe_op_in_unsafe_fn = deny`. All five crate manifests inherit that policy. Existing source and license notices, including xterm, Unicode, Microsoft scroll helpers, font fixtures and third-party decoder dependencies, are retained and hash-pinned by this audit. This is an attribution-preservation check, not a new complete dependency/license compatibility audit. The actual distribution dependency closure and generated notices belong to accepted568 and final333; their missing/matched distribution obligations are not waived.

## Reproduce the index

From a checkout with the exact supplied reference, Node22+ and the locked TypeScript tooling installed:

```sh
npm ci --prefix tools/compatibility --ignore-scripts --no-audit --no-fund
node scripts/compatibility-inventory.mjs
node scripts/compatibility-inventory.mjs --check
node --test scripts/compatibility-inventory.test.mjs scripts/compatibility-baseline-links.test.mjs scripts/compatibility-addon-mappings.test.mjs scripts/compatibility-public-mappings.test.mjs scripts/compatibility-internal-mappings.test.mjs
```

Initial new control failure is retained separately. It incorrectly required a separate test path in an older Win32 mapping that deliberately keeps assertions inline. The corrected control still requires test paths for every new mapping and implementation/evidence/command/binding/gap for every common/browser guide. No terminal assertion was changed. The original addon/public negative controls remain intact, including unknown/duplicate mapping rejection, exact-source isolation and absent-member overrides.
