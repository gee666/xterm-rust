# Current-stable WASM quality repair

Task 389 repairs concrete failures exposed by the all-target WASM CI gate added in task 383. Overall equivalence remains NOT PROVEN. Task 388 retains and reruns hosted verification; final task 304 remains separate.

## Observed failed run

Run [42039966380](https://github.com/gee666/xterm-rust/actions/runs/42039966380) executed integrated revision `aa3f79211e48175f317261592ff57bd20e4dc79c` from unique non-force branch `ci/bohemond383-aa3f7921`. The workflow completed with FAILURE, not a green native/WASM checkpoint.

- WASM all-target/all-feature compilation passed on stable Rust 1.99.1. Warning-denied Clippy then failed on `screen_options.rs`. On wasm32, `MAX_BUFFER_SIZE` equals `usize::MAX`, making the old final `.min(MAX_BUFFER_SIZE)` redundant after saturating addition.
- Windows Rust 1.85.1 passed its tests and other quality gates, but its cold optimized all-target release build reached the exact 300-second command deadline. That command is FAILED. The termination metadata reports a timeout and successful owned-tree termination, with no compiler diagnostic or test assertion failure.
- The other five native MSRV/stable OS jobs passed. Each native test configuration reported 2,060 passing functions across 409 summaries, two doctests across five summaries, and 343 individual-feature passes across 126 summaries. Zero failed or ignored tests. The focused Miri job passed 21 tests. These results do not turn either failed job into success.

All eight original artifact ZIPs and job logs remain under `results/quality/ci-42039966380/`, with SHA-256 bindings and derived observations. Verification checked all 116 ZIP-entry CRCs and raw native per-harness counts.

## Repairs

The capacity computation still saturates the rows-plus-scrollback addition, then enforces the source's unsigned 32-bit limit with a checked conversion. This retains the meaningful clamp on 64-bit hosts without a redundant minimum on 32-bit WASM. It does not remove the source limit or use a target-specific lint allowance.

One assertion set exercises seven capacity boundaries through actual sparse Buffer construction on native Windows and wasm32. Cases include no scrollback, ordinary addition, the exact limit, addition beyond the limit and maximum-plus-maximum. Huge source-supported capacities remain sparse with zero initialized lines. The native and browser harnesses share the same assertions and independently name the source `u32::MAX` limit.

A private matching Rust 1.99.1 WASM `--keep-going` run then exposed four cloned single-element argument slices hidden behind the first library error. Two production calls in Kitty placement/image layers and two existing public-buffer assertions now use `std::slice::from_ref`. JavaScript object identity, call order, errors and every existing assertion remain unchanged. No new dependency, unsafe code, suppression or successful no-op was added.

The native CI command deadline is now explicitly 600 seconds, still within the existing 35-minute job deadline. This follows the observed 300-second cold-build failure and the existing local 600-second matrix policy. It changes no test assertion, workload, profile or release setting. The failed attempt remains preserved; hosted postrepair success is not inferred from a larger configured deadline.

## Local verification

Rust 1.85.1 native buffer regressions passed 33 functions, including the new shared boundary test. Native all-target/all-feature workspace warning-denied Clippy and exact full formatting passed. Full workspace WASM all-target/all-feature warning-denied Clippy passed on both Rust 1.85.1 and actual stable 1.99.1.

Final browser checks passed in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, with zero failed or ignored tests per engine:

| Suite | Passed functions per engine |
| --- | ---: |
| New shared capacity boundaries | 1, containing all seven cases |
| Public buffer numeric assertions | 7 |
| Live Kitty protocol | 19 |
| Image canvas | 23 |
| Image renderer lifecycle | 13 |

Image suites used the existing source-comparator wrapper, not the generic runner without its prelude. Browser runners closed their owned servers/browsers. Raw local logs, browser reports and input/artifact hashes are retained under `results/quality/stable-wasm-repair389/`. These are scoped checks, not complete image/buffer/application equivalence.

## Tool environment caveat

All explicit targets and scratch paths are under owned `O:/xterm-rust-swarm-16/tmp/Q`; TEMP, TMP and TMPDIR were exported separately to its `tmp/`. Rust 1.99.1 and its Clippy/WASM components were installed into private `tmp/rustup199`. The private compiler installation was intentional, but I omitted `--no-self-update`. Rustup automatically updated the shared manager from 1.29.0 to 1.29.3. The shared installed compiler list and default remain Rust 1.85.1 plus 1.97.1. No credentials were printed, copied or modified.

The automatic manager change is an unintended shared-tool side effect, not an owned temporary path. It is recorded rather than hidden or rolled back by deleting unknown files. Future private installs must use `--no-self-update`. The private compiler and all owned temporary outputs must be removed after integration and final hosted verification. This report does not claim that the shared manager was unchanged.
