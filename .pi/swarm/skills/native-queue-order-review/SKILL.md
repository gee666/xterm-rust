---
name: native-queue-order-review
description: Use when checking shared-host WriteBuffer timer versus Promise microtask ordering, native pump regressions or PA03 pending lifecycle integration.
---

# Native queue ordering review

Read docs/native-queue-order-review.md first. The probe is scoped host scheduling evidence, not a full terminal/browser comparison. It follows the existing write-buffer-check source loader and keeps its outputs separate.

After the resource owner lifts any disk pause, run from the repository root with Node24+:

```text
node scripts/native-queue-order-oracle.mjs
```

The command transforms only type/module syntax and imports of the actual reference into tmp/native-queue-order/<runid>/. It runs two deterministic two-queue cases: already-resolved and rejected A handlers while B has a ready timer. A real macrotask fence drains Promise/queueMicrotask work between manually fired source timers. Do not replace it with sleeps or a guessed fixed number of Promise awaits.

The runner writes a unique results/native-queue-order/runs/<runid>/oracle.json and latest.json, marking running before source setup. Source hashes, complete action/callback/parsed/error logs and pending/task counters identify its scope. Failed attempts remain retained. Rust execution is explicitly unrun in this source-only report.

Antigone owns native pump implementation/regressions under task128. Task169 independently compares the source traces with the current native tests, then adds pending direct-reset/disposal checks. Do not edit another owner's native files or normalize away unexpected error/callback events.

All shell commands need explicit timeouts. Respect .cargo/config.toml and shared disk policy before any native tests. The source script and native results were initially UNRUN during the disk pause; only observed receipts can replace that status. Native results do not prove browser scheduling or native Linux/macOS runtime.
