# Service identity type boundary

Task366 repairs the native typed identity used by `ServiceRegistry`, `ServiceCollection` and `InstantiationService`. `ServiceId<T>` retained a registry key containing `TypeId::of::<T>()`, but its `PhantomData<fn() -> T>` permitted covariance in `T`. Safe Rust could convert `ServiceId<fn(&())>` to `ServiceId<fn(&'static ())>` without changing that retained key. Registering through the converted ID could then bypass the type check enforced by `set_value`, leaving the original typed ID unable to retrieve the value.

The marker is now `PhantomData<fn(T) -> T>`, making `T` invariant. This has no runtime storage, dependency or unsafe-code cost. It deliberately rejects the previously accepted identity conversion. Callers can still clone an ID, share its erased key, and explicitly register separate function types. The fix preserves the existing Rust type-validation contract rather than inventing JavaScript runtime type checks.

## Regression evidence

`results/service-identity/` retains commands' raw outputs and `verification.json` with source and implementation hashes. Checks ran on native Windows x64 using Rust/Cargo1.85.1 and Node24.15.0, based on maina2a1252.

- Negative control: the new `ServiceId` compile-fail doctest was added before changing the marker. Cargo exited101 because the conversion compiled successfully. The fixed version rejects that same conversion. The original `DispatchKey` compile-fail check remains intact.
- Normal Cargo runs passed14 service tests, including two new function-pointer identity/injection tests and12 existing tests. Both core doctests passed. No assertion was removed or weakened.
- The unchanged service-injection oracle passed2,601 source/Rust comparisons, including2,500 seeded operations, seed`0x5e7a1ce`. The source modules and oracle script were not changed. These modules have no original upstream test files, so this count is not2,601 migrated upstream tests.
- Full-workspace formatting, warning-denied native core/web all-target/all-feature Clippy, and warning-denied xterm-web WASM library Clippy passed.

The oracle's first attempt used the integration reference, which has no installed TypeScript dependency, and failed before comparisons. `oracle-missing-typescript.log` retains that failure. The successful run used `XT_SOURCE_ROOT` pointing to the preserved original P: reference and its existing TypeScript installation. No installation, reference mutation or dependency link was needed. The generated oracle report/build log are copied under this task's `differential/`; the old tracked `results/service-injection/` snapshot is preserved.

```sh
cargo test --locked -j2 -p xterm-core --test service_identity --test service_injection --test service_dependency_views
cargo test --locked -j2 -p xterm-core --doc
CARGO_BUILD_JOBS=2 XT_SOURCE_ROOT=/path/to/unchanged/source_repo node scripts/service-injection-oracle.mjs
cargo fmt --all -- --check
cargo clippy --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
```

WASM compilation is not a browser runtime test. Native Linux/macOS, complete common-family closure312, the full workspace quality/runtime matrix and full xterm.js equivalence are not established by this repair. Overall equivalence remains NOT PROVEN.

## Temporary ownership

Only `O:/xterm-rust-swarm-16/tmp/A16` belongs to this task, including its regular private `target/` and `tmp/`. No browser or server was started. Preserve tracked source and evidence, then remove those disposable files after review/integration. No other worktree, cache or original reference is a cleanup target.
