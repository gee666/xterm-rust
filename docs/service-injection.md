# Service registry and constructor injection

`crates/xterm-core/src/service_registry.rs` ports `ServiceRegistry.ts`. `instantiation.rs` ports `InstantiationService.ts`. Both retain source attribution. The core uses standard-library `Rc`, `Weak`, `Any`, `TypeId` and `RefCell`, with no unsafe code or new dependency.

## Rust representations

A caller owns `ServiceRegistry` and shares its typed IDs with the constructors it wires. `create_decorator::<T>` interns a name within that registry. Different registries intentionally have distinct identities. Unlike a JavaScript generic cast, a repeated name with a different Rust type is rejected. A `ServiceKey` erases only the generic parameter, not identity. Dynamic values are checked against the key's type before mutation.

`ConstructorMetadata` represents compiler-emitted parameter metadata and explicit parent inheritance. A child initially resolves parent dependencies. Its first successful decorator creates an independent list. Sorting is stable and mutates the owning metadata, including the parent when the child has no own decorators. `dependencies()` returns a Rust snapshot. Arbitrary JavaScript reflection, mutable returned-array aliasing, nonnumeric/negative decorator indices and JavaScript constructor invocation are not advertised as Rust APIs. Rust indices use `usize`; browser binding equivalence is a separate integration obligation, not established by this utility.

`ServiceCollection::set` returns `Option<Rc<T>>` instead of JavaScript `undefined` for an absent previous entry. Replacement preserves insertion order. `for_each` sees entries appended during traversal and replacements made before visiting an entry. It releases the collection borrow before callbacks; nested traversal is supported. `try_for_each` propagates a callback error immediately and retains mutations made before that error.

`InstantiationService::new` registers the same instance under `InstantiationService`. Its own entry is weak, including explicit re-registration of itself, so ownership does not introduce a self-cycle. Shared objects supplied by callers must use weak references for cyclic graphs; this API is not a tracing garbage collector. Typed construction callbacks receive static arguments followed by dependency values. Validation occurs first, including source-exact UNKNOWN and static argument mismatch messages. Callbacks may return their own `Result` without losing their error. `ServiceValue` preserves null/undefined as `EmptyValue`; bool, numeric and string values obey JavaScript truthiness. Other typed values represent objects and are truthy.

## Source branches and checks

| Acceptance cases | Rust evidence | Original comparison |
| --- | --- | --- |
| R1 name identity/type boundary | registry_identity_names_and_type_boundaries | R commands including empty/non-BMP names |
| R2 arity/inheritance/replacement/sort | inherited_metadata_replacement_arity_and_stable_sort | N/D/M/I commands including invalid arities and inherited in-place sort |
| C1 absence/replacement/order/identity | collection_replacement_identity_and_initial_entries | R/S/G/H/E commands |
| C1 callback insertion/replacement/reentry/errors | collection_iteration_is_live_reentrant_and_insertion_ordered; collection_callback_failure_keeps_prior_mutations_and_stops | L/E commands compare live insertion/replacement and nested iteration |
| I1 self-registration and release | self_registration_replacement_and_cycle_free_drop | Original self-identity assertion in oracle bootstrap |
| I1 falsey classes and object values | primitive_truthiness_and_objects | V commands for undefined/null/bools/zero/-zero/NaN/infinity/strings/object |
| I2 order/error precedence/callbacks | missing_service_precedes_static_count_and_never_calls_constructor; argument_errors_order_gaps_no_dependencies_and_callback_failures | I commands compare error text, unknown-before-count, index gaps, duplicate services and arbitrary static counts with no dependencies |

There are no original `ServiceRegistry.test.ts` or `InstantiationService.test.ts` files. Nine new Rust tests are not nine migrated upstream tests. The source oracle makes 2,563 comparisons, including 2,500 randomized operations with seed `99066318`, or `0x5e7a1ce`. It compiles unchanged source in memory with TypeScript and stubs only the imported `IInstantiationService` constant using the original registry factory and exact source name. It does not substitute the original registry or instantiation algorithms.

## Commands and current evidence

With Cargo and Node on PATH and upstream TypeScript dependencies installed:

```text
cargo test -p xterm-core --test service_injection
node scripts/service-injection-oracle.mjs
cargo clippy -p xterm-core --test service_injection --example service_injection_trace -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/service_registry.rs crates/xterm-core/src/instantiation.rs crates/xterm-core/tests/service_injection.rs crates/xterm-core/examples/service_injection_trace.rs
```

Local environment is native Windows x64, Node 24.15.0, TypeScript 5.9.3, rustc 1.85.1 and cargo 1.85.1. Initial 9-test and 2,563-comparison runs pass. Later shared-worktree checks encountered ongoing terminal/parser migrations. `results/service-injection/` retains replay input, expected/actual output, source hashes, oracle summary and build failures. Check current logs, not an earlier passing snapshot. Scoped formatting passes. Warning-denied Clippy must be rerun after integration before this task is accepted.

Task 101 separately verifies the terminal's direct typed construction against the upstream dependency graph and browser APIs. Coleman chose direct typed service construction for the terminal engine, not a runtime service locator. Native Linux/macOS, browser/WASM integration and complete xterm.js equivalence are NOT PROVEN by this utility evidence.
