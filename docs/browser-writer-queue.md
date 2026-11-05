# Browser write queue timing checks

Full equivalence is NOT PROVEN. This checkpoint adds tests, not a production
queue change. It does not close the retained uncontrolled-clock vt34 failure.

## What was checked

The source `src/common/input/WriteBuffer.ts` checks `performance.now()` after
each completed chunk. It yields once the batch has consumed at least 12ms.
Queued writes from a data listener therefore need not precede the next browser
render when that first chunk exceeds the budget. A source/Rust trace comparison
must record whether both implementations took the same slicing branch before
attributing an extra render to a queue-order defect.

`crates/xterm-web/tests/browser_writer_cases.js` exercises the real public
Terminal API. The Rust tests use the actual WASM terminal, not fake services or
the source engine. The source runner separately bundles the untouched supplied
browser Terminal and executes the same cases. Matching explicit assertions in
Rust and the source runner check event payloads, event-time cursor positions,
completion callbacks and final text.

Five scenarios run in Chromium, Firefox and WebKit:

1. An onData query listener queues `R` while the outer chunk writes `P` and `Q`.
   With the clock below the budget, both writes complete before onWriteParsed.
   The listener also checks that disableStdin suppresses reentrant user input.
2. The same listener advances the clock by 13ms. Both engines emit parsed at
   cursor 2, then complete `R` and emit parsed at cursor 3 in the next batch.
3. A write completion callback queues another write. Both completion callbacks
   precede the batch's parsed event.
4. Disposal cancels a queued write and its completion callback. A later timer
   sentinel verifies neither completion nor parsed fires.
5. A real opened browser terminal first awaits its setup render. Under the same
   below-budget clock as case 1, its first subsequent onRender sees cursor 3.
   This checks the actual Rust browser renderer, not only the native queue.

The clock controls only the protocol's slicing decision. Browser timers and
animation frames still run normally. The source and Rust assertions do not
filter events, reorder callbacks, or weaken the old vt34 assertions. These are
new source-derived tests, not executions of an unchanged upstream test suite.
The duplicated expected event arrays are intentional independent assertions
across the Rust test and source runner.

## Commands and evidence

Install the existing reference's Node dependencies and wasm-bindgen CLI 0.2.100.
Use a private worktree with Cargo, Rust's wasm32-unknown-unknown target, and
Playwright's three browser engines installed. Set TEMP/TMP to that worktree's
`tmp` directory. On this Windows run, CARGO_BUILD_JOBS was 2.

```sh
# REFERENCE_ROOT is optional when this worktree has source_repo/node_modules.
REFERENCE_ROOT=/path/to/unchanged/source_repo node scripts/terminal-writer-oracle.mjs
WASM_TEST=wasm_terminal_writer node scripts/browser-host-check.mjs
```

Set WASM_BINDGEN and PLAYWRIGHT_MODULE if the existing tools are outside the
worktree. The source runner removes its owned generated bundle in finally.
Browser-host-check retains generated artifacts by design. Remove owned generated
artifacts only after copying durable logs, counts and hashes.

`results/browser-writer/source.json` retains all five source captures per engine,
source input hashes and browser versions. `results/browser-writer/rust.json`
retains Rust test names, counts, versions, command outcomes and artifact hashes.
The Rust logs are beside that report. Both sides passed 15 executions total,
5 per engine. No tests were ignored, filtered or skipped. This is scoped Windows
browser evidence, not native Linux/macOS, all-feature, benchmark or full-suite
proof.

## Retained failure and next work

The old captures remain untouched at
`C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Sophie/tmp/browser-modes/vt-34/`.
The original scenario is in that branch's
`scripts/acceptance-adapters/terminal/mode-inputs.mjs`. Its input is retained in
these tests as well. The old uncontrolled captures include an extra Rust render
at cursor 2 before queued `R`, unlike the source capture. The new tests do not
establish why that old run took different render turns. In particular, they do
not establish that the old run exceeded 12ms.

A fresh full-state/RGBA/event capture with the original assertions, plus clock
observations around each actual chunk, still belongs to browser root closure
task 317. Do not replace that comparison with these smaller captures. Async
parser-handler reentry, resize inside parser callbacks, and the complete original
WriteBuffer suite also remain outside these five cases.

Njoya read the supplied handoff, repository snapshot, PORT_REQUIREMENTS.md and
AGENTS.md, then used current main in an O: private worktree. Ownership was limited
to queue tests, their source oracle and this report. No production Root, input,
scroll, decoration or renderer files changed.
