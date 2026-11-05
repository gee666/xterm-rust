# Shared raw mouse builtin policies

Task 256 extracts the existing native builtin restrictions and encoders into `xterm-core/src/mouse_fields.rs`. Native `MouseEvent` implements its field/operator contract in `mouse_fields_typed.rs`. `xterm-wasm-support/src/mouse_fields.rs` implements the same contract with live `JsValue` properties. There is one Rust policy implementation, not a second browser encoder. No dependency was added.

The host adapter catches only property operations and JavaScript arithmetic/string operators. Strict comparisons do not coerce. Bit operations coerce at each original read. DEFAULT retains addition before its greater-than checks and later character conversion. SGR computes its final character before eventCode and coordinates. Each getter/setter can throw its original value. No event is serialized or copied before a builtin call.

`MouseFields` is required because typed snapshots cannot retain an accessor's repeated reads, object coercion, Symbol/BigInt failures or partial writes. Native operations are infallible by type, so their wrappers eliminate `Infallible` rather than unwrap host errors. The raw functions return `Result` with the original `JsValue` failure.

## Reproduce

Install the source reference's locked Node dependencies, wasm32 target, wasm-bindgen 0.2.100 and Playwright browsers. Keep Git and Cargo on PATH. In an isolated worktree:

```sh
node scripts/mouse-fields-oracle.mjs
cargo test -p xterm-core --test mouse_differential --test mouse_state --test mouse_custom --test mouse_callbacks --test mouse_button_numeric
cargo test -p xterm-web --test mouse_policy --test mouse_routing
WASM_PACKAGE=xterm-wasm-support WASM_TEST=wasm_mouse_fields node scripts/browser-host-check.mjs
cargo clippy -p xterm-core -p xterm-wasm-support --lib --target wasm32-unknown-unknown -- -D warnings
```

Set WASM_BINDGEN and PLAYWRIGHT_MODULE if their installations are outside this worktree. An older worktree can use the primary reference's Node dependencies via NODE_PATH. The shared source loader handles its existing source junction without removing or traversing it for cleanup. Do not create a new junction.

The source oracle bundles the unchanged MouseStateService and dependencies. `tests/mouse_fields_cases.mjs` supplies identical host values/accessors to source and Rust. The 1,125 cases cover five protocols, three encodings, raw button/action/coordinate values, repeated changing coercions and thrown getters. Full read/write/coercion logs, output UTF-16 and sentinel identity are compared. TypeError messages are not compared because browsers differ. The existing numeric fixtures remain unchanged. Regeneration is explicit with `--generate`; do not regenerate a failing expectation to accept a Rust mismatch.

Observed Windows evidence: 27 core tests passed; the raw WASM test passed all 1,125 cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Exact browser commands/logs are under `results/browser-host/wasm_mouse_fields/`. Warning-free native and WASM library Clippy and focused rustfmt passed. Native browser-policy tests passed 13/13 after the refactor. `cargo fmt --all -- --check` could not launch rustfmt on Windows because its argument list exceeded the OS limit, error 206. That workspace gate is not passed. Native browser-policy results and outstanding Root compilation failures are recorded separately.

## Current-main recovery, 2026-11-05

Recovered the policy code from `swarm/Tuah` c307ed3 and its module-format correction into `swarm/Wyatt16`, based on main 3ee576a. The current browser runner already supports `WASM_PACKAGE`, so its newer artifact identity and cleanup handling remain unchanged.

Fresh Windows runs verified all 1,125 unchanged-source cases, passed 27 core tests and 13 native web policy/routing tests, and passed the raw WASM suite in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Native test logs and WASM library warning-denied Clippy are under `results/mouse-fields/*-recovery.log`. Browser commands, source/build hashes and per-engine logs are under `results/browser-host/xterm-wasm-support__wasm_mouse_fields/2026-11-05T08-27-23-449Z-9ffc7e6d-086e-40f2-9873-0eff7e9f9d97/`. Generated binding/WASM artifacts were removed after this run; hashes remain in the report. Focused rustfmt passed after sorting current support module declarations. The canonical merge helper then merged the component into main at `137dbf5`, passing changed-crate all-target tests, affected workspace checks and applicable WASM library checks. Exact log: `results/mouse-fields/gated-recovery-merge.log`. This does not establish the full all-feature workspace or raw-service facade gates. Current continuation is task305; `docs/handoffs/wyatt16.md` records the remaining live service boundaries.

## Scope still open

These are callable builtin policies, not a completed raw MouseStateService facade. Task 171 retains ordinary-object registries, prototype/numeric names, actual stable service receiver, custom callback returns, shared live Core mode authority, event ordering, lifecycle and real DOM routing. Task 85 still needs its actual web-crate WASM run after the committed Root graph builds. Full equivalence remains NOT PROVEN.
