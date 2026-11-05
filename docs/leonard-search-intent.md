# Search highlight getter timing

Continue the same swarm #2 Rust port under `PORT_REQUIREMENTS.md`, the current brief and `docs/intent.md`. Full equivalence remains NOT PROVEN.

Task458 follows accepted445. Own only the live highlight decoration-read phase in `SearchAddon`, its browser value adapter and focused native/shared JavaScript regressions. The source finishes matching and updates result tracking before reading `searchOptions.decorations` for eligibility, then reads it again as the call argument if eligible. Either getter may reenter, throw, or return a different value. A falsy second argument still enters decoration creation, including nullish color errors only when results need colors.

Preserve source assertions, current main, prerequisite commits and other owners' files. Work privately in `O:/xterm-rust-swarm-16/tmp/L458`, with a private target and scratch. Obtain grants before heavy work, Cargo `-j2`, no main ref actions. Retain exact failures and scoped receipts, independently review, then inspect links/processes and remove the entire owned temporary worktree. Integrators must take only new458 commits after the accepted445 chain. Full347 raw matching getters/terms and full322 search/serialization closure remain open.
