# Workspace integration progress

Task 60 remains in progress. Root/core manifests, lockfile and core exports are owned by Chloe. The browser manifest and root are owned by Xavier; addon root by Lane. Athena has delegated approval for the new DOM-free support member and its path dependency after the resource-cutover control freeze ended.

Delivered checkpoints include e624c6a, 4bb671d and 651a493. They establish the Rust 2021/MSRV 1.85 workspace, portable core separation, browser/addon members, dependency licensing, module exports and common collection integration. `results/workspace/run2-wasm-check.log` records an actual earlier WASM library compile, not current full browser proof. The current source tree has changed since that checkpoint.

The byte-preservation check found a real Windows checkout regression in source-loc.json. Existing raw fixture/results rules passed; the baseline file expanded from 2002 to 2097 bytes under autocrlf. With Aaliyah's approval, `.gitattributes` now keeps source LF and all baseline/raw results/fixtures byte-exact. Before/after receipts are `results/workspace/eol-checkout-before.json` and `eol-checkout-after.json`. No baseline content, AGENTS, source_repo or unrelated user file was edited or broadly renormalized.

Disk recovery preserved code, tests and evidence. The first reviewed cleanup removed only development artifacts. The later reviewed transaction moved one persistent generated target cache to C and kept the logical repository target junction, profiles, hashes and modification times. No credentials/source/results were moved. Kassa owns the resource receipt and self-service controlled build queues. This is resource coordination, not terminal/platform equivalence proof.

Remaining work:

- Resolve and checkpoint the support-member/path-dependency/Cargo.lock integration as one coherent graph.
- Commit all completed owned root exports and ensure every exported implementation is tracked. Do not commit other owners' incomplete files or user/private runtime files.
- Run fresh native/core/addon and applicable WASM checks after the ongoing key/marker/parser migrations are coherent. Preserve actual failures instead of relabeling historical green snapshots.
- Obtain independent review of the final coherent root/feature/byte-policy checkpoint. Native Linux/macOS and all-feature/release matrices remain the observed-CI duties of task 47, not assumed from workflow configuration.

Full equivalence is NOT PROVEN. Task 60 will request review only with current commands and evidence; other implementation gaps stay assigned to their source owners rather than hidden by successful placeholders.
