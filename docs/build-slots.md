# Controlled local build slots

The shared P drive has exhausted its free space twice during the 45-worker port. Manual turn requests also blocked implementation work. `scripts/build-slot.mjs` provides two bounded local queues, one for native checks and one for WASM/browser checks. Both use the existing Cargo target directory and profiles.

```sh
node scripts/build-slot.mjs --owner Kassa-137 --lane native -- cargo test -p xterm-core --test input_handler_core_print
node scripts/build-slot.mjs --owner Example-browser --lane wasm --timeout-ms 420000 -- node scripts/browser-host-check.mjs
```

Set the existing toolchain PATH first if required by `docs/workspace.md`. Give the outer bash tool a timeout longer than queue wait plus command deadline plus cleanup. Defaults are 120 seconds waiting and 180 seconds executing. `--wait-ms` and `--timeout-ms` accept finite values from 100 to 1800000 milliseconds.

## Rules

- Waiters are ordered by host-monotonic enqueue ticks and unique ID within their lane. A queued command does not start until it owns that lane's directory lease.
- A native and a WASM command may hold different lanes concurrently. Cargo still serializes access to its shared build directory where needed.
- The wrapper checks at least 1 GiB free on the repository and existing target filesystem before acquisition, before execution, during execution and afterward. It does not clean anything automatically.
- Use scoped debug tests/checks with existing dependencies. The CLI rejects `--workspace`, `--all-targets`, `--release`, target-directory overrides and Cargo config overrides. Do not hide those operations inside a script. Full matrices and release gates run remotely or require separate coordination.
- Do not override the shared Cargo target or development/test/release profiles. A lease does not authorize installing dependencies or creating alternate output trees.
- Tiny source-only Node filesystem/fixture checks do not need a build lease when they create at most 20 MiB and invoke no compiler, large bundler or browser. Real build/bundler/browser commands use a lease.
- Queue deadlines use a monotonic clock, so wall-clock adjustment cannot extend them. Progress callbacks are bounded by that same wait deadline.
- Wait timeout, command failure, cancellation, process timeout, resource failure and lease ownership changes all fail. Failed logs are retained, not overwritten.

Each attempt writes its request, process result, disk observations and stdout/stderr under a unique `results/build-slots/` directory. This receipt proves command execution and coordination only. It does not prove terminal behavior; the actual test assertions and source/runtime evidence remain required.

## Cancellation and recovery

SIGINT/SIGTERM cooperatively abort the owned command through `quality-process.mjs`. The helper stops its owned tree and then releases the lease. A command deadline provides a second bound. Adapter code must still clean its browsers and servers in `finally`.

Stale queued requests whose owner process has exited can be removed because they own no active lease. A stale active lease is never stolen: its child might still run after an abrupt owner death. The wrapper fails closed and names the stale owner. A coordinator must inspect that owner and its descendants, confirm all owned work ended, retain the failed attempt, then remove only the corresponding `tmp/build-slots/<lane>/active.lock` and stale request. Never kill an unknown PID or remove another live owner's lease.

For disk recovery, `scripts/clean-shared-target.mjs` defaults to a real Cargo dry run after checking the exact directory, Cargo cache marker, source profile hash and process idleness. Applying a coordinated native-development cleanup requires `--apply --approved-by NAME`. It never removes source, tests, results, tmp, source_repo or private swarm data. Keep release and WASM artifacts when the narrower cleanup is enough, and report actual retained directories rather than assuming Cargo behavior.

The observed 2026-10-28 cleanup was approved by Chloe and independently reviewed by Aaliyah. `cargo clean --profile dev` removed 1.2 GiB from native development output. P free space rose from 901980160 to 2173661184 bytes; release output of 234 MiB and WASM output of 301 MiB remained. The profile SHA stayed unchanged. The exact receipt is `results/quality/resource-clean-1793147954300/receipt.json`.

Run helper checks with:

```sh
node --test scripts/build-slot-core.test.mjs scripts/build-slot-review.test.mjs
```

Queue self-tests are automation evidence, not Rust or browser parity results.
