---
name: search-check
description: Use when changing the portable search engine, wrapped line translation, cache or search-state contracts, or checking their source assertion and differential evidence.
---

# Search verification

Run `node scripts/check-search.mjs` from the repository root with Node 24.15 or newer, Rust 1.85 and the original dependency tree installed under `source_repo/node_modules`. Add the local Cargo bin directory to PATH if the shell does not expose it.

Use your isolated agent worktree and private Cargo target. Read the worktree skill, then run `node scripts/check-search.mjs` inside that worktree. The checker runs unchanged original SearchEngine and SearchLineCache tests over the original Terminal, verifies retained fixtures, runs the currently declared 34 Rust tests and warning-denying scoped Clippy and checks owned Rust formatting. Commands have bounded timeouts. Raw logs, full core dependency snapshots, actual discovered/executed/pass/fail/unrun counts and tool versions go to `results/search/` and per-attempt archives. Changed input snapshots invalidate a run; a compile/resource failure is not a test pass.

`node scripts/search-oracle.mjs` checks the retained corpus rather than overwriting expected results. Only use `node scripts/search-oracle.mjs --generate` when deliberately updating source fixtures, then inspect every fixture and report change. Seed `0x5ea2c001` drives 2048 actual-cell randomized engine cases plus 1024 state cases. The full retained corpus has 2200 engine calls and 6116 wrapped translations including calls observed while the 86 unchanged original assertions run. Original code is bundled into `tmp/search-oracle`; source_repo files are not edited.

Use `search::with_buffer` for real terminal rings, not bounded row snapshots. It preserves stale and signed aliases. RangeError recursion outcomes and distinct nontermination safety guards are checked in `tests/search_ring.rs`. Read `docs/search.md`, `docs/search-addon.md` and `docs/acceptance-search.md`. The cache has explicit host clock and invalidation inputs. Its tests are not proof that terminal event subscriptions or browser timers are connected. SearchAddon, result tracking, decorations and Rust-backed three-browser behavior remain task #59. Native Windows execution is not Linux/macOS runtime evidence. Full equivalence is NOT PROVEN.
