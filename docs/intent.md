# User intent

Port the supplied xterm.js 6.0.0 implementation in `source_repo` to Rust, with full observable equivalence rather than a subset or a wrapper around the JavaScript terminal engine. The controlling requirements are `PORT_REQUIREMENTS.md`, the swarm brief, and the unchanged `AGENTS.md`.

The requested scope includes common terminal state and parsing, headless and public APIs, browser/WASM rendering and interaction, every addon, options, events, disposal, JavaScript UTF-16 semantics, streaming input, Unicode, and platform integration. Keep the platform-independent engine separate from browser and native adapters. Coordinate shared interfaces and file ownership across the requested 45-agent team and obtain independent review.

Preserve `source_repo` as the behavioral reference and preserve unrelated user files. Inventory and hash the reference, retain attribution and licenses, and map every API and source test suite to Rust implementation and evidence. Migrate upstream assertions without weakening them. Run original and Rust suites, deterministic and seeded randomized differential tests, integration tests, and actual native and Rust-backed browser end-to-end tests. Record missing, failed, skipped, and unrun checks honestly. Native Windows, Linux, and macOS runtime evidence is required; cross-compilation or a CI configuration alone does not establish it.

Deliver a reproducible, warning-free Rust workspace, maintained dependency resolution, documented MSRV/features, user-run native and browser demos, and clean-checkout commands. Document compatibility in `docs/compatibility.md`, exact verification in `docs/verification.md`, and reproducible performance, artifact-size, and LoC comparisons in `docs/comparison.md`. Preserve raw non-secret machine-readable evidence. Compare equivalent validated workloads using release builds, warmup and at least ten timed repetitions, with median and p95. Distinguish browser distributions from native executables and physical lines from comment-stripped code lines.

Completion requires demonstrated acceptance criteria, accepted independent reviews, working deliverables, and committed source. A small passing suite cannot prove full equivalence. Mark overall equivalence NOT PROVEN while any required coverage or runtime evidence is missing. If an external resource prevents progress, identify the exact missing resource, attempted resolutions, retained work, and concrete unblock steps rather than claiming success.

Do not change `AGENTS.md`, user-owned startup files, source assertions, or credentials. Keep temporary files under repository-root `tmp/`; every shell command needs a timeout.

## Current working method

The user now requires isolated per-agent git worktrees and branches, independent small Cargo targets, and parallel builds instead of the global build-slot queue. Rebase on main, build and test changed crates, then fast-forward main; branch owners resolve conflicts. Share downloaded dependencies and remove each worktree target after merge. Use GitHub Actions for observed Windows, Linux and macOS results. Keep main compiling, finish working code and real upstream tests, and keep reviews and evidence short. Infrastructure is justified only when it removes a real bottleneck.
