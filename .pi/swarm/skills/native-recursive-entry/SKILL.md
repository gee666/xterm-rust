---
name: native-recursive-entry
description: Use when changing native queued parser ownership, Bell-triggered resize, recursive flush behavior or decoded-buffer identity.
---

# Native recursive entry

Read docs/native-recursive-entry.md. Reuse scripts/headless-async-reentry-oracle.mjs rather than building another source loader. The runner bundles the unchanged supplied headless Terminal into an owned tmp directory and removes it in finally. It invokes the actual source WriteBuffer timer body, without sleeps or a guessed event schedule.

Run `node scripts/headless-async-reentry-oracle.mjs` after installing the supplied reference's Node dependencies. Set XTERM_SOURCE_ROOT to a separate existing original reference with dependencies when needed. Normal mode rejects drift. Use --generate only for reviewed new source cases, never to replace failing Rust expectations.

Run `cargo test -p xterm-core -j2 --test headless_async_resize --test headless_recursive_fixture --test headless_reentry --test headless_queue --test headless_queue_edges --test headless_disposal_contract --test native_queue_differential`. Compare every fixture field, including negative queue accounting after recursive flush. Do not replace AABB by AB, suppress nested errors, clone the parser, or approximate recursion by replaying a saved prefix.

Core direct writes and browser parsing retain their separate existing entrypoints. This fixture proves native Bell reentry only. Source callbacks inside arbitrary parser handlers, other public event families and browser queue reentry require their own unchanged-source assertions. Use an approved heavy slot, owned O: target and explicit command timeout. Preserve the source hashes, failing attempt and focused raw receipts before cleanup.
