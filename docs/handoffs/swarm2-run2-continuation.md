# Same-swarm continuation for run 2

Overall equivalence remains NOT PROVEN. This is an update to swarm #2, not a new swarm or a restart of the port. The controlling request is `Update for run 2` in the swarm brief. `PORT_REQUIREMENTS.md`, the original handoff and all existing detailed acceptance checklists still apply.

## Inspected baseline before edits

Malcolm inspected the following before modifying documentation for task403:

- Actual integration was main at `5082b32a`. Its tracked status was clean. The existing untracked `swarm/` directory was preserved.
- Launch master remains the older checkout on P:. Its numerous tracked modifications and untracked files were inspected as retained unrelated work and left untouched. It is not the integration target.
- The old `swarm/Malcolm16` branch points to integrated `1c2bdbf3`, an ancestor of current main. Its former O: worktree remains absent, as recorded after cleanup. No old patch was replayed. The new owned documentation worktree is `O:/xterm-rust-swarm-16/tmp/worktrees/MalcolmRun2` on `swarm/MalcolmRun2`, based on current main.
- Read the updated brief, repository and global AGENTS instructions, current intent/acceptance plan, current worktree helper, current task403 and existing votes. The original handoff and repository snapshot were read earlier in this same session and remain the historical reference.
- Recent main includes font atlas canvas scanning `83dd5bf1`, Mac kern warning/async failure handling `a1d37ca6`, Base64 fallback host values `7cbc9d6b`, primitive OSC handlers `12ad65c9`, and compositionstart ordering `9648ed69`. These are already present; their old accepted patches must not be applied again. The user announced fixes during the stop, but this audit does not guess which commits were user-authored.
- Resource observation before any heavy job was 34.5 GiB available RAM and 2506.8 GiB free on O:. No live Cargo process appeared in that snapshot. This is a dated observation, not a reservation or permission to exceed the team cap.

This document records inspection, not new runtime passes. No engine, fixture, assertion, credential or AGENTS file changed for task403.

## Acceptance delta and existing work

Criterion485 adds one testable continuation obligation: reuse the existing graph/history, inspect current state before edits, preserve updates, and reconcile saved findings against actual source/evidence before replay. Task403 records the delta and intent; task309 remains the ongoing integration/resource supervision task. Ownership must be read from the live task graph, not old lane tables or old mentions in long threads. At inspection, task309 belongs to Vesna, not Malcolm or Rodrigo.

| Updated request | Acceptance and work retained |
| --- | --- |
| Same swarm, sessions, graph and prior progress | 485 with403 and ongoing309; no replacement graph, no duplicate inventory |
| Inspect main, owned state, instructions, recent changes before edits | 485; each owner records source identity and reconciles saved tests/findings in their existing task and review |
| Preserve user fixes, unrelated work and unread coordination | 485; inspect uncertain effects, avoid stale patch replay, blanket reset/clean and unknown process termination |
| Clear ownership and shared interfaces, efficient application fixes | 446/447 and309; use existing domain closure tasks312-325 and concrete current failures rather than more infrastructure |
| Four heavy jobs, Cargo -j2, at most two browser matrices | 453 and309; resource owner confirms live allocations, adjusts only from measured RAM/disk and reuses matching artifacts |
| New temporary work only under O:, cleanup even on failure/cancellation | 453 and309 plus every owner; track paths/processes, preserve regressions/evidence, inspect reparse points and never traverse reference junctions |
| Preserve source_repo assertions, AGENTS and credentials; bounded shell commands | Existing quality/integrity contracts and485; originals stay read-only, every bash call has a timeout |
| Full Rust functionality and thorough unchanged-suite/differential/E2E proof | 447/449 and complete family/327/328 work; no JS terminal-engine delegation or successful placeholders |
| Stable quality, actual native OS/browser matrices and complete comparisons | 448/451 with304;450 with327/330;452 with303/326/329;454 with328; overall NOT PROVEN until fulfilled |

Existing satisfied scoped criteria remain valid unless their source/task contract changes. Existing coverage votes must be reread when their criterion or task-link content changes. A covered vote concerns planned coverage, not successful application execution. Preserve actual failing cases and all discovered/mapped/executed/passed/failed/skipped/unrun accounting.

## Safe implementation and integration

Owners compare retained branch diffs and accepted source commits with current main before reusing them. Rebase only owned committed work and resolve conflicts against current behavior, not historical advice. Agree shared-file ownership before edits. Source-bound artifacts are reusable only when their Rust inputs match. Final integration uses the existing canonical helper, with changed-crate/dependent/native/WASM checks; no failed gate is waived.

Task403 is documentation-only. Its independent review checks this delta against the brief and existing graph. Application tests and final native/browser/quality/comparison matrices remain separate unfinished work. A missing implementation or an internal dependency is not an external blocker.
