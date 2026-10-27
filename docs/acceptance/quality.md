# Quality and native-platform acceptance

These checks apply to the implemented workspace. They do not establish completion of absent terminal, browser, or addon behavior. Every check must refer to an immutable commit and observed output, not just a workflow definition. Missing implementation remains work, not an external blocker.

## Q1. Formatting and warning-free analysis

- `cargo fmt --all -- --check` exits zero without rewriting another owner's files.
- `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings` passes. Tests, examples and libraries participate, not only the library target.
- Repeat Clippy with `--no-default-features` so feature-disabled branches cannot hide warnings.
- No added blanket lint suppression, ignored test or successful placeholder may turn a failing contract green. Independent review examines changes and exceptional local lint allowances.
- A warning, syntax error or missing module fails the gate and retains stdout/stderr and the exact exit status.

## Q2. Builds, tests and features

- Workspace tests run all targets with all features, default features and no default features, with exact commands and test-summary counts retained in logs.
- Doctests run separately with all features. `--all-targets` does not replace doctests.
- Release builds include all targets and all features. Debug-only success is insufficient.
- Cargo metadata inventories every workspace member, declared MSRV, default feature and named feature. Each named feature runs separately with defaults disabled; default/no-default/all-feature configurations also run.
- Any unsupported combination requires a documented incompatibility and separate supported commands. Do not silently drop a package or optional dependency.
- Locked dependency resolution is mandatory. A missing/outdated Cargo.lock must fail rather than resolve a different dependency set silently.
- Runtime test failures, panic, assertion differences and compilation failures remain failures. Report discovered/executed/passed/failed/ignored counts from actual logs. An empty suite is not subsystem coverage.

## Q3. Native platform evidence

- Observe passing build and runtime tests on Windows, Linux and macOS, each on its own executing OS. Store `rustc -vV`, cargo/Node versions, architecture, OS release, CPU and source hash.
- The compiler host must match the executing OS. An installed cross-target, `cargo check`, or cross-compiled binary does not count as native runtime proof.
- Explicitly identify WSL, containers, virtual machines and hosted runners. Linux under WSL is Linux runtime evidence, not macOS or native Windows evidence.
- Reject conflicting `CARGO_BUILD_TARGET` during native checks. Successful runtime tests must execute on the recorded host OS. Native commands use the shared target directory rather than creating a duplicate target/host tree.
- Run the declared MSRV and stable toolchain on all three platforms in CI. Record the observed stable version rather than treating the word stable as a version.
- Native demonstration and terminal end-to-end proof belongs to task 46 and must be linked separately. Library tests cannot stand in for it.

## Q4. WASM and browser boundary

- Compile all applicable workspace libraries for `wasm32-unknown-unknown` with locked dependencies and all features. Record the target installation and compiler version.
- Keep browser/OS dependencies outside the portable core. Native builds must still compile the same portable engine.
- WASM compile results explicitly say compilation only. Browser runtime tests must invoke the Rust-backed build, in Chromium, Firefox and WebKit, under task 46.
- Missing WASM targets or bindings produce a recorded failure/unrun state, never a browser pass. Feature combinations that cannot represent one target need explicit split commands.

## Q5. Safety and independent review

- Inventory every Rust occurrence of `unsafe`, including private modules, test helpers and generated bindings. A lexical scan is a review aid, not a proof that code is safe.
- For actual unsafe operations, identify the FFI or measured need, ownership, memory lifetime, aliasing, bounds, alignment, thread and ABI obligations. Require adjacent SAFETY comments and public unsafe-function Safety documentation.
- Independent reviewers inspect each unsafe block and its focused tests. Run applicable Miri or sanitizers on a supported toolchain/target; retain tool availability, command and result. Unsupported tools remain explicitly unrun with reasons.
- If no unsafe operations exist in a committed scope, record the exact scope and review result. Do not require a fake sanitizer pass or claim that dependencies contain no unsafe code without auditing them.
- Full architecture, assertion coverage and browser review remains task 50. Passing Clippy is not independent review.

## Q6. Reliable gate automation

- No shell interpolation of paths/arguments. Logs combine stdout/stderr and retain failed exit status, spawn errors, timeout and elapsed duration.
- A missing executable is a normal recorded failure rather than an uncaught callback error.
- Every subprocess has a finite timeout. Timeout terminates the process tree, records termination errors and returns a failed result. It never becomes a retry or successful skip.
- Continue independent gates after a build/test failure to retain useful diagnostics. A fatal metadata/native-provenance failure stops invalid test planning and records why.
- Default output stays under results/ with a timestamped attempt name. Reject repository-root, outside-root and malformed destinations and any existing attempt directory. Failed evidence is never overwritten. Scratch test files stay under tmp/.
- Hash all crate inputs, including TSV/txt fixtures and browser glue, plus scripts/tooling/workflows/build configuration and manifests/lock before and after execution. Concurrent input mutation prevents a one-snapshot pass and requests a rerun on an immutable commit.
- Runner unit tests verify success output, failure output/status, missing executable and timeout. Integration tests must also check CLI rejection and provenance failure paths.

## Q7. CI provenance and artifact retention

- Every native matrix job runs its own runtime tests. Fail-fast is disabled so one OS failure does not erase another OS's result.
- Run quality gates on Windows, Linux and macOS at MSRV and stable. Each job has a finite overall deadline.
- Upload summary JSON and full logs even when a gate fails. Missing artifacts fail the artifact step.
- Record source commit, dirty status, source hash, CI run URL and actual tool versions. Before claiming a remote pass, observe completed job conclusions and retrieve the matching artifacts.
- Remote dispatch permission does not prove credentials exist. Attempt access without logging tokens; record denied/unavailable resources and the exact maintainer action needed.
- A configured workflow, submitted run or green unrelated revision is not proof. Do not claim an unexecuted platform passed.

## Q8. Shared disk and resumed-run safety

- Preserve `.cargo/config.toml` from705104c, with development/test debug=0 and incremental=false. Assertions and release settings remain unchanged.
- Do not create a duplicate native target/host directory or an alternate Cargo target directory merely to run quality checks.
- Observe free space before and after every subprocess and once per second during long commands. Default minimum headroom is1024 MiB; explicit choices cannot go below256 MiB.
- If space is insufficient before a command, retain the failing observation and do not start it. If it crosses the threshold during execution, terminate that process tree and retain resource/termination errors.
- A probe failure is a failed resource check, not permission to proceed. No automated cleanup, source deletion, evidence deletion or other owner's process termination is allowed.
- Unit tests simulate low space without filling the drive and check exact resource failure/state. Preserve the existing45-agent task graph and review history; no duplicate swarm run.
