# Original baseline and differential acceptance matrix

The reference is the supplied `source_repo` tree, not an assumed release checkout. Its package identifies version 6.0.0. Preserve original implementation, tests and fixtures. Installation and build output may change; original content may not.

## Source integrity and setup

- Retain a tracked sorted manifest of every original file with relative path, byte size and SHA-256. Match all 776 initially inventoried files from `tmp/source-manifest.json`; do not invent a public revision. Exclude only dependencies and known generated output when discovering additions.
- Before and after every automated baseline, verify every manifest entry exists and hashes identically. Missing files, changed assertions, modified fixtures, lockfile changes and changed source all fail integrity. No baseline is accepted if integrity fails even when Mocha passes.
- Record Node, npm, OS/architecture, exact commands and package-lock hash. Use the original lockfile. Explain `--ignore-scripts`: unit/bundle tests do not need native PTY installation, but later demo tests may require a real PTY build.
- Use original esbuild scripts and original Mocha runner. Build core and all 14 addons. Missing dependencies or a failed build must return a nonzero status and retain stdout/stderr; never substitute stale test outputs.

## Unit baseline

- Execute unchanged original assertion bodies through the supplied `bin/test_unit.js` runner. Use Mocha's JSON output file, because the tests themselves write escape replies to stdout. Keep raw stdout/stderr separately; do not parse or discard them to fabricate a JSON result.
- Discover every source `.test.ts` module under core and addon `src`. Map its generated `out-esbuild/**/*.test.js` path back to exactly one source path. Fail on an executed file without source provenance or an expected compiled test module missing after build.
- Retain full individual Mocha test records including title, full title, file, duration, pending and failure details. Summaries must provide per-source-file counts plus total discovered, executed, passed, failed, pending, skipped and unrun. Pending is an upstream state, not a pass and not a missing Rust assertion port.
- For the supplied reference, initial observed totals are 422 Mocha suites and 2,460 discovered cases: 2,389 passed, 71 pending, zero failed. Verify summaries against raw result arrays. These counts are a reference fingerprint, not permission to silently omit new or changed tests.
- Enumerate support-only `.test.ts` modules that define no executable cases. Distinguish that from missing discovery. For every source pending case retain the exact name and source path. Do not enable or disable source tests to change the baseline.
- Run every test-bearing core and addon module even if its corresponding Rust implementation is absent. Report Rust migration independently; original JS success alone counts as zero Rust execution.

## Original browser baselines

- Discover all configs from `bin/test_integration.js`, all original Playwright projects, test files and individual cases. Compare list-mode discovery with runtime reports. Include addons with no unit tests rather than treating them as covered by Mocha.
- Execute each applicable Chromium, Firefox and WebKit project against the original terminal with its real demo/server prerequisite. Record per-project, per-suite and per-case pass/failure/pending/skipped/unrun counts, browser versions and screenshots/traces for failures.
- A missing browser executable, graphics capability, native PTY, network fixture or server is an explicit failure or external blocker with attempted resolutions. Browser startup alone is not a passed suite. Keep original baseline and Rust-backed browser evidence separate.
- Any managed server must have an explicit shutdown, a readiness condition rather than a sleep, and a bounded lifetime. Never leave background processes running after the suite.

## Shared Rust/JS operation traces

- A Rust runner must call the actual Rust engine, never delegate terminal logic to JS. Both runners consume the exact same serialized operation list, input bytes, dimensions, scrollback and Unicode provider. Record schema version and hashes.
- Snapshot every cell's codepoints or UTF-16 content, width, foreground/background/extended attributes, line wrap flag, buffer dimensions, normal/alternate buffers, saved/active cursor, scrollback base/viewport, modes and tab stops. Compare generated replies, data/binary events, callback order, API results, marker/decoration disposal and error results separately.
- Directed cases cover print/control characters, all escape parser families, malformed sequences, cancellation, parser hooks and async continuation, UTF-8 overlong/invalid/truncated input, every UTF-16 surrogate boundary, Unicode width/grapheme joins, wrap/insert/delete/erase/scroll, resize/reflow, alternate screen, save/restore, resets, options, markers and disposal/reentrant callbacks.
- Replay identical streams with every relevant single split point, one-unit chunks and deterministic random chunk partitions. Snapshots and outputs must not change with valid chunking except documented callback scheduling boundaries.
- Record deterministic PRNG algorithm, seed, iteration count, operation count and corpus hash. Keep every counterexample with both snapshots and first divergent operation; shrinking may add a smaller case but must not discard the original seed.
- Add addon-specific traces for search results and UTF-16 positions, serialize round trips, Unicode provider activation, program/progress status, protocol replies, clipboard/attach flow and browser-linked rendering effects. Every addon must be represented or explicitly remain unverified.
- Do not replace mismatching expected data, weaken equality, ignore a failing feature or convert a panic/error to a success. Migration counts and test coverage mapping must refer to exact original cases, not approximate Rust test totals.

## Required evidence and review

Store machine-readable raw results and summaries under `results/upstream-baseline/` and differential results under a separate tracked directory. Summaries link commands and source hashes. Document every missing execution and each unported original assertion. Independent reviewers must read runner code, check source integrity, execute at least one original baseline and actual Rust trace, and inspect retained failures. Full terminal equivalence remains NOT PROVEN until original, Rust and applicable native/browser suites have demonstrated the required scope.
