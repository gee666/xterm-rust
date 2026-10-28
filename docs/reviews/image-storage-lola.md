# Image storage review

Reviewed task 126 from Chloe's commits `48554de` and `7dbc3cc`, copied into the private `swarm/LolaReview` branch as `e9dd5b3` and `b613186`. No source changes were written to Chloe's worktree and no automatic main merge was attempted.

Read the complete unchanged `source_repo/addons/addon-image/src/ImageStorage.ts` and the native storage modules. Checked budget/eviction ordering, shared tile payloads, original/actual image identity, marker callbacks, alternate-buffer cleanup, render ordering and resize quirks. The native host calls the real terminal and buffer service. Drawing tests record host operations and do not prove browser pixels.

Observed on Windows:

- `cargo test --locked -p xterm-addons --test image_storage`: 20 passed, including 567 full source-derived snapshots.
- `node scripts/image-storage-oracle.mjs`: 10 source cases, 567 snapshots verified without regenerating expected data.
- `cargo clippy --locked -p xterm-addons --test image_storage -- -D warnings`: passed.
- `rustfmt --edition 2021 --check` on storage modules and storage test files: passed.

Task 126 was accepted for this native storage component. Tasks 38 and 127 still own real browser assembly and pixel evidence. Overall image equivalence remains NOT PROVEN. Logs are `results/reviews/lola/storage-{tests,oracle,clippy,fmt}.log`.
