# Public API coverage audit

Full equivalence is NOT PROVEN. Task567 inspects native and browser public contracts on integrated main `224d1b339a43eddf899c2944d9dad734d8c44234` with task561's accepted addon inventory changes. It does not accept parent328, run a terminal suite or certify current native/browser behavior. Producer branches after that main revision are excluded.

## What the index means

`tools/compatibility/public-mappings.json` links inspected public Terminal, options, events, markers, modes, addon ownership, buffer, parser and Unicode contracts to implementation and scoped tests. Each record remains partial. A source-qualified owner applies only to that declaration file; it is a guide to the named API family, not evidence that each member, raw argument or assertion has passed. Repeated per-record links keep generated rows independently readable. This cohesive machine-readable mapping data exceeds the 350-line guideline.

The public wrappers' runtime records now link real Rust implementations rather than hardcoding them missing. Both original wrappers gate only `unicode` and protect `cols` and `rows` from public option assignment. The index obtains those names from the unchanged TypeScript AST. Rust's `PublicOptions::check_proposed_api`, `verify_mutable_option` and `OptionsBinding` implement corresponding contracts. A partial runtime record does not establish every option side effect or JS coercion path.

This milestone adds 11 module, 32 exact-source owner and five suite guides. The complete inventory now has 106 partial and 103 unmapped modules, plus 531 partial and 239 unmapped API records. That is 363 additional partial API records, not 363 newly implemented or passing APIs. The reference still has 776 files, 209 production modules, 770 declaration/member records, 120 suite/support files, 2,426 static test definitions, 32 static benchmark definitions and 162 fixture rows. All Rust suite counters remain null and fixture-runner mappings remain unrun.

Unmapped members remain missing even when their owner has a partial mapping. In particular, `Terminal.strings` is declared for headless but absent from the supplied headless runtime. No native static getter or headless JS package replacement is invented. Browser localization is separate and is callable through `browser/xterm.mjs` and Rust `BrowserRuntime.strings`. The broad Terminal row explicitly retains raw callback, GC, event-loop and complete browser-family gaps.

## Callable representations and limits

| Contract | Native and browser representation | Remaining work |
| --- | --- | --- |
| Terminal construction and operations | `xterm_core::headless::Terminal`, `QueuedTerminal`, `NativeWritePump`; browser `Terminal` from `browser/xterm.mjs` over the Rust WASM class | Native `usize` dimensions and synchronous methods do not emulate JS Number coercion or timers. Native hosts drive pending parser work or use the queue. No compatible headless npm constructor is claimed. Full reentry, exceptional callbacks, event order and final platform/browser matrices remain open. |
| Options and proposed access | `PublicOptions` shares `OptionsService`; `OptionsBinding` creates live JS descriptors and sequential bulk assignment; `unicode()` checks allowProposedApi | Native unknown keys return errors instead of detached expando properties. Browser descriptor/prototype/accessor/error behavior and every renderer/input side effect require their own tests. Only Unicode access is runtime gated by these wrappers. |
| Events | Native `Event<T>` and disposable subscriptions; browser callable event functions forward Rust emissions and UTF-16 values | Rust `Result`/`EmitError` replaces native JS exception propagation. Historical numeric/mode helpers do not prove real queue ordering, listener mutation or callback identity. |
| Markers and modes | Shared native `Marker`, native marker-list snapshots, browser live marker wrappers; `ModesSnapshot` and JS modes object | Snapshot versus mutable Array is an explicit native difference. Complete raw marker resources, disposal/GC and protocol projection still need full-family evidence. |
| Addons | Native `TerminalAddon` and `AddonManager` handles; JS ownership facade passes the actual public Terminal to Rust `_loadAddon` | Managed handles replace JS instance-method rewriting. LIFO/error/reentry/GC and every cross-addon lifecycle are not inferred from component tests. |
| Buffers, lines and cells | `BufferNamespaceApi`, `BufferApiView`, `BufferLine`, `CellData`; JS `BufferNamespace`, `BufferView`, `BufferLine`, `BufferCell` | Native borrowed handles and signed/unsigned APIs differ from JS Number indexing. Browser getLine uses Number-aware ring lookup. Cell getters retain numeric masks and default color -1. All raw getters, targets, stale slots, translations, lifetime and safety-divergence cases still need assertion-level reconciliation. |
| Parser and identifiers | Native `with_parser`/`ParserApi` and owned `HandlerResult`; JS `ParserApi.register{Csi,Dcs,Esc,Osc,Apc}Handler`, deprecated add aliases where implemented | Native transient facade replaces cached JS object identity. Browser identifier reads and Promise handling use JS boundaries over Rust parsing, not the xterm.js engine. Full hostile identifiers, callback errors/reentry/disposal and stream-family closure remain open. |
| Unicode | Native `UnicodeService`/provider callbacks; browser `UnicodeApi` and live `_core.unicodeService` | Fresh JS public views share the service. UTF-16 version names use lossless boundary encoding. Native String/vector/enum representations are explicit. Full provider mutation/error/property and generated-table coverage remains open. |

The two declaration modules are partial indexes of these inspected families, not blanket mappings of every declared type. Types outside this milestone remain missing. Common internal services, browser rendering/input/selection/scrolling/CSS modules, remaining addon helpers, source fixtures and full assertion accounting still belong to parent328 and their implementation owners.

## Evidence is historical unless rerun

The mapping retains old failures rather than choosing only green receipts. `results/headless/latest.json` is failed after changed inputs; `results/public-api/headless-native-run.json` has exit101; `integration-summary.json` is failed; `numeric-summary.json` is changed-during-run. Earlier independent results described in `docs/public-api-acceptance.md` are scoped historical observations and do not override those latest reports. `headless-case-map.json` maps source case names, not every assertion or a current run.

`results/public-adapters/verification.json`, `results/options/summary.json`, Unicode browser evidence and marker results cover different revisions and scopes. The compatibility index hashes their current bytes for identity only. It never converts their counts into certified Rust suite totals. The final integrated native/component/original suite matrix remains separately required. No arithmetic adds counts across unrelated suites, test functions, assertions or revisions.

## Reproduce this inventory

With the supplied exact reference and Node22+:

```sh
npm ci --prefix tools/compatibility --ignore-scripts --no-audit --no-fund
node scripts/compatibility-inventory.mjs
node scripts/compatibility-inventory.mjs --check
node --test scripts/compatibility-inventory.test.mjs scripts/compatibility-baseline-links.test.mjs scripts/compatibility-addon-mappings.test.mjs scripts/compatibility-public-mappings.test.mjs
```

These commands inspect mappings, AST inventories, evidence/source hashes and fail-closed validation. They do not compile Rust or execute terminal behavior. Mapped cargo and WASM commands are instructions for a separate execution with its documented prerequisites, owned targets and bounded resources. Task567 claims no fmt, Clippy, workspace tests, browser matrix, Miri, native OS, demo or comparison execution. Parent328 must still reconcile those actual final receipts, licenses, unsafe review and every remaining source/API/assertion gap.
