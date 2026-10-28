---
name: build-slots
description: Use when running scoped local native or WASM/browser checks while other swarm workers share the Cargo target cache.
---

Read `docs/build-slots.md` and `docs/target-relocation.md`. Use `node scripts/build-slot.mjs --owner NAME --lane native -- COMMAND ARGS`, or choose the wasm lane for browser/WASM checks. Defaults wait120s/execute180s; the outer bash timeout must exceed both plus cleanup, at least360s. Set explicit finite flags for longer commands. Do not wrap an acceptance parent that already owns an in-process lease in another same-lane CLI lease.

The reviewed FIFO queue uses host-monotonic ticks, atomic metadata publication, owned cleanup, abort/deadline propagation and1GiB repository+target filesystem guards. Never steal a stale active lease or bypass a resource failure. Failed commands retain immutable receipts under results/build-slots/.

Task218 relocated the sole persistent cache to C while keeping logical repo/target via a junction, allbytes/nsmtime/control hashes and Cargo profiles unchanged. Do not recreate a second target, set CARGO_TARGET_DIR, or manually clean the linked cache. Source/evidence/auth/private data never moved. Pin executable/WASM copies as regular files inside acceptance outputs before hashing or execution; do not weaken source symlink guards.

Local leases allow scoped debug checks with existing dependencies. No hidden workspace/all-target/release builds or dependency installation. Full quality matrices/release gates remain remote or separately coordinated. Tiny source-only Node checks under20MiB with no compiler/bundler/browser may run without a lease.

Self-tests: `node --test scripts/build-slot-core.test.mjs scripts/build-slot-review.test.mjs`. These prove automation behavior, not terminal parity.
