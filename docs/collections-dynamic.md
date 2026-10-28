# Dynamic sorted-list contracts

`SortedList` retains scalar convenience APIs and adds the explicit mutation-capable contracts needed to represent JavaScript iteration in safe Rust.

- `KeyCursor::new(key)` creates a lazy generator state. `list.next_by_key(&mut cursor)` flushes pending changes on first resume, returns one cloned entry per resume, and then observes later list mutations. It does not precompute a values snapshot. Use `Rc<T>` entries to preserve reference identity.
- `for_each_by_key_reentrant` passes `&mut list` and the current entry to the callback. The callback may insert, delete, clear or recursively iterate. `try_for_each_by_key_reentrant` propagates typed callback errors without discarding prior mutations or advancing the work index after failure.
- `IterationContext` explicitly represents the source module's shared work index. Lists that may interact through nested callbacks or suspended cursors must share a context with `with_iteration_context`. A terminal/application owner supplies that shared context rather than relying on unsafe or process-global mutable state. Integration with host scheduling and context ownership remains task 83.
- `SortedListError` distinguishes source array-length errors from key getter failures. `delete` and `delete_by` now return `Result<bool, SortedListError>` because their pending insertion flush can fail too.
- Native scalar getters accept an exhausted pending batch as JavaScript undefined, which compares unordered. For a throwing source getter such as `entry => entry.key`, configure `with_undefined_key_error` with the original callback error description. This preserves both the source merge-exhaustion error and the source's above-maximum deletion error. Optional source getters such as `entry => entry?.marker.line` must keep the default behavior.
- Option-based keys map `None` to unordered numeric comparison and supply `with_undefined_key` to distinguish undefined from NaN during deletion. An undefined deletion key returns before changing the shared work index. Query keys remain numbers, as declared by the source API.

Source callback mutations can expose bugs. Repeated pending deletions retain the original cursor walk; a failed negative array allocation leaves the original deletion guard stuck until clear. A throwing insertion getter leaves both the old array and the sorted pending batch intact. A generator that throws during its first flush closes permanently, even if the list later recovers. Binary search uses the original left midpoint, including when external key changes make the stored order inconsistent.

Rust's standard sort expects a consistent comparator. NaN breaks that assumption. The unordered-key path uses V8's observed initial-run and stable binary-insertion rules for small batches and never passes an inconsistent comparator into Rust's total-order sorter. ECMAScript explicitly leaves inconsistent-comparator order implementation-defined; identical large-batch ordering across all JS engines is not claimed. This is recorded in the full equivalence gaps rather than counted as browser evidence.

## Reproduction and retained cases

```sh
cargo test -p xterm-core --test collections_iteration --test collections_dynamic_keys
node scripts/collections-dynamic-oracle.mjs
```

The dynamic runner executes the untouched `SortedList.ts` body after in-memory TypeScript erasure with a controlled queue. It retains 55 directed operations and 4,000 seeded operations, seed 356830, in `tests/fixtures/collections-dynamic.tsv`. Rust consumes every one of the 4,055 rows and compares outputs and typed error classes. Cases include identity-bearing values, undefined/NaN/infinite and mutated keys, two lists sharing the work index, lazy generator close, callback deletion/clear/nesting/failure and retained post-error state. Source hashes, fixture hashes, commands and tool versions are in `results/collections/dynamic-oracle.json`; Rust logs are in `dynamic-rust.log`. Mismatches retain the failing prefix and output in `dynamic-counterexample.json`.

Malformed mutable order combined with the source's shared work index can make nested callbacks revisit entries indefinitely. The generated workload callback throws after 128 accumulated visits in both implementations. This is a bounded throwing input callback, not a modification of the source algorithm or a suppressed test failure. The compared trace includes the resulting error and all visits committed before it. Real browser idle callbacks are not exercised by this controlled queue.

`collections_iteration.rs` and `collections_dynamic_keys.rs` retain eight directed Rust regressions. The unchanged scalar/original assertion suite and repeated-delete tests remain required. Multi-key prototype behavior is separate task 105. Passing these component tests does not establish terminal or browser equivalence.
