# Browser core disposal inspection

Full equivalence remains NOT PROVEN. Task413 repairs the missing internal lifecycle inspection used by six unchanged original browser assertions. It does not implement the entire JavaScript DisposableStore API or its writable private fields.

## Contract and implementation

`Terminal._core._store` now returns one stable Rust-backed inspection object. Its read-only `_isDisposed` and `isDisposed` getters read the same `Rc<Cell<bool>>` used by the existing core disposal guard. No second flag, simulated service or JavaScript terminal logic was added. `operations.rs` disposal ordering is unchanged. Public disposal, internal `_disposeCore`, retained views and addon-disposal callbacks observe the same state, including after an addon throws.

The source is `src/common/Lifecycle.ts`: DisposableStore marks `_isDisposed` before disposing children; `Disposable` owns `_store`. Original `test/playwright/Terminal.test.ts` lines747 and760 read the flag after unopened and opened disposal. Rust keeps its existing native/browser cleanup implementations; this view does not pretend to offer source `add`, `clear`, direct private-field mutation or the complete internal service ABI. Those broader internal contracts remain under315/317/328.

## Evidence

Before this fix, the complete original core attempt on main51112351 discovered1434 cases, attempted1172, passed1133, failed39, originally skipped240 and left22 unrun after two beforeAll timeouts. Six failures were the missing `_core._store._isDisposed`;19 were missing registerDecoration and12 missing registerLinkProvider. Raw original assertions are preserved under `results/e2e/upstream/2026-11-05T16-52-06-511Z-aabed0ab-d451-49a9-86ca-84a54bca4106/`. No source assertions changed. The remaining bindings and full rerun stay task327/331/321 work.

The patched source is2aea06d7 atop current maind29e4bad, including accepted image options. `results/disposal-store/verification.json` pins exact source/build/report identities and raw logs. Commands from the owned O: worktree:

```sh
WASM_TEST=wasm_terminal_disposal node scripts/browser-host-check.mjs
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite core --grep 'API Integration Tests dispose' --no-build --timeout-ms 120000
cargo clippy --locked -p xterm-web --all-targets --all-features -j2 -- -D warnings
cargo clippy --locked -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_terminal_disposal -j2 -- -D warnings
cargo fmt --all -- --check
rustfmt --edition 2021 --check crates/xterm-web/src/terminal/internal_api.rs crates/xterm-web/src/terminal/mod.rs crates/xterm-web/tests/wasm_terminal_disposal.rs
```

Rust tests passed3 functions in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. They exercise stable store/core identity, false-before/true-after opened and unopened disposal, repeated disposal, no remaining parent DOM, direct core disposal, and state visible inside addon callbacks with exact thrown-object identity and no repeated callback. All six unchanged original disposal cases then passed, with zero failures/skips/unrun. Native and focused WASM Clippy passed with warnings denied. Exact full-workspace fmt still failed Windows OS206; only the listed changed-file rustfmt check passed. Do not upgrade it to a full formatting pass.

Fresh pre-fix checkpoint evidence also retains the unchanged JS unit baseline2389pass/71original pending/2460discovered and Rust all-feature/all-target workspace2067pass/417summaries. They are separate counts and source51112351, not the full patched matrix or one-to-one assertion migration. Task327 remains open.

Raw logs and original Playwright error contexts are retained byte-for-byte, including trailing blank records and whitespace in source excerpts. As a result, an unrestricted Git whitespace check flags those evidence files. The handwritten source and documentation check is separate and passes; raw evidence was not rewritten to make that check green.

Generated WASM/binding/compiler artifacts are disposable owned outputs, not committed distributions. Reports retain their identities; rebuild with locked dependencies and wasm-bindgen0.2.100 to reproduce. New temporary work stayed under `O:/xterm-rust-swarm-16/tmp/worktrees/MalcolmRun2`, using the regular private target/tmp and one owned source_repo/node_modules junction to existing P: dependencies. Unlink that junction before recursive worktree cleanup. No original dependency/reference files are owned or deletable by this task.
