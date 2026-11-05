# Reviewed link, scroll and addon integration

Task435 is a delegated canonical integration from task309, not full application acceptance. The coordinator reserved the sole next main update after task415's isolated acceptance. The branch rebased on actual main `785a7781`, preserving the prior task429 batch and existing equivalent task405 mouseup repair.

## Frozen scope

- Task401 original source `84b418de` and independent review receipt `44839672`. The source cherry is `cc21a9a5`, receipt `ea02ff30`.
- Task415 accepted public custom-provider source through `475bc199`, accepted receipt `bbccfdcb`, independent execution receipt `96523601`. The rebase preserves the reviewed production bytes, including the corrected old-token re-registration behavior. See `docs/public-link-provider.md` for the exact scope and retained Firefox setup failure/replay.
- Task425 reviewed constructor-copy source `8b05bb23` and independent review receipt `fc43d1a0`.
- Task426 reviewed generic live Scrollable source `4e02103f`, independent review `509d866c` and cleanup receipt `484ab38f`.
- Task430 accepted search falsiness source `f6cc79de`, proof `4af9d7b6`, independent review `9296a876` and cleanup receipt `4f6ec6c5`.
- Task429 receipt-only commit `24074afc`. Its production is already in main and is not replayed.

`results/integration435/freeze.json` records the original chains, initial main and source freeze, plus38 exact producer-byte checks. It includes every task415 changed crate file, not only its final review fix. No producer patches conflicted. No unaccepted source was added. The final source freeze before receipt-only documentation is `569366f8`.

## Gate and scope

The existing `scripts/worktree-merge.mjs --main main` is the only authorized main update path. It rebases, checks clean tracked source, builds and tests changed crates, checks affected dependents and workspace/WASM libraries, and fast-forwards only the unchanged tested head/base. It removes only the regular private target on success. No gate or final-ref check is bypassed.

Native jobs use Cargo-j2 and one warm O: target. Browser suites are not repeated by this integration task. The accepted producer proofs remain scoped and retain their original source/build identities. Union checks and exact canonical outcome are recorded separately under `results/integration435/`; they must not be confused with earlier producer browser proof.

Exact workspace formatting is still a known Windows OS206 failure. Individual member format checks do not replace that command. Full source equivalence, final native Windows/Linux/macOS evidence, raw link/scroll/option family closure, demos and comparison deliverables remain unfinished.

## Observed canonical outcome

The unchanged helper fast-forwarded actual main from `785a7781` to frozen `dcde3e9c` and removed the regular private target. Its changed-crate default-feature gate passed534 tests across179 result summaries, with zero ignored tests. All affected/workspace/native/WASM checks passed. Separate union workspace all-target/all-feature tests passed2,086 tests across438 summaries, with zero failures or ignored tests. Strict native workspace and WASM xterm-web all-target/all-feature Clippy passed. All five package format checks passed; exact full formatting failed Windows OS206 and remains a failure.

The helper transcript, quality logs and exact counters are retained in the producer's receipt-only continuation under `results/integration435/`. They are not a new source change or another main update. Browser evidence remains the accepted scoped producer evidence; no union browser matrix was run. Task425's receipt-only cleanup commit `400f5a1e` arrived after the source freeze and is carried only in this post-gate receipt continuation, without restarting or relabeling the frozen gate.

## Cleanup

The owned worktree is `O:/xterm-rust-swarm-16/tmp/E415`, including its regular `target`, `tmp` and untracked generated browser outputs. Its copied reference is regular except the explicitly owned `source_repo/node_modules` junction to the pre-existing P: dependencies. That junction must be unlinked explicitly before any recursive cleanup. Reference sources, original dependencies, other worktrees and untracked integration `swarm/` content are outside the deletion set. Source and concise raw evidence are committed before cleanup.
