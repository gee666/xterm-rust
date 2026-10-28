# Native public adapter review

Reviewer Antigone, task 163, implementation task 66 owned by Charulata. This is a native implementation review, not acceptance of pending browser/WASM behavior.

## Source and code inspection

Inspected the unchanged source AddonManager, BufferApiView and BufferNamespaceApi contracts and the Rust adapters under `crates/xterm-core/src/public_api/`. The native headless wrapper uses these adapters against the real Rust engine, not a mock terminal.

- Addon registration precedes activation, so activation failure retains cleanup ownership. The managed handle marks disposal before invoking user code, preserves the captured removal index across callback mutation and leaves a failed record marked disposed. Reverse manager traversal matches the source's fixed initial index walk. Invalidated traversal is a named native error rather than an unchecked index or swallowed callback failure.
- Rust does not replace a raw object's dispose method. Managed handles are an explicit native representation change. Duplicate object loading and cross-manager aliasing remain JavaScript binding obligations; these native tests must not be used to close that gap.
- Buffer getters read the actual shared buffer and line identities. Namespace views retain their public identity and refresh backing buffers after reset. Signed and unsigned line access deliberately preserve CircularList's wrapped and stale-slot behavior rather than imposing the typings' advertised bounds. Native borrow conflicts are recoverable errors. Negative cell indexing uses a production signed adapter rather than a test-only conversion that cannot reach implementation code.
- Parser registrations forward owned Params snapshots and UTF-16 payloads for CSI, DCS, ESC, OSC and APC. A transient borrowed ParserApi replaces the JavaScript cached wrapper without duplicating parser state. The terminal adapter guards bare CSI t before custom callbacks using current window options; prefixed or intermediate-bearing commands do not gain that guard. Actual parser registrations own their lifecycle handles and pending futures.
- No application unsafe block or browser dependency was added to these native adapters. Weak namespace subscriptions and WeakAddonManager avoid required library ownership cycles. User callbacks must not capture a strong manager back into an addon unless explicit disposal breaks that ownership cycle.

Inspected the two original AddonManager assertions and the native activation, disposal, error/reentry, buffer and parser suites. Original terminal-consumer assertions are also exercised through `public_adapter_terminal` and the headless original-case tests.

## Findings

### Callback error causes

`AddonError<E>` originally implemented Error without returning its Callback(E) cause. HeadlessError could expose the AddonError but a generic WASM source-chain walker could not recover the original callback object from it. Charulata added `source()` for Callback(E) and a directed downcast test. This fix has been inspected and its native downcast test passed in this reviewer's 23-case rerun.

### Verification receipts

The inspected `scripts/check-public-adapters.mjs` overwrote `command-N.log` and `verification.json` on each invocation and recorded input hashes only after execution. A later successful run could replace a failed command log, and concurrent source edits could be assigned to an earlier run. Requested versioned receipts, before/after identity checks and reuse of `quality-process.mjs` for bounded process-tree handling. Dynamic discovered/executed/failed/skipped/unrun accounting is already present. The revised runner now retains unique run directories, compares before/after source hashes and uses the bounded process-tree helper. The later bootstrap correction persists an incomplete attempt before dynamically importing helpers or reading inputs, so setup failure cannot leave an older pass looking current. The status guard now also rejects skipped/unrun/count-mismatch cases. These revisions have been inspected. A controlled run of the versioned runner passed with stable input hashes; the newest counter/registration additions still need a fresh reviewer rerun.

## Observed execution and pending gates

Before the disk pause, this reviewer ran the actual `public_adapter_terminal` consumer tests as part of `scripts/check-headless.mjs`. The commands passed in `results/headless/runs/run-1793147195775/native-tests.log`, alongside the headless and queue cases. Warning-denied scoped Clippy passed in that run. Concurrent unrelated source changes invalidated the combined snapshot receipt, which remains marked failed. These are observations, not a clean final snapshot claim.

Under Kassa's controlled Wave 2 allocation, this reviewer ran both source oracles and all 23 native adapter cases, including the new error-source test. All passed with `snapshotStable: true`. The immutable adapter receipt is `results/public-adapters/runs/2026-10-28T00-57-56-911Z-072b66b9/report.json`; the independent outer receipt is `results/reviews/antigone-wave2-1793149070527/report.json`. The source addon report contains two unchanged original cases and a checks count of 28; the buffer report contains 12 directed checks.

The combined Clippy attempt failed on an unrelated redundant closure in `terminal/commands.rs`, retained in the outer receipt. A later 190-leased headless run passed all 89 native cases but failed Clippy on an unused buffer helper; that failure remains in `results/headless/runs/run-1793149761128-d748f05b/clippy.log`. These are not native-adapter lint passes. Three new registration-during-callback cases subsequently increased the focused adapter inventory to 26. The owner reports a passing stable 26-case run, but this reviewer has not yet independently rerun those additions or obtained a current warning-free scoped Clippy receipt. Additional commands use the accepted build-slot queue and its 1 GiB resource guard. This is coordination waiting, not an external functionality blocker. Task 66's browser addon method mutation, JavaScript Number indexing and original browser Parser/Terminal suites remain unaccepted regardless of native results.

## Inspected file hashes

```text
e04f6d278f1596d1853f31741740f5217ed9034613d4128d17a65cb45c05477c  crates/xterm-core/src/public_api/addon_manager.rs
345cac244a50cce0d451025d105c87237283c36447a04a8969af25e3f04d63fd  crates/xterm-core/src/public_api/buffer.rs
d3ccbe1ea19810356bd0db76a6723aeb098e69577e83a95d52d9046c783976c6  crates/xterm-core/src/public_api/parser.rs
6e8ca0069b7acd0e6acba5f644f7278cbeaeca35ae865ec4ca4fd3274d261d3e  scripts/check-public-adapters.mjs
```

These are the initial inspected file hashes, before the receipt-runner and cached-registrar revisions. Newer execution hashes are recorded in the immutable run receipts above. Neither set certifies execution of a newer working tree. Overall equivalence remains NOT PROVEN.
