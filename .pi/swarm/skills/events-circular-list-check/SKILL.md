---
name: events-circular-list-check
description: Use when checking synchronous Event listeners or CircularList storage against unmodified upstream assertions and seeded Rust/JavaScript operation traces.
---

Run `node scripts/check-events-and-circular-list.mjs` from the repository root with Node 24 and Cargo/Rust on PATH. No npm installation is needed. Node's native TypeScript transformation erases types; the original assertion bodies and implementation execute in a VM with an equivalent Chai assertion adapter. source_repo stays unchanged.

The script runs all 7 Event and 19 CircularList upstream tests, containing 95 original assertions, plus 12 directed source checks. It builds Rust event_trace and circular_list_trace examples, then compares 10,008 event operation traces and 20,047 circular storage snapshots. Seed is `0xc1ac01a7`. Event cases include addition/removal, nested dispatch, exact callback-error identity and disposal during traversal. Circular comparisons include all wrapped/stale slots, length/capacity/fullness, return/error results, callback-time storage, signed indexing and f64 rounding/subnormal/nonfinite cases. Each subprocess has a 120-second timeout. On mismatch the replayable prefix since the latest reset is retained under results/events-circular-list/.

The script runs 74 Rust tests across event, event_edges, event_errors, circular_list, circular_list_edges and lifecycle_explicit. Reports include original and Rust source hashes, harness hashes, versions, upstream outcomes and raw logs. Input files are checked again after execution to reject source changes during a run.

Use `--isolated` only when unrelated in-flight workspace modules prevent a scoped utility check. It compiles these exact source modules with rustc into small nonincremental files under repository-root tmp/, with no new Cargo target directory. Results go to verification-isolated.json and rust-tests-isolated.log, separate from full Cargo evidence. Isolated success is not a workspace-quality pass.

Also run scoped Clippy and rustfmt, then workspace quality gates after integration. This is utility evidence, not complete terminal equivalence. docs/acceptance/events-circular-list.md maps named acceptance cases to tests and lists actual Buffer/browser obligations. The report records Rust representation changes and rejected invalid size/range inputs instead of hiding them.
