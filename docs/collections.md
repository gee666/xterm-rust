# Common collection ports

These modules implement the common sorted-list and multi-key-map utilities, not a terminal. Full xterm.js equivalence remains NOT PROVEN. Browser idle scheduling and JavaScript prototype behavior are outside the verified behavior described below.

## API mapping

| Upstream | Rust | Contract |
| --- | --- | --- |
| `SortedList<T>` | `sorted_list::SortedList<T, F>` | `F: Fn(&T) -> f64`, key callbacks return a JavaScript-sized number. |
| `clear`, `insert`, `delete` | `clear`, `insert`, `delete` | Scalar deletion uses `PartialEq`; insertion/deletion return `Result` for deferred failures. |
| strict reference equality during `delete` | `delete_by` | Pass `Rc::ptr_eq` for object identity; structural equality is not a substitute. |
| `getKeyIterator`, `forEachByKey` | `KeyCursor`/`next_by_key`, `for_each_by_key_reentrant` | Explicit mutation-capable iteration; share `IterationContext` across interacting lists. See `collections-dynamic.md`. |
| Read-only traversal conveniences | `get_key_iterator`, `for_each_by_key` | Borrowed entries; these convenience methods cannot mutate the same list during traversal. |
| `values` | `values` | `Result` of an owned snapshot requires `T: Clone`. Use reference-counted entries to preserve object identity. |
| internal idle tasks | `flush`, reads | Explicit/read-driven flushing, no native event-loop scheduler. |
| `TwoKeyMap`, `FourKeyMap` | `multi_key_map::{TwoKeyMap, FourKeyMap}` | Key arguments accept `PropertyKey` conversions. |
| map `set`, `get`, `clear` | `set`, `get`, `clear` | Missing entries return `None`, values are borrowed. |
| JavaScript property coercion | `PropertyKey` | `f64`, `i32`, `u32`, `String` and `&str`; other Rust integer widths require an explicit JavaScript-number conversion. |

New sorted insertion batches precede existing equal-key entries. Equal-key insertion order within each batch stays stable. A batch flush merges rather than repeatedly inserting into the stored array. Rust's immutable iterator borrow prevents the source's callback-driven list mutation during key iteration; `values` instead makes the same mutation-safe snapshot as upstream. Key callbacks must remain stable during an operation. NaN keys never match numeric equality; their inconsistent ordering cannot define a portable sort order. A separate stable binary-insertion path avoids passing that inconsistent comparator to Rust's total-order sort implementation and follows the observed V8 small-batch ordering.

Numeric map keys use `ryu-js` 1.0.3 for ECMAScript number formatting, including negative zero, NaN, infinity, large decimal exponents and small decimal fractions. Rust's standard float formatter differs on these boundaries. This is the reason for the dependency. Its license is `Apache-2.0 OR BSL-1.0`, not MIT; this repository retains the BSL notice in `licenses/ryu-js-BSL-1.0.txt`. Its declared MSRV is Rust 1.71, below the workspace's 1.85. No application unsafe code is needed for these utilities. Source attribution is retained in each port and `licenses/xterm-js-MIT.txt`.

## Evidence and reproduction

Run with Node 24 and Cargo in PATH:

```sh
cargo test -p xterm-core --test collections_upstream --test collections_regression
node scripts/collections-oracle.mjs
cargo clippy -p xterm-core --lib --test collections_upstream --example collections_trace -- -D warnings
```

The original test bodies run after TypeScript erasure in memory, with controlled deferred queue callbacks. No source test or assertion is rewritten. Cross-realm arrays are cloned into the same realm before strict comparison. The original suites contain 11 tests and 49 assertions. Rust retains all of these in 11 tests, with three additional identity, coercion and pending-change tests and three retained deletion/error regressions. `results/collections/oracle.json` records source hashes, commands, Node/Cargo versions, counts and seed `5285869`. The seeded run performs 5,000 collection iterations plus 1,010 numeric map coercion cases, totaling 7,038 trace operations and 3,131 compared results. Unlike the initial oracle run, these traces do not force a read after deletion. They retain repeated-index and negative-array-length counterexamples with post-error state checks. Rust output comes from the built `collections_trace` example, not from JavaScript. That example is a verification protocol driver, not the requested interactive terminal demo.

The observed platform is native Windows. Linux/macOS execution is not established by these results. Other terminal subsystems and browser integration need their own evidence.

## Remaining differences and checks

- `IdleTaskQueue` timing is not executed. The oracle controls deferred callbacks and flushes on reads.
- Repeated pending deletion indices are deliberately not deduplicated. The original cursor can leave later requested removals intact. Negative-array-length failure maps to `Err(InvalidArrayLength)` from insertion, reads, key callbacks or explicit flush. Like the source, the failed deletion-flush guard stays set until `clear`, so subsequent reads retain old entries. Three regressions and differential traces verify this behavior.
- JavaScript inherited object keys and `__proto__` can mutate or retrieve prototypes. Rust stores these as ordinary string keys and does not expose prototype mutation.
- Borrowed traversal conveniences cannot mutate the same list. The explicit `KeyCursor` and reentrant callback APIs preserve source mutations, lazy generator close, shared work-index effects and callback failures. `results/collections/dynamic-oracle.json` records 4,055 additional compared source operations.
- Non-total NaN sort order is implementation-defined in JavaScript and not asserted equivalent.

Task 76 owns dynamic key/reentrant list behavior. Task 105 owns prototype-sensitive maps, raw UTF-16 keys, random f64 bit patterns and all tuple positions. Task 83 owns real idle-scheduled batch behavior and host-level shared iteration context. The detailed criteria are 104 through 109, defined in `docs/collections-acceptance.md`. These differences must remain visible in the full compatibility inventory. Passing these scoped checks does not discharge the full-port criterion.
