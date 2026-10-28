# Queue oracle startup and fixture guards

Task 199 hardens these entrypoints:

- `scripts/write-buffer-oracle.mjs`
- `scripts/write-buffer-evidence.mjs`
- `scripts/native-queue-oracle.mjs`
- `scripts/input-handler-async-baseline.mjs`
- `scripts/input-handler-async-traces.mjs`

Each writes an incomplete receipt immediately after resolving its repository root, before temporary directories, local helper imports, bundler dependencies, fixture reads or toolchain commands. An uncaught-exception monitor records the original failure and does not suppress Node's nonzero exit. A successful receipt is written only after its assertions and fixture checks finish. Primitive thrown values are represented too.

The short startup block deliberately remains in each CLI rather than a new imported framework. A missing local helper must fail after the old pass has already been invalidated; keeping that ordering at the entrypoint makes it auditable.

The three fixture-producing oracles now reuse `scripts/oracle-fixture.mjs`. Normal runs verify existing data and reject drift. Regeneration requires `--generate`. This prevents verification from silently replacing expected data. The first InputHandler lifecycle fixture must be generated explicitly, then retained and reviewed before normal verification.

The WriteBuffer evidence checker requires a passed current source receipt and exactly 48 Rust checks after the timer-token, cancellation and host-reentry follow-up. The older retained 42-test result is a historical snapshot, not validation of changed source. It records exact implementation/test/fixture/log hashes and does not promote scoped evidence to full project equivalence.

## Pending validation

The startup scaffolding is implemented. The lifecycle source generator has since executed successfully in a controlled slot and generated six real source scenarios. Negative startup/fixture probes remain UNRUN. The task is released for an automation reviewer to extend the existing `scripts/check-oracle-startup.mjs` checker, not introduce another framework. Workload regions stay with tasks 174/150/209; this handoff owns startup receipt ordering and negative-checker definitions. Current WriteBuffer evidence depends on task 174's fresh test receipt.

Required negative observations for each applicable oracle:

1. Seed an old passing receipt, break the reference or bundler dependency, and require a nonzero exit with a new failed/incomplete receipt.
2. Seed an old passing receipt, block its temporary path with a regular file, and require the same fail-closed outcome without a compiler service starting.
3. Supply a corrupt or absent fixture on a normal verification run. Require failure without changing that fixture.
4. Verify explicit regeneration separately and rerun current positive source/native checks with exact fixture and source hashes.

The evidence-only checker has no temporary directory or bundler dependency. Its negative checks must corrupt/miss required source receipts or result logs and require a nonzero exit plus a failed/incomplete verification receipt. A monitor alone is not passing evidence; the checker must observe the actual child exit and final file contents.

No new builder may run until the quality owner clears or assigns a serialized disk-safe window. Existing historical evidence is preserved. Overall equivalence remains NOT PROVEN.
