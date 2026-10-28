# Serialize task 131 continuation

Worktree `C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Vikramaditya`, branch `swarm/Vikramaditya`. All code is committed; task 131 is unfinished. Do not delete the worktree, its source_repo is an older reference junction.

Implemented Rust WASM backend in `crates/xterm-web/src/addons/serialize*.rs`, minimal JS lifetime facade `browser/serialize.mjs`, numeric packed-cell HTML support in core and an unchanged-upstream Playwright runner. Native tasks37/88 are accepted. The branch privately stacks Xavier's browser checkpoint, latest merge7dae405 including d307a1f, and Willie9f33ab3. Do not merge other owners' WIP to main without their integration readiness.

Commands/results:
- `cargo test -p xterm-core --all-targets --all-features`: passed after Willie9f33ab3.
- Numeric HTML tests, HTML edge tests and 3350 source-output comparisons passed; scoped core Clippy passed.
- Latest `cargo check -p xterm-web --target wasm32-unknown-unknown --lib`: PASSED after d307a1f. Earlier six host errors are fixed.
- `node scripts/test-serialize-browser.mjs --list`: 120 unchanged upstream cases discovered across Chromium, Firefox and WebKit. Only setup adds a WASM-ready wait. Actual browser runtime NOT RUN yet.
- Pushed earlier branch CI https://github.com/gee666/xterm-rust/actions/runs/40658330487 failed native quality/WASM, Miri passed. Push current commits and run the existing rust.yml workflow again; do not call Linux/macOS blocked.

Next actions:
1. Integrate Charulata894d70f real BufferLine JSON needed by untouched upstream round-trip assertions. Her current branch may have dependency updates. Do not replace the assertions with pointer comparisons or fake empty data.
2. Check `browser/xterm.mjs` exports `SerializeAddon` from `./serialize.mjs`; add if absent. Build xterm-web wasm32 and run existing wasm-bindgen CLI into `browser/pkg`. No source JS engine in the page.
3. Run `node scripts/test-serialize-browser.mjs --project chromium` first, then all three browsers; fix actual failures. Runner owns/shuts down its server and child process tree. Source dependencies are available through source_repo reference.
4. Add binding tests for options/getter ordering, negative/fractional/NaN coordinates, markers, UTF-16, exceptions, selection/theme, console diagnostics and JS retention. `docs/serialize-browser-acceptance.md` defines SB01-SB08.
5. Known unproven limits: non-finite HTML traversals use an explicit safety RangeError rather than source nontermination; malformed nonnumeric marker.line coercion is incomplete; warnings are emitted after rendering, which can differ if console.warn itself mutates state. Do not claim all SB cases pass.
6. Full core tests write buffer-terminal result files; preserve failures if any, otherwise restore generated tracked outputs before merging. Serialize report generation already goes to tmp/serialize, not tracked results.
7. Use private worktree builds and the worktree merge script. No shared queue. Keep evidence short. When task is finished use swarm_review, fix findings and resume review; merge only code that builds/tests.

Shared main files were restored when work moved here. No edits to AGENTS, no credentials copied, no background services remain. Overall equivalence NOT PROVEN.
