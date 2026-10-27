# Collection and workspace acceptance checks

This checklist responds to user post 32. It defines gates, not claims that they pass. Tests must compare the specified values or errors with unchanged upstream code, not merely execute a function. Full-port criteria remain active and are not replaced by these subsystem checks.

## COL-INSERT

Compare empty, one-element, ascending, descending, negative, fractional, zero, infinity and duplicate-key lists against `SortedList.ts`. For each insertion, verify sorted values after a read and after multiple pending inserts. Distinguish stable order within a batch from new-before-old order across equal-key batches with identity-bearing entries. Exercise inserts before, within and after an existing list and exhaustion of either merge input. Verify clear discards pending insertions and deletions, clears any failed-flush state, and permits reuse. Custom key objects and mutable key fields must use source ordering, not structural sorting. Key callbacks returning NaN or undefined need an explicit typed representation decision; NaN must not cause Rust's inconsistent-comparator sort to panic.

## COL-DELETE

Delete from empty storage, absent lower/middle/upper keys and already-flushed deletions. Delete first/middle/last entries, each equal-key object identity, and a distinct equal-valued object. Verify each boolean return before any read. Preserve duplicate pending deletion indices, their sorted cursor walk and retained later removals. Retain the concrete counterexample `[1,2,3]`, delete `1`, delete `1`, delete `2`, then `values` must yield `[2,3]`. Repeat deletes beyond array length and assert the JavaScript RangeError maps to typed Rust `InvalidArrayLength`, with no panic. After failure verify source's stuck flush guard, subsequent reads/inserts and clear recovery. Interleave inserts with pending removals, empty/full clears and multiple reads. Random traces must include repeated deletes without forced intervening reads and compare errors and post-error state.

## COL-ITERATE

For both key iterator and callback iteration, test empty storage, absent keys below/between/above stored keys, negative/fractional keys, duplicate groups and exact visit order. Verify `values` snapshots before subsequent insert/delete/clear operations. Rc-backed entries retain pointer identity through snapshots. Source callback/iterator reentrancy must be mapped explicitly and tested, including insertion, deletion, clear and nested key iteration. A Rust borrowing restriction that prevents the source behavior is a remaining compatibility gap, not a passing equivalence assertion.

## COL-MAP

Test missing/get/set/overwrite/clear/reuse for two and four keys, isolating every key position and checking no collision between unrelated tuples. Number/string coercion must match ECMAScript exactly for negative zero, NaN, both infinities, fractions, smallest/largest finite numbers, integers near 2^32 and 2^53, decimal/scientific thresholds 1e-6 and 1e21 and their neighbors. Compare seeded random f64 bit patterns, not just ordinary integers. Test owned and borrowed string forms, empty strings, non-BMP text and lone surrogate representation decisions. Test `__proto__`, `constructor`, `toString`, `hasOwnProperty` and `Object.prototype` changes. Upstream inherited lookup or mutation must have a typed representation or remain a named failing gap; plain HashMap behavior is not equivalent for these cases.

## COL-EVIDENCE

Run all 11 original SortedList/MultiKeyMap test bodies and all 49 assertions without changing their bodies. Preserve all assertions in Rust, plus the adversarial paths above. Record discovered/executed/passed/failed/skipped/unrun counts separately for original cases and Rust cases. Retain seed, iteration count, complete operation trace on mismatch, exact source hashes, commands, environment/toolchain versions and independent review findings in `results/collections/`. Cover numeric and identity-bearing list values and all four map key positions. A controlled idle queue proves only flush-on-read logic; actual idle scheduling needs browser scheduling evidence. A failed oracle exits nonzero and saves the counterexample before reporting failure. Native Windows evidence must not imply Linux/macOS passed.

## WS-CONTRACT

A clean checkout resolves the committed Cargo.lock under declared Rust 1.85/MSRV and stable Rust. Every exported module and generated table is tracked and compiles, with no nominal placeholder APIs. Core dependencies have no browser/OS requirement. Native and WASM adapters are separate workspace members; wasm-bindgen/js-sys/web-sys dependencies stay target-gated. Record feature defaults, edition, supported target combinations and dependency purposes/licenses/MSRV. Preserve upstream notices. Native workspace, no-default features, all-features, doctests, release builds, rustfmt and warnings-denied Clippy pass; WASM compiles/tests are recorded separately. Actual Windows/Linux/macOS execution and browser execution remain distinct gates, not inferred from successful cross-compilation. Root/member integration changes require independent review of code and actual commands. Temporary files stay under tmp/, private swarm runtime and credentials stay untracked, and user-owned files remain untouched.
