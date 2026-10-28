# Acceptance contracts

These source-specific checklists, the live swarm criteria and their executable evidence are the project's acceptance requirements. `PORT_REQUIREMENTS.md` defines the full scope. `docs/acceptance-plan.md` groups work; its umbrella rows are not proof that individual source paths or edge cases are covered.

The first broad plan was reopened after user post 32. Planning task 2 remains unfinished until source/API/assertion gaps have explicit focused contracts. A large criterion count is not a completeness result.

## Required form for every behavior check

Each source method/branch must identify:

1. The exact read-only upstream source and test location, including shared helper and inherited API contracts where relevant.
2. Initial state and input domain, including JavaScript UTF-16/numeric conversions and typed Rust boundary decisions.
3. The action or sequence, including chunk splits, asynchronous suspension, reentrant callbacks, disposal and repeated calls.
4. Exact return/error, state mutations, object identity, emitted data/events and callback ordering. Error cases include the state committed before failure.
5. Every branch boundary and adjacent values, invalid/empty/overflow input, recovery after rejection and cancellation/disposal races. An expected no-op must be a source behavior, never an implementation placeholder.
6. The corresponding Rust test name and original assertion, plus deterministic differential input/seed and actual integration/E2E gate when another subsystem is required.
7. Reproduction command, source/build/fixture identity, observed counts/results and independent reviewer. Unwritten, failed, skipped and unrun checks remain distinguishable.

For example, Unicode width accumulation does not simply say "handles Unicode". Its contract specifies that a high surrogate followed by a non-low UTF-16 unit consumes both, calls the second unit's standalone width before the first unit's properties and does not retry that unit for joining. The persisted oracle verifies both the total and call history.

## Source coverage and metadata checks

- `results/acceptance/criteria-index.json` is an exported snapshot of live criterion revisions, exact source fields and task links. It is not a test-result file and becomes stale when the work graph changes.
- `node --test scripts/check-acceptance-index.test.mjs` checks the index auditor's positive and negative behavior.
- `node scripts/check-acceptance-index.mjs` validates the exported snapshot, linked work and exact file references. It rejects invalid/duplicate identifiers, missing fields, parent traversal and unexpected I/O errors. Missing/nonregular references or unlinked criteria produce failure, not a success percentage.
- The auditor never guesses a module's behavior from a task title or a directory reference. Cross-project quality, comparison and runtime requirements legitimately have no single original TypeScript method.
- Inventory task 22 owns `results/compatibility/{modules,apis,suites}.json` and explicit behavior-criterion rules. Every source/API/test entry must have a focused mapping or an explicit acceptance gap. A metadata reference is not implementation/test execution proof.
- Source owners must send exact criterion-ID/source-path rules to inventory task 22. A checklist covering one helper must not automatically mark every API in that family covered.
- Export the graph again after criterion source/link edits, then audit it. Final evidence requires a stable snapshot and an independent review of the actual checklists and assertions, not just an all-green metadata script.

## Detailed checklist locations

The criterion registry is the current ID index. This directory and related docs hold the longer branch/edge-case matrices:

| Family | Checklist |
| --- | --- |
| Unicode core, service and public Unicode view | [unicode-core.md](unicode-core.md) |
| UTF decoders and Unicode addons | [unicode-text.md](unicode-text.md) |
| Buffers, reflow, markers, service integration | [buffer.md](buffer.md), `docs/criteria/buffer-line.md`, `docs/buffer-reflow.md` |
| Charset maps and live integration | [charset.md](charset.md) |
| Events and CircularList | [events-circular-list.md](events-circular-list.md) |
| Lifecycle and native owner cleanup | [lifecycle.md](lifecycle.md) |
| Params and VT500 transition data | `docs/criteria/parser-storage.md` |
| Main parser, OSC, DCS and APC | `docs/escape-parser-acceptance.md`, `docs/osc.md`, [dcs.md](dcs.md), `docs/criteria/apc.md` |
| Cells/attributes and options | `docs/cell-acceptance.md`, `docs/options-acceptance.md` |
| Legacy, Kitty and browser keyboard/mouse/IME/clipboard | [legacy-keyboard.md](legacy-keyboard.md), `docs/kitty-acceptance.md`, `docs/browser-input-acceptance.md`, [mouse-state.md](mouse-state.md) |
| Engine handlers, write queue and public wrappers | `docs/engine-contract.md`, `docs/criteria/write-buffer.md`, [sgr.md](sgr.md), `docs/public-api-acceptance.md`, [public-adapters.md](public-adapters.md) |
| Common services and injection | `docs/core-service-acceptance.md`, `docs/service-injection-acceptance.md`, `docs/decoration-acceptance.md`, [osc-link-service.md](osc-link-service.md) |
| Browser DOM/rendering/colors/links/selection/scheduling | `docs/browser-acceptance.md`, `docs/acceptance-color-rendering.md`, `docs/link-acceptance.md`, `docs/selection-acceptance.md`, `docs/task-scheduling-acceptance.md` |
| Search, serialize and utility addons | `docs/acceptance-search.md`, `docs/serialize-acceptance.md`, `docs/utility-addons-acceptance.md` |
| Font and status addons | `docs/font-addon-acceptance.md`, `docs/status-addons-acceptance.md` |
| Native/browser demos and actual E2E | [demos.md](demos.md), [e2e.md](e2e.md) |
| Speed/size and classified LoC | [comparison.md](comparison.md), [loc.md](loc.md) |
| Quality, runtime, evidence and inventory integrity | [quality.md](quality.md), `docs/verification-acceptance.md`, [compatibility.md](compatibility.md) |

Image and WebGL source acceptance must be added by their family owners, not silently treated as covered by the other addon rows. Missing checklist/source/test mappings remain planning gaps until added and independently inspected.
