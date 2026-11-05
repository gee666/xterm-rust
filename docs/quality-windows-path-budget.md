# Windows formatting path budget

Overall equivalence remains NOT PROVEN. Task 433 repairs the hosted Windows checkout layout. Task 304 still requires observed full native gates on a stable integrated revision. No new hosted run is claimed here.

## Reproduced failure

At integrated source `785a7781f369e8cb5f9331387c5fbe2eea89f995`, both installed Windows toolchains, Rust 1.85.1 and 1.97.1, fail:

```sh
cargo fmt --all -- --check
```

The result is exit 1 with `The filename or extension is too long. (os error 206)`. This is not a formatting diff. All 436 distinct target roots use Rust edition 2021. At the allowed physical worktree `O:/xterm-rust-swarm-16/tmp/X304`, their paths require 34,847 characters with conservative quoting, before the Windows canonical-path prefix. cargo-fmt canonicalizes these paths and starts one rustfmt process per edition. Windows cannot launch that command line.

The earlier physical short-path remedy in `docs/quality-bohemond16.md` worked for 354 roots. It no longer fits the current target inventory. A drive alias does not shorten canonical paths. A RUSTFMT adapter would also receive the oversized argument list, so it cannot fix this initial spawn. No adapter, root consolidation, assertion change or package exclusion was introduced.

The earlier `926d8355` checkpoint has separate evidence for 431 roots. It is not used as evidence for the later source.

## Full source checks, with distinct command accounting

At `785a7781`, all five package checks pass on both toolchains:

```sh
cargo +1.85.1 fmt -p PACKAGE -- --check
cargo +1.97.1 fmt -p PACKAGE -- --check
```

PACKAGE is each of xterm-core, xterm-web, xterm-addons, xterm-wasm-support and xterm-font-addons. The retained inventory records every target root, kind, relative path and file hash.

A second check invokes the actual toolchain rustfmt once for all 436 roots, using equivalent relative file paths. Its 20,949-character argument list fits Windows. Both toolchains pass. `direct-rustfmt-summary.json` contains the complete executable arguments, count, log path and exit status. This demonstrates full-root formatting, but neither this check nor the package checks are reported as a successful exact cargo-fmt command.

Raw receipts are in `results/quality/x304-format/785a7781/`. The exact full cargo-fmt failure remains retained there. This checkpoint does not establish Clippy, tests, release builds, other operating systems or browser runtime results.

## Hosted Windows repair

The native workflow creates a fresh detached worktree at `RUNNER_TEMP/q` only on Windows. Git checks out the exact `GITHUB_SHA`, not a moving branch. All existing verifier, relocation probe and native quality commands run there without changing their arguments. Linux and macOS continue using their original checkout. The native gate runner still executes the exact full cargo-fmt command and still fails the job on any required gate failure.

The Windows job prints its actual physical path and per-edition target count and quoted file-argument budget before running the verifier. A previous hosted Windows log records the default checkout at `D:/a/xterm-rust/xterm-rust`. Applying the current 436-root inventory to that path yields 33,975 quoted file-argument characters including canonical prefixes. The modeled `D:/a/_temp/q` path yields 27,871. These are path-budget calculations, not observations of a new hosted run. The new job records the actual runner path rather than assuming it matches this model.

Artifacts upload from the selected worktree before cleanup. Cleanup requires the ownership marker to match `RUNNER_TEMP/q`, rejects a linked or non-directory root before enumeration, and rejects child reparse points before removal. A pre-existing path is refused, never adopted or deleted. Cleanup runs after failed steps too. Cleanup failures remain visible rather than triggering broader deletion.

## Local validation

The unchanged six verifier suites plus `scripts/windows-quality-worktree.test.mjs` pass on native Windows: 55 tests, zero failed, skipped or cancelled. The new tests execute the exact YAML PowerShell against tiny owned Git fixtures. They check requested-SHA identity, the actual metadata budget, existing-path refusal, incorrect ownership markers, root and child junction refusal, generated-file cleanup and failed-checkout cleanup. Every fixture is removed in finally cleanup. No compiler or browser job was started.

An initial test-fixture failure is retained in `workflow-tests-first.log`. The fixture inherited global CRLF checkout conversion. Its private repository now explicitly sets `core.autocrlf=false`; the byte-preservation assertion was not weakened.

```sh
node --test scripts/quality-process.test.mjs scripts/quality-policy.test.mjs scripts/quality-snapshot.test.mjs scripts/quality-managed.test.mjs scripts/build-queue-retired.test.mjs scripts/target-relocation.test.mjs scripts/windows-quality-worktree.test.mjs
```

The new PowerShell execution test is Windows-only. Its platform-neutral command and artifact assertions run on every platform. A successful local fixture test or workflow edit is not hosted CI evidence. The new hosted matrix, including current stable Rust, remains unrun here and belongs to task 304.

## Temporary ownership

Only `O:/xterm-rust-swarm-16/tmp/X304` and its own target/tmp directories are owned locally. No short root outside the authorized run parent was created. No background process is owned. Preserve the branch and durable results, inspect all links and dirty files, then remove only this worktree after integration or handoff. Local full cargo-fmt remains failed unless a separately authorized shorter physical root or a later upstream tool fix is actually tested.
