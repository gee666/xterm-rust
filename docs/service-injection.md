# Service registry and constructor injection

`crates/xterm-core/src/service_registry.rs` ports `ServiceRegistry.ts`. `instantiation.rs` ports `InstantiationService.ts`. Both retain source attribution. The core uses standard-library `Rc`, `Weak`, `Any`, `TypeId` and `RefCell`, with no unsafe code or new dependency.

## Rust representations

A caller owns `ServiceRegistry` and shares its typed IDs with the constructors it wires. `create_decorator::<T>` interns a name within that registry. Different registries intentionally have distinct identities. Unlike a JavaScript generic cast, a repeated name with a different Rust type is rejected. A `ServiceKey` erases only the generic parameter, not identity. Dynamic values are checked against the key's type before mutation.

`ConstructorMetadata` represents compiler-emitted parameter metadata and explicit parent inheritance. `dependencies_view()` is the typed equivalent of `getServiceDependencies`. Its `DependencyList` is an `Rc<RefCell<Vec<Dependency>>>`. Annotated constructors and descendants share the same mutable list until a child's first successful decorator creates its own list. Without inherited or own annotations, each call returns a fresh, unattached empty list. Clearing an annotated list does not detach it. Views retained before a child gains its own decorators still refer to the parent's list.

Live edits to service IDs, indices, ordering and membership affect subsequent construction. Stable sorting mutates that same list, even if injection later fails. Release list borrow guards before calling metadata methods or constructing instances. No list or metadata borrow is held while the constructor callback runs. `dependencies()` remains an owned snapshot for existing callers. Dependency records use Rust values rather than JavaScript object references; arbitrary reflection, nonnumeric/negative indices and JavaScript constructor invocation are separate binding concerns. Rust indices use `usize`.

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

There are no original `ServiceRegistry.test.ts` or `InstantiationService.test.ts` files. Nine new Rust tests are not nine migrated upstream tests. The source oracle makes 2,601 comparisons, including 38 live-view operations and 2,500 randomized operations with seed `99066318`, or `0x5e7a1ce`. It compiles unchanged source in memory with TypeScript and stubs only the imported `IInstantiationService` constant using the original registry factory and exact source name. It does not substitute the original registry or instantiation algorithms.

## Commands and current evidence

With Cargo and Node on PATH and upstream TypeScript dependencies installed:

```text
cargo test -p xterm-core --test service_injection --test service_dependency_views
node scripts/service-injection-oracle.mjs
cargo clippy -p xterm-core --test service_injection --test service_dependency_views --example service_injection_trace -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/service_registry.rs crates/xterm-core/src/instantiation.rs crates/xterm-core/tests/service_injection.rs crates/xterm-core/tests/service_dependency_views.rs crates/xterm-core/examples/service_injection_trace.rs
```

Local environment is native Windows x64, Node 24.15.0, TypeScript 5.9.3, rustc 1.85.1 and cargo 1.85.1. Fresh 12-test and 2,601-comparison runs pass after the shared terminal/parser migrations. Three live-view tests cover detached empties, multi-level inheritance and replacement, clearing/redecorating, error-time sorting and constructor callback mutation. Worktrees may set `XT_SOURCE_ROOT` to the preserved reference directory with TypeScript installed; source modules still execute unchanged. Scoped warning-denied Clippy and formatting pass. `results/service-injection/` retains replay input, expected/actual output, source and implementation hashes, oracle summary and the earlier failed compilation attempt. A retained `failed-build.json` records a historical integration failure, not the current passing build. Check `build.log`, `tests.log`, `clippy.log` and the implementation hashes against current files before reusing this evidence.

Task 101 separately verifies the terminal's direct typed construction against the upstream dependency graph and browser APIs. Coleman chose direct typed service construction for the terminal engine, not a runtime service locator. Native Linux/macOS, browser/WASM integration and complete xterm.js equivalence are NOT PROVEN by this utility evidence.
