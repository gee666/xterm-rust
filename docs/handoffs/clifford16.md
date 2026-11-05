# Clifford16 font integration handoff

Task293 passed independent review. Criterion461 covers only recovered browser font addons, valid public joiners and unchanged web-font-suite behavior. Full font equivalence is NOT PROVEN.

## Source and results

The producer branch is `swarm/Clifford16`. Four reviewed Fergus commits were recovered on current main rather than applying obsolete shared working bytes. The font crate remains independent of the browser terminal and uses existing workspace dependencies. `xterm-web` reexports its constructors and modules.

New `terminal/joiner_api.rs` registers with the renderer's actual `BrowserServices.joiners`. Public registration/deregistration error before open, refresh behavior and callback-driven registry changes have real DOM regression tests. The actual WASM distribution exports LigaturesAddon, WebFontsAddon, enableLigatures and loadFonts.

`docs/font-terminal-integration.md` explains contracts, commands and boundaries. `results/font-terminal/` retains source hashes, original-suite output, three-browser summaries, focused Clippy output and rejected initial new-test expectations. The original reference test files match the preserved P: checkout byte-for-byte.

Observed Windows results, rerun independently by task review:

- Actual-terminal joiner/fallback/style/disposal/mutation suite: 3 tests per Chromium, Firefox and WebKit.
- Recovered ligature component suite: 4 tests per engine.
- Unchanged original WebFontsAddon Playwright suite: 15 discovered, executed and passed, zero failed/skipped/unrun.
- Font-crate formatting and focused font/all-target plus terminal/new-test WASM Clippy with warnings denied passed.

Native Linux/macOS and final workspace matrices are not claimed. Full workspace formatting on the longer Clifford16 path hit Windows OS206; the quality owner has the exact receipt and is investigating a shorter temporary path.

## Remaining font work

Task307 explicitly retains raw or malformed joiner-return coercion/identity and exception behavior, unused font-metadata constructor rejection, unchanged legacy 8-case Node LigaturesAddon tests and loaded-font shaping through an actual terminal. The current adapter accepts arrays of finite nonnegative integer UTF16 offsets. Generic font fallback success does not prove real font shaping.

Historical component evidence and old task numbers in recovered documents remain scoped history. The source empty-pattern hang remains an explicit Rust safety divergence, not equivalence.

## Temporary resources

The only owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/Clifford16`. The runner owns and closes browser/server processes. The temporary `source_repo/node_modules` junction points to the existing P: dependency install. Before whole-worktree cleanup, unlink it, reject remaining reparse points, inspect tracked/untracked changes, preserve required receipts and confirm HEAD is integrated. Do not remove the original reference or any old worktree.

Final canonical gate and cleanup receipts must be added to `results/font-terminal/` and the coordination post before retiring. Malcolm owns integration/resource task309; do not independently write main while another producer has final-ref exclusivity.
