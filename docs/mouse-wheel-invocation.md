# Live wheel callback invocation

Overall mouse equivalence remains NOT PROVEN. Task334 fixes one concrete defect in real Terminal wheel dispatch. Task305 still owns the raw service facade, actual callback receiver and ordinary-object registry semantics.

## Change

`terminal/input_services.rs::custom_wheel` now invokes the stored callback with `Reflect.apply`, not `Function.call1`. The source calls its stored wheel handler directly, so looking up a callback's own `call` property was wrong. This change does not replace source-style event listener calls. Handler lookup releases its RefCell borrow before invocation, allowing replacement and recursive dispatch.

The callback still receives the original WheelEvent and vetoes only the literal boolean false. Exceptions propagate as the original JavaScript value. The existing undefined receiver is **not** source-equivalent and remains an explicit task305 gap. No fake service receiver or second protocol authority was introduced.

## Verification

On Windows with Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100:

```sh
WASM_TEST=wasm_terminal_wheel node scripts/browser-host-check.mjs
NODE_PATH=<existing source_repo/node_modules> node scripts/mouse-interop-oracle.mjs
cargo clippy --locked -j2 -p xterm-web --lib --test wasm_terminal_wheel --target wasm32-unknown-unknown -- -D warnings
rustfmt --edition 2021 --check crates/xterm-web/src/terminal/input_services.rs crates/xterm-web/tests/wasm_terminal_wheel.rs
```

The new tests construct the real Rust Terminal, open its DOM, select the shared native protocol, attach public wheel handlers and dispatch actual WheelEvents. They do not substitute an input-only host.

- Before the fix, the initial four-test suite passed3 and failed1 in each browser on the poisoned `call` case. Retained `results/mouse-wheel/baseline-results.json` records exact failure, browser versions and artifact hashes.
- After the fix, the final five-test suite passes5/5 in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. It covers active and passive routes, own `call` values/getters, original event identity, fifteen return classes, eight thrown-value classes and recursive hook replacement. A fifth guard verifies that ordinary onData listeners still read their own `call`, preserving that distinct source behavior and thrown identity.
- Existing unchanged-source oracle passes all9 groups. This is source-only evidence, not raw Rust registry parity. Its exact source hashes and receiver observations are retained in `source-results.json`.
- Focused WASM Clippy with warnings denied and two-file rustfmt checks pass. No original source assertion changed. An initial test compile failed because ErrorEvent was not an enabled web-sys feature. The test now reads its error property through Reflect, using existing Event support without a new dependency/feature.

`results/mouse-wheel/` retains command logs, source observations and full before/after reports. The baseline contains four tests because the onData guard was added after that run. No baseline assertion was weakened. Final full workspace, native three-OS, all-feature/release and original browser suites remain separate required gates.

## Continuing task305

The current native service is typed and infallible for restrictions/encoders. Raw browser callbacks can return arbitrary values or throw arbitrary values, while externally mutable registry objects can throw during getters. A sound facade needs a fallible boundary through actual DOM reporting and mode setters, not an empty object passed as `this` or a synchronized independent mouse service. Native DEC mode changes and public raw selections must use one authority. Preserve UTF16 names, inherited/prototype lookup, numeric Object.keys order, shared builtin protocol object identity, receiver shape and source method-call ordering. Reuse `xterm-wasm-support::mouse_fields` builtin callbacks and the existing source interop oracle.

The isolated temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/Fatima16`. No reference/dependency junction was created. Browser runners own and close their servers/browsers. Generated WASM/binding artifacts and the private target must be removed after review/integration, keeping these concise reports and regression source.
