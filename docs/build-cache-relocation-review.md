# Generated Cargo cache relocation review

Scope is one coordinated persistent build-cache relocation, not source cleanup or an additional build tree. Chloe approved the recovery approach after repeated disk exhaustion. Kassa owns implementation and execution. Aaliyah's checklist below is a plan review, not a claim that a move has executed or that an unseen script is safe.

## Preconditions

- Put both native and WASM build lanes into maintenance. Existing owners must finish, abort through their own handles, or explicitly release work. No new build begins during the copy/cutover. Do not steal a live or unexplained stale lease.
- Capture a fresh process inventory. Refuse active Cargo/rustc/rustdoc/Clippy/link/bindgen processes and running executables from the selected target tree. Account for any owned server still reading target artifacts. Do not kill an unknown PID.
- Select only repository-root `target`, confirm its actual file type and Cargo cache marker, and reject unexpected links. Never copy the repository, source_repo, results, tmp, .pi, Cargo/Rustup home or authentication files.
- The destination is a new, empty, project-specific persistent cache directory on a filesystem with enough room for the full apparent byte sum plus reserve. It must not overwrite a pre-existing user directory. Record ownership outside the byte-for-byte target copy.
- Freeze and hash build-control files for the transaction, including Cargo manifests/lockfile and `.cargo/config.toml`. No profile, assertion, release, environment, credential or login changes are part of this move.

## Copy and proof

- Retain an immutable receipt under a new results directory with exact source/destination paths, approval, commands, free bytes and process/lease observations. A receipt is recovery evidence, not test equivalence.
- Copy generated contents only, without following an unexpected directory link into unrelated data. Preserve file bytes and modification times used by Cargo. Size alone is not verification.
- Compare complete relative-path sets, lengths, hashes and modification times for every copied file. Recheck the source after copy so an unexpected writer cannot silently invalidate the comparison. Treat hardlink copies as separate paths in the upper-bound space calculation.
- Any failed copy, path/hash/time mismatch, unexpected writer or changed control file aborts before destructive cleanup. Retain the original cache and failure details. No automatic retry or unrelated cleanup disguises the failure.

## Cutover and rollback

- Keep the logical repository `target` path. An approved Windows junction may point it at the single persistent physical cache, avoiding committed machine-specific paths or CARGO_TARGET_DIR overrides.
- Keep the old generated cache until the destination is verified and the junction is validated. A temporary rollback directory belongs under repository-root tmp. This temporary duplication is part of the one coordinated transaction, not a second active build cache.
- If cutover fails, restore the original logical target without overwriting an unexplained path created by another process. Preserve both owned copies for inspection when rollback is uncertain. Never delete a source or evidence directory to make room.
- Validate real/logical target resolution and Cargo metadata without compiling. Quality/native checkers must recognize the approved logical and physical aliases as the same shared target, and monitor both the repository and target filesystems.
- Only after verified cutover, unchanged control files and a fresh idle check may the exact old generated backup be removed. Record actual reclaimed and retained bytes. No broad delete, force overwrite of user data, or automatic stale-lock removal is authorized.
- Release maintenance only after success or a safe rollback. An interrupted or ambiguous operation remains visibly failed/incomplete until a coordinator inspects it.

## Integration checks

- Root source/evidence inventories must continue to reject source links. Only the explicitly reserved generated port-root target path may be excluded before a link check. Do not whitelist arbitrary source links or follow the cache during LoC/source scans.
- Update cache ownership/recovery documentation and build-slot probes. Future clean operations must not follow a newly linked target without validating its registered ownership and exact destination.
- Native Linux/macOS and browser runtime claims remain unproved by a cache move. Retained binaries do not establish a new test pass. Rebuild and rerun the required assertions under the normal resource guards.

Until the concrete implementation and its dry-run receipt are inspected, this document supplies conditions for review only. It does not approve an arbitrary path or command.
