# Mouse callback failures

Task345 makes native mouse restrictions and encoders fallible and propagates their errors through real Terminal DOM reporting. This is a prerequisite for task305, not completion of its raw registry or callback receiver.

## Contract

`MouseRestriction` returns `Result<bool, EmitError>`. `MouseEncoding` returns `Result<Vec<u16>, EmitError>`. Existing native callers wrap successful results in `Ok` and handle the service methods' `Result`. The default protocols still use the same shared Rust field policies. Their impossible `Infallible` branch is eliminated explicitly, not converted into a default.

Rejection, empty encoding and failure are different results. A failed restriction skips encoding and output. A failed encoder skips output. Neither failure commits the outer report's last-event cache. Mutations performed before failure remain visible. A completed recursive report retains its own cache update even if the outer callback later fails. Registry and callback borrows never cross invocation or capture destruction.

`MousePolicy::trigger` now returns `MouseRouteError<E>`, preserving callback errors separately from output errors. `route` uses the same error representation without nested wrapping. The actual DOM adapter unwraps a callback error through the existing `to_js_error` chain, so arbitrary JavaScript thrown values, including undefined, false, symbols and object identities, survive unchanged. No new dependency or unsafe code is needed.

## Verification scope

The new native `mouse_failures` suite covers restriction and encoder errors, pre-failure event mutation, original native error identity/source chain, successful retry after failure and recursive replacement followed by failure. The real Terminal suite `wasm_terminal_mouse_errors` covers eight JS thrown-value classes for each callback kind and recursive DOM dispatch/replacement. It opens the Rust Terminal and uses its actual native mouse service and DOM listeners, not a substitute MouseHost.

The source interop probe keeps all existing nine groups and adds two groups for restriction/encoder exceptions and recursive replacement. It bundles unchanged `MouseStateService.ts`, retains input/module hashes and disposes esbuild. Its source-only result must not be counted as Rust parity or an original upstream suite.

On Windows with Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, final scoped checks passed:

- 43 native tests in eight targets, including the two new failure tests and all16,426 retained differential rows. No ignored tests.
- Real Terminal error suite2/2, existing wheel suite5/5 and mouse DOM suite24/24 in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. These are31 tests per engine,93 expanded cases, not93 original suites.
- Source-only interop11/11 groups. The two new groups cover16 exception cases and2 recursive replacement cases. Original upstream suites executed by this probe remain0.
- Native core/web all-target/all-feature warning-denied Clippy and three scoped WASM suites plus library warning-denied Clippy. Exact `cargo fmt --all -- --check` passed against c0dd42b plus the owned changes. After rebasing onto a656b8f, the unchanged inherited `terminal/mod.rs` marker declaration order fails full formatting. This later failure is retained rather than relabeled as a pass.

`results/mouse-callback-errors/` retains raw reports, browser artifact hashes and full command logs. The browser runner's generic `componentOnly` field is unchanged; these two new tests actually construct/open Terminal. Reproduce from a worktree with the documented existing Playwright/esbuild dependencies, wasm-bindgen0.2.100, and Cargo-j2:

```sh
cargo test --locked -j2 -p xterm-core --test mouse_callbacks --test mouse_custom --test mouse_state --test mouse_differential --test public_api_differential -p xterm-web --test mouse_routing --test mouse_policy --test mouse_failures
WASM_TEST=wasm_terminal_mouse_errors node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_wheel node scripts/browser-host-check.mjs
WASM_TEST=wasm_mouse node scripts/browser-host-check.mjs
node scripts/mouse-interop-oracle.mjs
cargo clippy --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --test wasm_terminal_mouse_errors --test wasm_terminal_wheel --test wasm_mouse --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
```

The existing mouse DOM matrix initially passed23/24 in each engine. Its iframe fixture tried casting a child-realm HTMLDocument using the parent realm's `instanceof Document`, failing before any mouse assertion. Using the existing typed `HtmlIFrameElement::content_document` getter fixes that setup without changing any assertion or production routing. The retained before-fixture-fix report contains all three failures; the final matrix passes24/24. Initial WASM checks on old32fe51f found the already-repaired stale input test host. Rebasing onto c0dd42b reused the accepted input repair. It then exposed two unused imports and two missing Default implementations in mouse test helpers; those causes are fixed without suppressions.

## Independent review

Task345 accepted HEAD2449524 against a656b8f. The isolated reviewer reran all43 focused native tests and warning-denied native core/web all-target/all-feature Clippy successfully, inspected the error/borrow/cache paths and verified every retained browser report and generated WASM/binding hash. No browser rerun was authorized in that compiler-only review slot; none is claimed.

The reviewer reproduced an inherited full-format failure at `terminal/mod.rs`: `mod marker_api;` must follow `pub mod link_services;`. Task337 introduced that module ordering in the integrated base. This is recorded in `post-rebase-fmt.log` and routed to the integration/quality owner. It does not change task345's reviewed callback behavior. The concise review receipt distinguishes this from the earlier c0dd42b full-format pass.

## Canonical integration

The canonical helper integrated the reviewed mouse changes and accepted font metadata339 into main3baa698790c47fd21f9b5e89c9cfd55d2ca5d036. It also retained the previous recursive-parser gate receipt and independently inspected Milica's formatting-only559daa9 fix. Full formatting now passes. The unchanged failed post-rebase check above remains historical evidence, not the current result.

The standard helper passed builds/all-target tests for changed xterm-core, xterm-addons, xterm-web and xterm-font-addons, affected dependent all-target checks, native workspace check and applicable WASM library checks. The raw log contains1,987 passed tests across369 test summaries, zero failures and zero ignored tests. These are default-feature canonical gates, not the final all-feature/release/platform matrix. `canonical-merge.json` and `.log` retain the command and exact scope; `integrated-fmt.log` retains the restored full-format result. The helper removed this worktree's private target after the successful fast-forward.

## Remaining work

The browser registration in these tests uses the public native Rust callback API. This does not expose JavaScript addProtocol/addEncoding, preserve raw callback event-object identity or allow non-string encoding results. Task305 still needs ordinary-object registries, UTF16/inherited/prototype names and ordering, shared builtin object identities, the stable actual service receiver, fallible raw getter handling, and one authoritative native/browser mode state. No fake receiver or second protocol service was added.

Required complete original suites, full workspace/release/feature gates, real three-OS runtime evidence and final application proof remain separate work. Overall equivalence is NOT PROVEN.
