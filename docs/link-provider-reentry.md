# Provider callback transaction ordering

Task 464 repairs one source callback-ordering defect. Full selection/link parity and overall equivalence remain NOT PROVEN.

## Contract and implementation

In `source_repo/src/browser/Linkifier.ts`, `_askForLink` captures a shared `linkProvided` local. Each callback assigns the return from `_checkLinkProviderResult` only after that method finishes. `_checkLinkProviderResult` invokes `_handleNewLink` before returning. A synchronous hover callback can therefore resolve another provider while the outer assignment is still pending. A thrown hover does not assign the outer result.

The Rust controller previously returned a selected handle from `ProviderReplies`, which had already set its shared request flag, then invoked hover. That suppressed the source's nested selection. With a higher-priority link and a pending lower provider, resolving the lower provider inside the first hover produces two higher-link hovers in the original, but only one in the old Rust controller.

`try_receive_selecting` and `try_cached_selecting` keep the selection callback inside the existing priority algorithm. They assign the shared result only after successful selection. No table or reply-group borrow spans selection. Existing nonselecting typed APIs delegate with an empty selection callback, preserving their behavior and signatures. The browser controller uses the selecting variants. There is no new provider registry, terminal engine delegation, dependency or unsafe code.

## Verification commands

Run from the task branch root, with the original reference and installed tools available:

```sh
SOURCE_ROOT=/absolute/reference/source_repo node scripts/link-provider-transactions-source.mjs
SOURCE_ROOT=/absolute/reference/source_repo PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs node scripts/public-link-provider-source.mjs --provider-reentry --generate
cargo test --locked -j2 -p xterm-web --test link_provider_transactions --test link_replies_live --test link_replies_differential
WASM_TEST=wasm_link_provider_reentry CARGO_BUILD_JOBS=2 PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs WASM_BINDGEN=/absolute/wasm-bindgen node scripts/browser-host-check.mjs
```

The source method runner transpiles unchanged TypeScript in memory. Its four cases cover nested selection, thrown outer selection, caught nested failure and a synthetic cached-method transaction. The cache case isolates the method ordering, not a separate live provider request.

The same opened-Terminal program runs with the original and Rust. Its six cases resolve nested undefined, empty and nonempty lower replies, three providers, an outer hover failure and a caught nested hover failure. Each case asserts two hovers, exact thrown marker identity where applicable, later activation of the current link, pointer state and absence of unexpected DOM errors. It uses real public provider registration and real screen mouse events, not a fake terminal service. The Rust test compares the complete trace against the original fixture.

## Evidence and limits

Results and their content pins are retained under `results/link-provider-reentry/index.json`.

- Original opened Terminal passed all six cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. The final Rust test passed one function replaying those same six cases per engine.
- The exact two production files from base `32bf954467fe3389e9cfe42fdd8c34d425975b62` compiled against the new browser assertion, then failed in Chromium with `nested undefined: expected 2 hovers, got 1`. The driver restored both final files in `finally` and verified exact hashes before the final three-engine pass.
- A separate native early-commit mutation failed the intended one-versus-two selection assertion. This is a targeted ordering mutation, not a claim that the new selecting APIs existed on the old base. Exact restoration was verified before the final native tests.
- Four existing browser targets passed per engine after the fix. Public provider has one function, linkifier seven, disposal three and terminal links four. This is 15 existing functions per engine, separate from the new six-case function.
- Focused native tests passed nine functions, including the retained 12,219-operation seeded reply differential. The subsequent `cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 372 tests across 152 summaries, zero failed or ignored.
- `cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings` and the same command with `--target wasm32-unknown-unknown` passed. The WASM library-only lint also passed before the full WASM-target lint. Changed-file rustfmt passed. Exact workspace `cargo fmt --all -- --check` failed with Windows OS error 206. No workspace format pass is claimed.

The first source browser trace wrongly assumed the high provider could select before all lower replies completed. The built-in OSC provider's truthy empty reply requires the all-replies fallback. A second harness revision incorrectly reused the same-row provider cache. Both source failures are retained. The final trace completes lower replies first and alternates rows, with all six assertions intact. No Rust production changes were made to accommodate either harness correction.

The preflight original reference check passed all 776 manifest hashes. No original source, assertion, AGENTS.md or credential file was changed. The browser programs exercise exported Terminal providers, not demo distribution loading or the full unchanged upstream matrix. Native Linux/macOS and full workspace completion are not claimed. Temporary browser/compiler binaries are pinned for review and removed during owned-worktree cleanup; the receipts record their original locations and hashes.

This directed regression does not establish arbitrary mapped-container, sparse/prototype reply, malformed payload, out-of-domain drawing coordinate, lifecycle-GC or complete selection/link family parity. Task 321 still owns that closure. It does not validate utility task 450, which remains an independently preserved producer.
