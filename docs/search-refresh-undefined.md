# Delayed search refresh with no cached term

The full user intent remains `docs/intent.md` and `PORT_REQUIREMENTS.md`. This task continues the same 16-agent swarm and fixes one observable search defect, not full equivalence. Overall equivalence is NOT PROVEN.

Task422 preserves source undefined when a queued write/resize refresh fires after the cache has been cleared. Current main substitutes an empty UTF16 term, which becomes a cached empty string and changes later flag getter reads. Source captures the current cached term before spreading options, so copy callbacks may reenter without replacing that captured absence.

Own files are `crates/xterm-addons/src/search/addon.rs`, `term.rs`, and new directed native/browser fixtures. Preserve accepted412 and delegated integration. No source_repo assertions, AGENTS.md, dependencies, credentials or main refs will be changed. Use private `O:/xterm-rust-swarm-16/tmp/Q422` for source, target and scratch. All owned generated files and processes must be cleaned after durable source/evidence retention. Heavy runs need the coordinator's explicit grant, Cargo -j2, sequential browser matrices and bounded commands.

Verification planned: unchanged-source shared browser assertions, actual Rust WASM three-engine assertions, native virtual-clock regressions and warning-denied scoped Clippy. Retain the old-producer negative control, source hashes and exact executed counts. Full workspace/platform proof and all other347/322 gaps remain open.
