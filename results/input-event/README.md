# InputEvent verification

Observed locally on Windows with Node v24.15.0, rustc 1.85.1 and Cargo 1.85.1. The producer started from integrated main3ee576a. Source changes are in commits9e71763 and16ad77b, followed by fixture/lint corrections in this branch. No Linux/macOS runtime or full equivalence claim is made.

## Fresh scoped outcomes

- Native `cargo test -p xterm-web --test input_event_getters --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters -j2`:20 tests passed. See native.log.
- `cargo clippy -p xterm-web --test input_event_getters --test keyboard_dispatch_callbacks -j2 -- -D warnings`: passed. See clippy-native.log.
- `cargo clippy -p xterm-web --target wasm32-unknown-unknown --test wasm_input_event --test wasm_input --test wasm_input_callbacks --test wasm_keyboard_live -j2 -- -D warnings`: first failed on an unused test-helper import, then passed after removing it. Both clippy-wasm.log and clippy-wasm-fixed.log are retained.
- Focused `rustfmt --edition 2021 --check` for all changed Rust files: passed. See fmt-focused.log. This is not a full workspace formatting receipt.
- `node --test scripts/input-event-fixture.test.mjs`:2 checks passed. See source-fixture.log for upstream and fixture hashes. The copied method bodies match upstream with only line-ending/surrounding-whitespace normalization.

Browser command was `WASM_TEST=NAME node scripts/browser-host-check.mjs`, one suite at a time with CARGO_BUILD_JOBS=2. WASM_BINDGEN selected the installed0.2.100 CLI; PLAYWRIGHT_MODULE selected the existing source_repo/node_modules/playwright/index.mjs. TEMP/TMP and private target were in the owned O: worktree.

Final observed component counts per Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4:

| Suite | Test functions passed per engine |
| --- | ---: |
| wasm_input_event |4 |
| wasm_input |7 |
| wasm_input_callbacks |3 |
| wasm_keyboard_live |12 |

Total26 test functions per engine,78 expanded browser executions, no ignored/filtered cases. The InputEvent guard test additionally compares288 input combinations per engine against the unchanged source method. Do not count those as288 upstream test definitions.

## Failing-first and fixture findings

The08:46:54 InputEvent run temporarily restored only the old DOM binder from9e71763, leaving the new dispatcher/test suite in place. It failed3/4 test functions in each engine. Restoring the fixed binder produced4/4 passes at08:47:24. This establishes a real behavioral regression, not a compile-only demonstration.

The08:47:38 input run failed only Firefox clipboard setup. A browser-only reproduction showed a fresh DataTransfer containing `one\ntwo`, but Firefox's ClipboardEvent constructor returned empty clipboardData. The test helper now supplies the same clipboardData property explicitly; no copy/paste assertions changed. firefox-clipboard-lifetime.log retains the independent observation and an initial corrected Node ESM-path invocation error.

The08:50:19 keyboard run failed one Rust expectation in all engines. Changing keyCode230 to16 takes three source reads, not the old literal two. The corrected test executes the unchanged CompositionHelper.keydown body, asserts its three reads and compares Rust. It passed12/12 at08:52:22. This is a corrected port-test expectation, not a modified upstream assertion or a production-policy change.

`matrix.json` indexes every fresh browser run. `browser/` retains full reports and per-engine output, including failures, compiler/binding/CSS/test source hashes and commands. Artifact paths in those reports identify temporary outputs. Disposable WASM/bindings/build artifacts are not retained after cleanup; hashes and logs are retained. Existing older tracked reports were not used as fresh evidence.

## Independent review

Task292 was accepted at002669c after independent inspection of the complete diff from main4b02ba1. The reviewer reran both source-fixture integrity checks, all20 focused native regressions, `git diff --check`, and the real DOM InputEvent suite in all three engines. Each engine passed4 functions, including288 source-body guard combinations. The09:02:24 report and full outputs are retained here and indexed in matrix.json with `independentReview: true`. Existing input/callback/keyboard reports were inspected. The reviewer found no concrete defect and confirmed the remaining raw payload gap is explicit. Reviewer artifacts were removed after those reports were saved.

## Canonical integration

The reviewed producer and owner-confirmed accepted search290, wheel334 and CI-recovery336 source/receipts were integrated together by the canonical helper at mainc0dd42be7420b828e9c2b0572c19374f6dcf7c5a. It passed1821 native tests across327 summaries with zero failures/ignored cases. Changed core/web builds and all-target tests, affected all-target checks, the default workspace check and applicable WASM library checks passed. The regular private target was removed automatically. See canonical-merge.json and canonical-merge.log. The active CI Node verifier was also rerun independently,45/45 passing, in batched-quality-node.log.

These are the canonical helper's specific gates, not the full workspace all-feature/Clippy/fmt/final-platform matrix. No full equivalence conclusion follows.

## Remaining scope

See docs/input-event-dispatch.md. Admitted second-read non-string payloads still need raw browser CoreService event plumbing. Complete keyboard/IME/focus/accessibility, full original suites and final integrated platform/quality gates remain separate work. This scoped pass is not full port equivalence.
