# Taliesin task564

Task564 ACCEPTED, no review findings. Own source75eb88ce and evidence74daf5ef, followed by this receipt-only handoff/review/preflight commit. Preserve on swarm/TaliesinSelection and archive/Taliesin564-complete. Base main224d1b33.

## Integration

Pick ONLY task564's own commits after accepted original545 and557. Exclude all seven private prerequisites in this branch:545copies6e098b56/6cce7d04/26eb5c2f/6c44f125 and557copiesd5369e5a/4f1d1e2f/bd0d960c. Do not append to the frozen554 batch. Canonical309 owner chooses the future batch and reruns its integration gates.

Production changes only `crates/xterm-web/src/selection/dom.rs` and `dom_schedule.rs`. Stable JS-owned Fn callbacks capture Weak<State>. Teardown removes move then up on the screen's current owner document, then calls catch-enabled real-window clearInterval with live slot or undefined, then clears the slot only after success. Host callbacks run without a RefCell borrow. No dependency, unsafe, terminal-engine delegation or source/assertion edit.

## Evidence

See `docs/selection-drag-teardown.md` and `results/selection-drag-teardown`. Shared public Terminal trace has9 cases, plain/throwonce/reenterselect at move/up/timer phases, each source and final actualRust passing in Chromium, Firefox and WebKit. One WASM test function expands9 cases. The unchanged oldproduction control fails in Chromium at intended timer-first and missing-retry-removal traces, with0 pageerrors. Old/final compiler/servedWASM/binding hashes differ; identical test/program/fixture inputs are verified, and both compiler artifacts embed exact program+fixture.

Native all-target/all-feature web432PASS across185 test executables, including zero-test native builds of WASM targets/examples. Strictnative/WASMlib+five browser testtargetsPASS. Retained teardown545 onefunction18cases, geometry557 onefunction8cases, selection8functions and TerminalSelection1 each passed3 engines. Fullworkspacefmt failsOS206; wholewebmemberfmt passes. These are not fullworkspace, nativeOS or fullsource-suite results.

Reviewer reran5 pinned nativeEXEs37PASS, changedfile rustfmt and read-onlyaudit1461 inputs/8frozen/73execution receipts/21runtime+776original/776copy. Finalpostreview receipts extend that snapshot. No additional compiler/browser launch occurred in review.

## Cleanup and remaining work

Compiler/browser/server grants were released on post409/comment4991,0jobs. The preflight is durable; final actual cleanup result lives privately in `.pi/swarm/work/2/taliesin564-cleanup.json`. Do not replay removal or infer a still-live worktree from historical binary paths. Cleanup verifies the archive, source and original reference before and after removing owned outputs and the normal owned git worktree. Runtime pins are historical after removal.

Full321 retains repeated-mousedown interval lifetime, RAF/options/raw numeric/DOM/disposal/GC and whole-suite closure. The existing Rust drag path stops the previous drag before registration. Source may orphan older intervals on repeated mousedown, and this task does not emulate that leak. Search527, full304 nativeOS/stable quality, source mapping/comparisons and full equivalence remain OPEN. Image private ABI ordinary failures were passed to task569; no presence-only facade was added here.
