# Lifecycle port

`crates/xterm-core/src/lifecycle.rs` ports the supplied `source_repo/src/common/Lifecycle.ts`. It has no platform or browser dependencies and adds no package dependency.

| Upstream | Rust |
| --- | --- |
| `IDisposable` | `Disposable` trait with `dispose(&self)` |
| `toDisposable` | `to_disposable`, repeatable `Fn()` callback |
| `dispose` | `dispose`, `dispose_optional`, `dispose_all` for Rust's separate argument types |
| `combinedDisposable` | `combined_disposable` |
| `DisposableStore` | `DisposableStore` |
| abstract `Disposable` | embedded `DisposableOwner` with `register` |
| `Disposable.None` | `NONE` and `NoopDisposable` |
| `MutableDisposable` | `MutableDisposable`, `value`, `set`, `clear` |

A `DisposableHandle` is `Rc<dyn Disposable>`. Pointer identity preserves JavaScript's Set deduplication and mutable replacement checks. These utilities are single-threaded, as the source is. A callback may register another resource because no `RefCell` borrow is held while callbacks run. `clear` visits newly registered resources in the same traversal, like a JavaScript Set iterator. Disposal marks the store closed before callbacks, so registrations during disposal are immediate.

Rust stores, mutable disposables and embedded owners dispose on drop. This is a deliberate Rust ownership addition, not a JavaScript garbage-collection guarantee. Dropping an arbitrary callback handle does not invoke it. Applications should avoid reference cycles, use `Weak` for callbacks referring to their owners, and explicitly dispose where source callback timing is required. As upstream, a mutable owner ignores assignments after disposal without disposing the assigned object.

Callbacks have no returned error, matching `IDisposable.dispose(): void`. A user callback panic propagates; callbacks should not panic during drop. No unsafe code is used. There is no upstream `Lifecycle.test.ts` in the supplied tree, so the tests are direct contract assertions, not a claimed migration of a nonexistent suite.

## Checks

With Rust 1.85 or later and Node 24 or later:

```text
cargo test -p xterm-core --test lifecycle
node scripts/check-lifecycle.mjs
```

The Rust integration suite currently has nine tests. It checks callback repetition, helper return identity, ordered combined disposals, store deduplication, reuse after clear, permanent disposal, callbacks registering resources during traversal, mutable replacement visibility, disposal visibility and Rust drop cleanup.

The TypeScript checker imports the unchanged reference directly with Node's built-in type stripping. It and the Rust test `same_trace_as_typescript_oracle` assert the same 16-callback fixture, `crates/xterm-core/tests/fixtures/lifecycle-trace.txt`. This is one deterministic trace, not randomized terminal-state equivalence. It retains a nested-clear counterexample found during independent review. The first Vec-index implementation skipped a resource added after a nested clear; tombstones now preserve live Set iterator positions. Node emits a module-type warning for the upstream package; the checker does not edit its package metadata to suppress that warning.

Native Windows execution is recorded in `results/lifecycle/`. Linux and macOS runtime checks remain unrun. The lifecycle checks do not establish complete terminal or browser equivalence.
