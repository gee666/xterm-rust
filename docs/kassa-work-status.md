# Kassa work status

Overall xterm.js equivalence is NOT PROVEN. This status separates accepted infrastructure, observed failures, current tests and remaining work.

## Task 47, native quality and platform CI

The runner, six native OS/toolchain matrix jobs, WASM compilation job, focused Miri job, exact logs/host validation, feature matrix and failure/resource guards are implemented. Commits include4cbbd8c,e18b1d1,7162a1d,a795b84. Two actual remote runs were observed and retained, not inferred from YAML:

-40604393672 at4cbbd8c: native Node helper tests passed, Rust preflight failed the now-fixed CARGO_INCREMENTAL=0 guard; WASM exposed uncommitted source.
-40608206436 at52cca2a: native Node helper tests passed on all six jobs; actual Rust builds/tests could not complete because that immutable snapshot had stale EmitError constructors and missing Buffer shared methods. Miri setup ran but the crate did not compile. Eight artifacts and extracted logs are retained under results/quality/ci-40608206436/.

Remaining work is an updated coherent committed snapshot, a third actual remote run, resolution of all build/format/lint/test failures, actual passing MSRV/stable Windows/Linux/macOS results, WASM and applicable Miri results, final evidence update and independent review. No full-platform Rust pass is currently claimed. Native browser runtime and demos remain separate tasks.

## Task 114, managed processes and cancellation

The owned process lifecycle implementation, readiness/error/timeout/resource handling, ENOSPC logging guard, exact cleanup and shared tree helper were reviewed and accepted once. It was reopened after identifying that a Unix group-only kill can miss nested detached groups. The implementation now inventories PID/PPID descendants, supports cooperative AbortSignal cancellation and has regressions for both inherited and detached child groups. Thirty current local Node helper tests pass in results/quality/runner-tests-nested-abort-current.log. Windows is the observed runtime; actual updated Unix helper runs remain part of task47's next CI matrix. Request independent re-review of the changed helper, not a claim that the whole terminal passed.

## Task 137, InputHandler print/control assertions

The unchanged original suite selection executes24cases/81assertions. Faithful Rust ports cover8print,3ISO-2022,5wide-cell edit and8reverse-wrap/backspace cases. A further54directed real-engine traces compare221full buffer/cell/cursor/margin/mode/event snapshots, including2a11y callback-time states. All26Rust tests now pass after fixing an observer-only event-queue consumption bug; production assertions were not weakened. The latest retained local attempt is results/input-handler-core/native-attempt-3.log. A prior failure and shared-API compilation failure are preserved separately.

The exact vt-22 adapter is written under scripts/acceptance-adapters/terminal/print-control.cases.mjs and pins a regular executable before execution/hash. Remaining work is executing the required acceptance-oracle --case vt-22 command, scoped Clippy and formatting checks, final source/verification mapping, selective commit and independent review. This24case selection does not claim the rest of InputHandler.test.ts is migrated.

## Accepted coordination and recovery

Task190's reviewed native/WASM FIFO queues are in scripts/build-slot*.mjs and docs/build-slots.md. Task218's one-cache relocation is accepted:2608files/1558942761bytes, six exact byte/mode/nanosecond-time checks, unchanged controls and successful pre/post Cargo metadata. Logical P:target remains unchanged through a junction to the single persistent C cache. No source/auth/evidence/profile migration occurred. See docs/target-relocation.md. Continue scoped debug checks through190; no target/env/profile overrides, arbitrary cleanup or hidden local full/release matrices.
