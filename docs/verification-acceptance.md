# Verification acceptance checks

These checks supplement the umbrella Evidence and Quality criteria. They govern task #50 and the final report, not implementation correctness by themselves. A finite test corpus does not prove equivalence. A recorded missing check remains a gap until actual evidence replaces it.

## V01: reference integrity and provenance

- Retain the pre-launch manifest as `docs/baseline/source-manifest.json`, with path, exact byte length and SHA-256 for each of its 776 files.
- Identify the source as the user-supplied xterm.js 6.0.0 tree. Do not invent a public commit or download URL.
- Recompute all listed hashes without changing source bytes, tests or assertions. Report missing files separately from changed files, with expected and actual values.
- Invalid, duplicate, absolute or parent-traversal manifest entries must fail. Missing files and byte/hash mismatches must produce nonzero CLI status. I/O failures other than missing files must propagate.
- Do not present the initial manifest as proof that subsequently added files are unchanged. The compatibility inventory must separately account for the current source tree and declared generated-output exclusions.
- Index every retained result with a repository-relative path, byte length and SHA-256. Exclude the generated index itself. Reject symlink and nonregular evidence entries rather than silently reading unrelated paths.
- Test exact CRLF and non-BMP byte hashing, nested ordering, altered/missing entries, invalid/duplicate paths and self-exclusion. An evidence file's existence must never imply a passing check.

## V02: original and Rust suite count accounting

- For each discovered upstream unit, browser, integration, benchmark and support file, record the suite kind and implementation/test mapping or an explicit unmapped gap.
- Distinguish static test definitions from executed cases and assertions. Loops, generated cases, shared registrations and support files must not be counted as runtime passes.
- Report discovered, mapped, executed, passed, failed, skipped and unrun counts where measured. Use unknown or not assessed when no execution report exists, never zero by assumption.
- Original baselines execute the unchanged upstream assertions. Explain import/type transformations, assertion-library adaptations, hooks, asynchronous awaiting and any unsupported runner behavior.
- Rust tests must call Rust implementation code. Calling the original JS engine establishes a baseline only, not port coverage.
- Keep per-suite logs/results. Reconcile count differences rather than summing incompatible case/assertion/operation totals into a single success percentage.

## V03: command receipts and snapshot identity

- Every claimed run identifies exact argv, working directory, exit code or signal, timeout/failure, tool versions, platform/architecture and its raw log path.
- Record applicable Cargo profile, features, target, lockfile policy and environment flags. A local Windows run is not evidence for Linux or macOS.
- Link each successful result to the source/fixture hashes or immutable revision it tested. Detect a changing shared tree and require an immutable-snapshot rerun before claiming final workspace success.
- Retain failed attempts and subsequent successful reruns separately. Never overwrite historical failures with a green summary.
- Reproduction commands must run from the repository root and explain Node/reference dependencies. Temporary outputs stay under root `tmp/`.
- Index generation is a snapshot of retained files, not test execution. Regenerate it after all report/log edits; avoid redirecting its stdout into an indexed file during generation.

## V04: differential scope and counterexamples

- Each differential corpus identifies the unchanged JS oracle, Rust entry point, input domain, seed, operation/trace count, chunk boundaries and exact compared outputs.
- Full terminal claims require full cells, codepoints, widths, attributes, buffers, cursor, modes, scrollback, events, replies and API results, not just printable text.
- Distinguish exhaustive finite channel/codepoint checks from seeded samples. Report what the random generator cannot generate.
- Cover malformed input, split UTF-8/UTF-16, lone surrogates, resizing/reflow, alternate screen, asynchronous handlers and addon operations through their actual owners' suites.
- Retain the first failing operation and preceding trace, seed, expected/actual result and rerun command. A named fixed regression must continue to pass alongside the original assertions.
- Do not mark a differential mismatch as an external blocker when the Rust implementation can be fixed.

## V05: independent review and known findings

- A reviewer other than the implementation author reads actual upstream and Rust source, migrated assertions and oracle adapters, and executes applicable checks.
- Review callback order, identity/disposal/reentrancy, numeric conversions, UTF-16 indexing, limits, asynchronous cleanup, error behavior, attribution, dependencies and any unsafe boundary.
- Record reviewer identity, inspected paths, commands/results and precise findings. Task acceptance alone is not executable evidence.
- Keep unresolved findings visible with their affected contracts and responsible task. Acceptance of a small module does not accept terminal/browser integration.
- Review fixes against retained counterexamples. In particular retain nested-clear Set iteration, malformed CSS parseInt behavior, and active parser registration mutation regressions discovered in this port.
- Inspect shared algorithms once as shared behavior, and verify protocol-specific adapters separately. Shared code must not conceal behavior differences.

## V06: runtime, safety and honest completion

- Report fmt, warning-denied Clippy, workspace tests, doctests, release builds, default/no-default/all and named feature combinations separately.
- Native Windows, Linux and macOS each need observed runtime evidence and execution identity. CI configuration and cross-compilation are not native runtime passes.
- WASM builds need target/binding evidence. Chromium, Firefox and WebKit e2e must execute the Rust-backed demo and assert rendering, input, selection, clipboard, sizing, links, accessibility, disposal and addons.
- Inventory unsafe code. Any required FFI boundary needs adjacent safety invariants, independent review and applicable Miri/sanitizer evidence. Record unavailable checks explicitly. A lexical scan is not proof of safety.
- Report missing demos, incomplete APIs/addons and absent matched benchmark/size/LoC measurements explicitly. Never manufacture zero timings or speedups.
- Overall equivalence remains `NOT PROVEN` while any implementation, mapping, failing test, unrun required platform/browser check or unresolved review finding remains.
- A final blocker records the missing external resource or decision, attempted resolutions and concrete unblock step. Internal dependency waiting remains unfinished work, not a blocker.
