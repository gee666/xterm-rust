# Resuming run 2

The updated brief keeps the original 45-worker swarm and the full scope in `PORT_REQUIREMENTS.md`. It does not authorize a second swarm, a reduced utility-only port, or completion based on acceptance documents alone. Existing tasks, claims, source, tests, results, commits and independent review findings remain authoritative. Re-read current files and task revisions before replaying writes.

Main reported that drive P filled to about 7 MB and caused SQLite's `cannot rollback - no transaction is active`. Main's database quick check passed. Main stopped workers cleanly and used `cargo clean` to remove about 3 GiB of generated build artifacts, not source or evidence. These are Main's reported recovery actions, not a new diagnosis by another worker. On resume Cornelia independently observed about 3.0 GiB free with `df -h .` and read `.cargo/config.toml` at commit `705104c`.

The shared configuration disables development/test debug symbols and incremental caches. Assertions remain enabled. Release settings remain unchanged for valid performance and artifact comparisons. Do not override these profiles or create duplicate target directories casually. Check free space before large builds and report low space before another exhaustion. Criterion 237 and quality task 47 cover the shared resource preflight and evidence. No cleanup may remove source, tests, evidence, credentials or another worker's active output without checking ownership.

## Existing acceptance mapping remains required

- Criteria 1 and detailed subsystem criteria still require full path/edge-case acceptance and independent review. Task 89 records the update rather than replacing the acceptance plan.
- Criterion 2 and implementation tasks 26-43, their detailed criteria and supporting component tasks require real terminal, headless, browser/WASM and addon integration. Missing implementation is work, not an external blocker. Resolve interface dependencies and continue implementation.
- Criteria 4-7 and tasks 44-47 require original and Rust assertion suites, deterministic/randomized differential checks, working native/browser demos and actual end-to-end execution. Documents and smoke tests do not substitute for those deliverables.
- Criteria 8-9 and tasks 48-49 still require matched-workload speed/artifact-size comparisons and classified LoC with raw reproducible evidence. Preserve release build settings and workload validation.
- Criterion 10 and task 50 audit exact evidence and review findings. Native Linux/macOS execution cannot be inferred from cross-compilation or configured CI. A required check remains unrun until an observed result exists.

Overall equivalence remains NOT PROVEN until the complete acceptance gates have demonstrated evidence. Keep prior counterexamples and change requests, finish outstanding review, and collect fresh coverage votes after content/link changes. Lifecycle transitions alone do not require inventing a new implementation task.
