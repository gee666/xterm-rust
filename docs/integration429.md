# Accepted application batch429

This batch combines accepted tasks416,417,418,419,421,422 and424 on main926d8355. It makes no new production changes. All42 source/test/script files touched by the selected producer commits match their final accepted trees byte for byte. Task421 deliberately supersedes416's shared serialization runner and case file, retaining both fixes. `results/integration429/producer-identity.json` lists each expected tree and SHA256.

## Included fixes

- 416 preserves an empty first HTML row for numeric NaN or positive-infinite start columns.
- 421 preserves invalid ANSI row-count failure ordering, late getter identity and the browser's own RangeError.
- 417 shares smooth animation curves across numeric and raw JavaScript values, including undefined dimension fallback.
- 418 preserves Base64 TextEncoder construction/encode results and fallback host calls.
- 419 adds deferred fvar constructor validation, refreshed payload reads and GSUB/fvar/META error ordering.
- 422 retains undefined search terms across delayed refresh instead of caching an empty string.
- 424 resizes the currently active Fit terminal after measurement callbacks reactivate the addon.

Each producer's independent review, source oracle, browser reports, retained negative controls and scoped limitations remain in its own documentation/results directory. The batch does not relabel those producer runs as new integrated browser runs. Raw serialization's retained numeric-boundary mismatch remains unresolved. Full task301 live scrolling,342 font variation,346 serialization,347 raw search and323 utility parity are not closed by these repairs.

## Integration verification

Use the unchanged `node scripts/worktree-merge.mjs` from the private worktree with a regular private target, Cargo jobs2, owned O: TEMP/TMP/TMPDIR and the coordinator's sole final-ref grant. Its native changed-package build/all-target tests, affected checks, workspace check and applicable WASM library checks must all pass before its guarded fast-forward. New source and original assertions remain unchanged.

The exact `cargo fmt --all -- --check` command still fails with Windows OS206. All five workspace members pass separate formatting checks. This batch does not waive that failure or declare the full stable quality/platform matrix complete.

The unchanged canonical helper passed and fast-forwarded main to785a7781f369e8cb5f9331387c5fbe2eea89f995. It ran530 native tests in188 summaries, with0 failed, ignored or filtered, for changed crates xterm-addons, xterm-font-addons and xterm-web. It also passed affected all-target, workspace and applicable WASM library checks, then removed its private target. This is a default-feature changed-crate gate, not a full workspace/all-feature/platform/browser proof. `canonical-merge.log` and `canonical-summary.json` retain the exact execution. Receipt-only changes after this source merge will join a later warm producer batch rather than force a cold rebuild.

Overall equivalence remains NOT PROVEN. All new temporary work belongs only to `O:/xterm-rust-swarm-16/tmp/J`; it will be removed after preserving commits and concise results, checking live processes and scanning links without traversal. Original reference files, credentials, other owners' worktrees and the integration checkout's existing untracked `swarm/` directory are outside the cleanup set.
