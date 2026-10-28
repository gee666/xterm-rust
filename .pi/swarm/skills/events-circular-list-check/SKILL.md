---
name: events-circular-list-check
description: Use when checking synchronous Event listeners or CircularList storage against unmodified upstream assertions and seeded Rust/JavaScript operation traces.
---

Run `node scripts/check-events-and-circular-list.mjs` from the repository root with Node 24 and Cargo/Rust on PATH. No npm installation is needed. Node's native TypeScript transformation erases types; the original assertion bodies and implementation execute in a VM with an equivalent Chai assertion adapter. source_repo stays unchanged.

The script runs all 7 Event and 19 CircularList upstream tests, 95 original assertions. It builds the Rust event_trace and circular_list_trace examples, then compares 10,008 event operation traces and 20,001 circular storage snapshots. The seed is `0xc1ac01a7`. Event cases include listener removal/addition, nested dispatch and disposal during traversal. Circular storage comparisons include all wrapped slots, stale slots, length/capacity/fullness, return/error results and callback-time storage snapshots. Each subprocess has a 120-second timeout. Counterexamples are retained under results/events-circular-list/ on mismatch.

It also runs `cargo test -p xterm-core --test event --test circular_list`. Results, source hashes, versions, upstream outcomes and Rust logs go under results/events-circular-list/. This is scoped utility evidence, not complete terminal equivalence. The report lists Rust representation changes and rejected invalid CircularList size/range inputs.
