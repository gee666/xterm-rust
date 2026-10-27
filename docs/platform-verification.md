# Native quality and CI evidence

Overall xterm.js equivalence is NOT PROVEN. Quality scripts check the implemented workspace, not missing terminal/browser/addon contracts. `docs/acceptance/quality.md` defines the individual gates and failure cases.

## Reproduce

Use Node24 and a Rust toolchain with rustfmt and Clippy, plus Git. The workspace declares Rust1.85 as its MSRV. CI tests1.85.1 and the observed stable release.

```sh
node --test scripts/quality-process.test.mjs scripts/quality-policy.test.mjs scripts/quality-snapshot.test.mjs
node scripts/quality-gates.mjs --out results/quality/NEW-ATTEMPT
rustup target add wasm32-unknown-unknown
node scripts/quality-gates.mjs --wasm-check --out results/quality/NEW-WASM-ATTEMPT
```

Every output directory must be new. The default includes a timestamp. Failed logs are retained, not overwritten by a later attempt. Each command records its arguments, exit status, timeout, duration, log and Cargo per-suite result summaries. Compiler/OS/CPU provenance and input hashes accompany the run. Source input changes prevent a one-snapshot gate pass.

Honor `.cargo/config.toml` from705104c. Development/test debug symbols and incremental caches are disabled without disabling assertions. Release settings remain unchanged. Native commands share the normal Cargo target directory. Free space is checked before and after every command and sampled each second during execution. The default minimum reserve is1024 MiB. `--min-free-mib` accepts explicit choices from256 MiB to1 TiB. Crossing the threshold stops this runner's process tree and records a resource failure. It does not delete artifacts, stop other owners' builds or retry indefinitely.

Git Bash on the launch machine does not initially expose Cargo or Git. Its local setup is:

```sh
export PATH='/c/Users/user/.cargo/bin:/c/Program Files/Git/cmd:'"$PATH"
```

These executable paths are local setup, not portable application requirements.

## Observed local evidence

- `results/quality/windows-initial/summary.json` retains the first Windows attempt using rustc/cargo1.85.1, Node24.15.0 and Windows10.0.26300 on an AMD Ryzen97900X. Formatting failed, and six lint/build/test gates failed while owners were changing the DCS result API and a BufferLine test initializer. Doctests passed. This attempt is not a workspace pass.
- That initial runner used explicit native-host targeting before the run2 disk policy. It is retained as historical failure evidence. The current runner no longer creates a duplicate native-host output tree.
- `results/quality/runner-tests-run2.log` records13 passing Node tests for output/status retention, missing executables, timeouts, descendant termination, low-disk termination, CLI boundaries, native-host rejection, exact test summaries, fixture/config/workflow hashing and disk observations.
- Initial WSL probing observed Debian under Linux6.18.54.1-microsoft-standard-WSL2. `rustc`, `cargo` and `node` were absent from that environment. No Linux Rust runtime pass was obtained from that probe. No Windows authentication files were copied or linked.
- An installed macOS cross-target is not macOS runtime evidence. No macOS runtime result is claimed here without retrieved CI logs.
- `node scripts/github-quality.mjs runs` initially returned an empty list for this workflow. This proves neither a CI pass nor a completed platform run.

## Remote execution and receipts

The user authorized running GitHub Actions. `.github/workflows/rust.yml` defines native Windows/Linux/macOS jobs on MSRV and stable, plus a separate WASM compilation job. Each native job runs the runner tests and all workspace gates. The workflow preserves artifacts even on failure and does not fail-fast across platforms.

Push a complete committed integration snapshot to an authorized branch. Do not commit private swarm runtime data or unrelated user files. Observe and retain remote results with:

```sh
node scripts/github-quality.mjs runs
node scripts/github-quality.mjs dispatch BRANCH
node scripts/github-quality.mjs status RUN_ID
node scripts/github-quality.mjs artifacts RUN_ID
```

The helper uses private environment tokens or Git's existing credential helper. Tokens remain in memory and are not printed or saved in evidence. Receipts identify the source SHA, run URL, actual completed job conclusions and runner labels. Matching artifact ZIPs are retained under `results/quality/ci-RUN_ID/`.

A workflow definition, dispatch receipt or pending job is not execution proof. A native test pass on one revision does not establish another revision. WASM compilation is explicitly not browser runtime proof. Chromium/Firefox/WebKit checks, native demo end-to-end behavior, independent full-port review, unsafe-operation review and applicable Miri/sanitizer execution remain separately accountable under tasks46/50 and the implementation owners.
