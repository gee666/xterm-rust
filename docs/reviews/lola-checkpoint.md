# Lola review checkpoint

Overall equivalence is NOT PROVEN. These are scoped independent review results, not a workspace pass.

## OSC tasks 220 and 238

Inspected `osc.rs`, `osc_registry.rs`, shared `handler_registry.rs` and `handler_slots.rs`, raw failure tests, disposal tests, and unchanged `source_repo/src/common/parser/OscParser.ts`. The new disposal handle clears registration, fallback and active properties without resetting parser state or restoring callbacks. Raw start/put failures propagate before later callbacks, retaining partial parser state.

`node scripts/check-osc.mjs` passed on Windows with Node 24.15.0. It includes 23 original tests, 512 traces of 96 operations, and 18 raw-handler failure fixtures. Record comparison uses `canonicalTsvRecords`, while raw and canonical hashes remain separate. Log: `results/reviews/lola/osc-source.log`. Six additional unchanged-source disposal traces match the expectations in `osc_disposal.rs`, covering fallback removal/new registrations and active-list errors in start/put/end. Results: `results/reviews/lola/osc-disposal-source.json`.

Created a private worktree through `node scripts/worktree-create.mjs --agent LolaReview`. Base `4cf030c` failed to compile with 24 terminal/charset/write-scheduler integration errors. After producer commits, `eaa1d28` stopped at a lock/manifest mismatch. Both failures are retained. Owners fixed and committed those dependencies.

At `23760fa`, `cargo test --locked -p xterm-core` with the nine OSC/outer-OSC test targets passed all 39 tests. Adding `--test mouse_button_numeric` passed one more test covering 26 numeric expectations. Focused Clippy over those targets passed with `-D warnings`. Logs: `results/reviews/lola/osc-23760fa-{tests,clippy}.log`. Task 220 was independently accepted. Task 238 had already been accepted by another reviewer, so no duplicate decision was issued.

## Image tasks 156 and 224

Inspected native IIP transfer/geometry/report code, options/control policy, tests, replay examples and unchanged IIPHandler/ImageAddon source. The implementation and documents distinguish display requests from actual decoded pixels and browser execution.

At private committed checkpoint `23760fa`, independently passed `cargo test --locked -p xterm-addons --test image_iip --test image_policy`: 10 IIP and 7 policy tests. `node scripts/image-iip-oracle.mjs` passed 9925 comparisons, and `node scripts/image-policy-oracle.mjs` passed 2357. Focused Clippy including both replay examples passed with `-D warnings`. Tasks 156 and 224 were accepted for these native components only. Logs: `results/reviews/lola/image-23760fa-{tests,clippy}.log`, `image-iip-oracle.log` and `image-policy-oracle.log`. Actual decoded pixels, storage and browser integration remain separate gates.

## Native cleanup task 179

Inspected the native cleanup scheduler, queued terminal factory wiring, ownership slots, host and error-identity regressions. Madeline independently accepted task 179 before my duplicate assignment was processed; I did not issue a second decision. Producer and consumer are now committed as `ed485b0` and `f36a853`. My committed-checkpoint runtime rerun remains pending.

## Acceptance runner task 140

Independently ran `node --test scripts/acceptance-runner.test.mjs`, 19 passed with zero failures/skips. Also ran `node --test scripts/acceptance-artifacts.test.mjs`, 6 passed with zero failures/skips. These are runner self-tests, not actual terminal/browser acceptance cases.

Found an integration defect with the required worktree workflow. Worktree creation installs `source_repo` as a junction, but `acceptance-validation.mjs` uses `regularFile` for source hashes and rejects any symbolic ancestor. Calling `regularFile` on the real LolaReview worktree and `source_repo/src/common/parser/OscParser.ts` fails with `nonregular or symbolic reference`. Before correction, every real worktree acceptance result failed source validation. Lukeni added `referenceFile`, resolving only the deliberately shared source root while retaining nested-link and artifact/log rejection. I inspected the correction, independently passed 20 runner and 6 artifact tests, and verified the real LolaReview source path now resolves. Task 140 was accepted and its correction committed as `c927f34`. No terminal/browser acceptance case is counted from these 26 infrastructure tests.

Logs: `results/reviews/lola/acceptance-runner-tests.log`, `acceptance-artifacts-tests.log`, `worktree-reference-probe.log`, and `acceptance-followup-tests.log`.

## Worktree tooling task 245

Read the create/isolation/target-cleanup implementation and the review probe. Independently ran `node --test scripts/worktree.test.mjs`: four passed, including an actual tiny crate build/test/fast-forward and target removal. Log: `results/reviews/lola/worktree-tests.log`. The separate create probe needed refresh after creation began precreating its target. Timur was notified; the old review document still said its tests were unrun. No unsafe raw removal of a worktree containing the shared reference junction was attempted.

## Cell and adapter coverage votes

Task 73 was already independently accepted by Euphemia before a duplicate assignment reached me. I did not issue a second acceptance. Read its C1-C5 acceptance matrix and current payload Drop fix; coverage votes distinguish the linked plan from test completion. Public adapter votes retain all A/B/P source and browser requirements. They do not mark pending browser, release, platform or GC checks passed.
