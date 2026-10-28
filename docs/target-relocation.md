# One canonical cache relocation on the Windows host

Chloe, the workspace owner, approved one coordinated relocation of generated Cargo artifacts after repeated native-development cleanup failed to keep enough disk space. The P drive reached about 1 GiB free while C had about 100 GiB. This is a persistent cache, not temporary scratch, a second active target directory, or a compiler/profile override.

The application and Cargo configuration keep using the same repository `target/` path. That path becomes a Windows directory junction to a new user-owned local cache. No source, tests, results, tmp directory, source_repo, credentials, auth files or private swarm runtime data moves. Native and WASM/release artifacts all retain their bytes and timestamps.

## Review and execution

Code is in `scripts/target-relocation.mjs`, `scripts/relocate-target-windows.mjs`, and the shared conservative process inventory `scripts/quality-idle.mjs`. Ten tiny filesystem unit tests in `scripts/target-relocation.test.mjs` cover read-only planning, promotion, refusal of busy/existing destinations, corrupt copy refusal, failed junction/probe rollback by same-volume rename, partial backup-cleanup failure, control-file mutation and preserving unknown canonical entries for manual recovery. They execute no compiler and use only repository-root tmp fixtures.

The Windows adapter defaults to a new project-specific cache under the current user's `.xterm-rust-target` directory. It rejects a same-drive destination, network paths, redirected destination ancestors and existing destinations. A different destination can be supplied explicitly.

```sh
node --test scripts/target-relocation.test.mjs
node scripts/relocate-target-windows.mjs --approved-by Chloe --reviewed-by Aaliyah
# Only after the dry-run manifests and the implementation are independently reviewed:
node scripts/relocate-target-windows.mjs --approved-by Chloe --reviewed-by Aaliyah --apply
```

The outer command timeout must allow copy, validation and rollback, at least 600 seconds. Each native copy has a separate 120-second deadline and uses robocopy `/COPY:DAT /DCOPY:DAT /R:0 /W:0`. The OS copy preserves Windows FILETIME precision rather than rounding through JavaScript Date values. There are no automatic retry loops.

## Transaction invariants

1. Check the real exact target directory, Cargo cache marker, control hashes for Cargo manifests/lock, attributes, existing AGENTS and profiles, and a fresh conservative process inventory. AGENTS files are only read; no test creates one.
2. Applying creates the global build-slot maintenance flag and takes both native and WASM lane leases. New queued commands fail closed. Existing live builds cause refusal; the script never kills unknown processes.
3. Hash every regular file and inventory exact paths, modes and nanosecond modification times, including Cargo `invoked.timestamp` files. Reject nested links and nonregular entries.
4. Copy to a new empty persistent destination. Verify every copied byte and metadata value, then recheck that the original cache, control files and process inventory did not change.
5. Save the verified manifest and phase journal. Atomically rename the old cache to a unique rollback directory under repository tmp on the same P volume. Create the canonical junction and verify it while the original bytes still exist untouched in the rollback directory.
6. Run actual Cargo metadata before and after cutover. Validate its workspace and resolved shared target, unchanged control hashes and both cache manifests. Only after those checks delete the exact owned rollback directory. A junction/probe failure rolls back with a same-volume rename rather than recopying. Never overwrite an unknown canonical entry; retain the verified cache, rollback directory and pause for manual recovery if safe rollback is impossible.
7. Record exact tool hashes, copy exit codes, profile hash, source manifests, process observations and before/after filesystem free space under a unique `results/quality/target-relocation-*` attempt. A failed or rolled-back attempt never reports successful promotion.

An abrupt OS/process death can interrupt any cross-volume operation. The phase journal, verified destination and any retained same-volume rollback directory remain the recovery evidence. Do not clear stale maintenance leases or retry by deleting an arbitrary path. Inspect the saved manifest, verify the destination and canonical path, confirm process idleness, and resume or restore the known generated cache deliberately.

After promotion, resource checks must continue monitoring both the repository drive and the actual target filesystem. Do not run the older real-directory cleanup helper against the junction or recreate a second target cache. Acceptance adapters should pin executables/WASM as regular files inside their output or content-addressed artifact directory before hashing/executing. This preserves the strict no-source-symlink rule without widening artifact access to arbitrary external paths.

## Observed promotion

The authorized one-time apply completed with exit 0. The canonical P target now resolves to `C:\\Users\\user\\.xterm-rust-target\\7144d30a1fc3`. All 2608 files, totaling 1558942761 logical bytes, matched six full byte/path/mode/nanosecond-time verification passes with manifest SHA256 `b4bb1c2ac87634710ca501e334428ee0e63a771b979e4e1d6a8db1aac0713773`. Cargo metadata kept the logical P target and resolved it to the new C backing directory after promotion. Control hashes remained equal. The exact rollback backup was removed only after all checks passed.

The receipt is `results/quality/target-relocation-1793170946352-20096-675fdf83/receipt.json`; `space-and-review.json` records repository free space rising from 941076480 to 2480951296 bytes and target filesystem free space of 106935476224 bytes. Actual copy logs, tool hashes, controls and per-stage verification hashes are retained there. No source, credentials, existing tmp data, evidence or profile configuration moved or changed.

A successful cache relocation is operational evidence only. It does not establish xterm.js feature parity or a passing Rust/browser test matrix.
