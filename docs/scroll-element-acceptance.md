# Scrollable element verification

Task167 continues Batu's concrete implementation, carried on the accepted Bharata integration stack. Branch `swarm/Dominic` starts at `71d73e4`. No old shared-tree files were copied back and no whole-stack main merge is claimed.

## Source and implementation

The read-only references are `source_repo/src/browser/scrollable/scrollableElement.ts`, `scrollableElementOptions.ts`, `mouseEvent.ts` and their imports. The supplied tree has no dedicated `scrollableElement.test.ts` or `mouseEvent.test.ts`. These checks add source-derived scenarios; they are not renamed original assertions.

Rust implementation:

- `crates/xterm-web/src/scroll/element.rs` owns composition, external/owned models, wheel listener and timer lifecycle, event forwarding, scrolling and disposal.
- `element_layout.rs` implements class changes, mutable options, eager/lazy render and shadows.
- `element_wheel.rs` implements predominant axes, conversion, sensitivity, rounding, future-position accumulation and consumption.
- `options.rs` preserves the source option getter/read order, defaults and unvalidated values.
- `mouse_event.rs`, `wheel_event.rs`, `iframe.rs` and `environment.rs` implement normalization, coordinate ownership and shared wheel classification.

Positions/dimensions use named Rust structs. The public methods and binding names remain listed in `docs/scroll-element-api.md`. Raw non-number model positions are task215, not silently claimed as covered by typed f64 tests here. Root/Viewport integration and main merge remain task71.

## Option accounting

| Source options | Evidence |
|---|---|
| lazyRender, useShadows | Source scenarios `lazy render`, `no shadows`; explicit render/error case and event-time shadow observations |
| className | Constructor default and source `class update`; Rust creation-only update test |
| handleMouseWheel | Source `toggle wheel`, disposed listener; getter exception leaves already-disabled listener detached |
| mouseWheelSmoothScroll | Shared-model smooth future accumulation; owned-model mutation occurs before any later getter; creation-only update test |
| flipAxes, scrollYToX | Source flipped axis, converted vertical and canceling diagonal scenarios |
| consumeMouseWheelIfScrollbarIsNeeded, alwaysConsumeMouseWheel | Source bottom boundary, needed and not-needed cases |
| mouseWheelScrollSensitivity | Source zero, inverted, fractional and updated values |
| fastScrollSensitivity | Source Alt default and explicit/updated fast values |
| scrollPredominantAxis | Source vertical/horizontal/tied diagonal, disabled and updated values |
| listenOnDomNode | Source external listener scenario |
| horizontal, vertical | Source hidden/visible and updated visibility scenarios |
| horizontalScrollbarSize, verticalScrollbarSize | Source updated geometry scenario; scrollbar helper suites own exact geometry |
| horizontalSliderSize, verticalSliderSize | Constructor size defaults exercised through every fixture; detailed slider geometry belongs to tasks165/166 |
| horizontalHasArrows, verticalHasArrows, scrollByPage | Source update geometry scenario; actual arrow/track interaction belongs to task166 |

Only the eleven change-interface fields are mutable through `update_options`, matching upstream. Creation-only fields and undefined change values are checked separately. Malformed raw model arithmetic stays explicit in task215.

## Executed tests

`scripts/scroll-element-source.mjs` bundles the unchanged source with esbuild and runs 38 scenarios in each engine. Its report retains input options/actions, source hash and complete expected positions, consumption and DOM class observations. `scripts/scroll-mouse-source.mjs` retains 13 direct wheel and eight mouse-constructor cases per engine. Each report is overwritten with `running` before execution and `failed` on errors. Servers and browsers close in finally paths.

`crates/xterm-web/tests/wasm_scroll_element.rs` and three support modules execute 11 Rust/WASM test functions per engine:

- The original three port fixtures cover DOM-before-event ordering, listener toggling, external-model survival and lazy rendering. The wheel fixture now explicitly supplies wheelDeltaY=-120 and wheelDeltaX=0. The former unrun fixture incorrectly assumed native synthetic WheelEvent legacy fields were identical across engines. Its 50px assertion is unchanged. The additional native-event differential preserves and verifies the actual engine differences.
- One function replays all 38 retained Element source scenarios, comparing every step's top/left position, default prevention, container class and all shadow classes.
- One function compares all 13 wheel and eight mouse source-constructor traces, including legacy fields, modern line/pixel/page normalization, target fallbacks, strict button comparison, fractional/negative coordinates and client-coordinate fallback.
- Six functions cover getter exception identity and partial listener mutation, owned-model getter ordering/integer positions, frozen-option failure, nested event consumption, smooth future accumulation/disposal and creation-only update rejection.

All 11 functions passed in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 on native Windows. No ignored or filtered cases. The served Rust suite does not import the upstream engine. Reference bundles run only in the separate source-generation processes.

Native checks passed 43 library tests, five scrollable tests, the 1,024-snapshot scrollbar differential test and seven visibility tests. Native scoped Clippy passed. Full `xterm-web --all-targets --all-features` tests fail in unrelated `tests/support/cells.rs:18`, passing u32 to the now-u64 URL field. Full native Clippy also reports unrelated `renderer/width_cache_tests.rs:131` excessive precision. Strict WASM Clippy fails on six pre-existing stack issues: unused renderer cursor_width, two unused fast_dom_node helpers, unused terminal lifecycle fields, and two callback type-complexity warnings. These were not suppressed or edited under task167.

## Reproduction and artifacts

See the `scroll-element-check` skill for commands and environment overrides. Raw source inputs/results are committed in `results/scroll-element/source.json` and `mouse-source.json`. Browser runner reports and logs remain under `results/browser-host/wasm_scroll_element/`, with the latest summary at `results/browser-host/wasm_scroll_element-results.json`. Native command logs are retained under `results/scroll-element/`.

Environment: cargo/rustc1.85.1, wasm-bindgen CLI/runtime0.2.100, Node24.15.0. A fresh Cargo1.85 target created its cache tag only under `target/wasm32-unknown-unknown`; the runner requires the parent tag too. This run copied that actual Cargo-generated tag into the private target root. No assertion or source was changed to bypass the artifact check.

The initial task167 evidence above was collected on the private integration stack. It did not establish main integration or whole-port equivalence.

## Main integration, task283

Only task167 checkpoint `b29f518` was cherry-picked onto a fresh main branch, `swarm/DominicIntegrate`. Main already contained identical concrete Element producers. No private Root history was replayed. Canonical `node scripts/worktree-merge.mjs` first refused to merge because main advanced during testing. A second bounded invocation rebased, retested and fast-forwarded main to `25058162d4792128fc2cc619a12eb5d250822988`.

The accepted merge ran default-feature native build and all-target tests for changed crate xterm-web, affected-dependent all-target checking, workspace checking and applicable WASM library checking. Native tests passed298 cases across90 targets, including targets with zero tests. This default-feature gate does not imply all-feature Clippy or release/platform gates. Full output is `results/scroll-element/canonical-merge-2.log`; the refused race attempt is `canonical-merge.log`.

After that exact merge, a clean private target rebuilt the suite and all11 Rust functions again passed Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. See the updated `results/browser-host/wasm_scroll_element-results.json` and `results/scroll-element/integration.json`. The old cells.rs failure above belongs to the initial Bharata stack, not this integrated checkpoint. Other global gates were not silently inferred from this component pass.

Full browser API equivalence, all-feature workspace health, Linux/macOS runtime success, malformed raw model arithmetic, cross-origin iframe completeness and every dynamic getter path remain separate obligations. Overall equivalence remains NOT PROVEN.
