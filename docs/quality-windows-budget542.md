# Windows formatting budget at 514 targets

Overall equivalence remains NOT PROVEN. Task 542 repairs a hosted Windows checkout-path regression found while preparing final quality task 304. It does not claim a new hosted run or completion of task 304.

## Current failure and retained baseline

At integrated source `0442e3578a7f57d35c2ae6ff20d7186820983065`, locked Cargo metadata lists 514 edition-2021 target roots. The previous hosted run used physical `D:/a/_temp/q`. Applying the current target inventory to that observed path requires 33,681 file-argument characters, including Windows canonical-path prefixes, quotes and separators. This exceeds the 32,767-character Windows process limit before adding the executable or other arguments.

Task 433's repair worked at its earlier source. The retained run `42090157228` actually passed full formatting with 444 roots and 28,872 characters. That earlier pass does not establish that today's larger inventory fits. I reran its unchanged read-only artifact audit. All eight ZIPs, 116 entry CRCs and 22 checksums passed. Seven jobs passed, but WASM strict Clippy failed at the then-current atlas-page test. The old failure remains unchanged. No new current-stable or native Linux/macOS result is claimed.

The exact local `cargo fmt --all -- --check` command also fails with OS206 at the owned physical path `O:/xterm-rust-swarm-16/tmp/Tahmasp304`. Its raw output and exit status are retained. This is not reported as a formatting pass.

## Repair

Both Windows setup and cleanup now select the canonical physical sibling of `RUNNER_TEMP`, using `[IO.Path]::GetFullPath` on `RUNNER_TEMP/../q`. At the previously observed runner layout, the result is `D:/a/q`. The current 514-root inventory requires 30,597 file-argument characters there, leaving 2,170 characters before the Windows limit. The regression test reserves another 512 characters for the executable and non-file arguments.

The path-budget numbers are models, not a new observation of a hosted process. The existing workflow still records the actual physical root and actual target inventory before running the unchanged exact full cargo-fmt gate. If a future runner layout or target inventory no longer fits, the required gate must fail visibly rather than exclude targets.

No drive alias, RUSTFMT wrapper, package split, target exclusion, assertion change, Rust edition change or new dependency was introduced. Linux and macOS checkout paths and all native/WASM/Miri gate commands remain unchanged.

Setup still refuses any existing destination before publishing its ownership marker. It checks out the exact `GITHUB_SHA`. Artifact upload still occurs before cleanup. Cleanup requires the exact selected path marker, rejects a linked root and child reparse points, and removes only the owned Git worktree. Moving one parent up does not authorize deleting that parent or `RUNNER_TEMP`.

## Tests and negative control

The existing 55 native Windows Node verifier cases remain intact. Two new cases bring the total to 57, all passing with zero failures, skips or cancellations:

- A platform-neutral regression inventories the actual workspace using locked Cargo metadata and checks the shortened modeled command line for every edition group.
- Native Windows executes the exact workflow PowerShell with a trailing separator in `RUNNER_TEMP` and verifies that it resolves to the same protected checkout.

The existing native fixture now expects the shorter sibling path. It retains immutable-SHA, actual metadata-budget, existing-path refusal, wrong-marker refusal, root-junction refusal, child-junction refusal and failed-checkout cleanup assertions. A sentinel inside `RUNNER_TEMP` additionally verifies that owned-checkout cleanup does not remove its sibling data.

For the targeted negative control, I temporarily restored the exact original workflow bytes from `0442e357`, ran only the new inventory test and observed exit 1 at the expected old-path assertion. A finally block restored the fixed workflow bytes before the complete passing run. An earlier broad old-workflow run also failed and is retained rather than hidden. No original source or upstream assertion was modified.

Reproduce the light verifier run from the repository root with Git, Cargo and Node 24 in PATH:

```sh
node --test scripts/quality-process.test.mjs scripts/quality-policy.test.mjs scripts/quality-snapshot.test.mjs scripts/quality-managed.test.mjs scripts/build-queue-retired.test.mjs scripts/target-relocation.test.mjs scripts/windows-quality-worktree.test.mjs
```

All fixture paths are under the owning checkout's `tmp/`, and fixture cleanup runs in finally blocks. These tests invoke Cargo metadata, not a Rust compiler. Native Windows executed the PowerShell lifecycle tests. Other operating systems were not run locally.

## Evidence and next execution

`results/quality/tahmasp304-preflight/` retains the exact starting revision, raw metadata, both modeled budgets, prior artifact audit, old-workflow failures, passing verifier output and exact local cargo-fmt OS206 receipt. The preflight verifies all 776 reference hashes against both the owned physical reference copy and original reference. Its tool hashes describe the initial source before the workflow repair, not the edited source.

The read-only GitHub run listing showed no active run or run for `0442e357`. No push, dispatch, compiler, browser, server or new CI execution occurred. No credentials were copied or printed. The coordinator must accept and integrate this repair, freeze an explicit source revision, then authorize a fresh unique-ref CI execution. Task 304 still requires actual Windows/Linux/macOS MSRV/stable quality results, release/features/doctests and WASM outcomes on the chosen integrated source.

## Temporary checkout recovery

The initial worktree creator exceeded its 120-second outer deadline while checking out tracked files. Inspection found the exact new branch at `0442e357`, an initializing lock, no index, a stale private index lock and no surviving Git or creator process. I removed only that verified stale private lock, completed the interrupted checkout from the same HEAD, unlocked the worktree and let the existing helper create its regular private target and physical reference copy. I did not repeat worktree creation or alter main. An initial attempted completion refused the stale lock and made no checkout changes.

An inline preflight script had a syntax error before executing any statements. Its corrected file-based version completed. Neither event is a Rust gate result. The only owned temporary root is `O:/xterm-rust-swarm-16/tmp/Tahmasp304`, including its empty private target and private scratch. No permissions from earlier canceled CI grants were reused. The root must be inspected for links, preserved commits, dirty files and workers before later removal.
