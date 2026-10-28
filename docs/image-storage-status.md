# Image storage

Native storage, real terminal/buffer effects, resource ownership, callback reentry, render commands and resize behavior are implemented.

Commands in the Chloe worktree:

- `cargo test -p xterm-addons --test image_storage`: 20 passed, including 567 snapshots from unchanged source.
- `cargo clippy -p xterm-addons --test image_storage -- -D warnings`: passed.
- `node scripts/image-storage-oracle.mjs`: 10 cases and 567 source snapshots verified without rewriting expected data.
- Scoped rustfmt checks: passed. Whole-workspace `cargo fmt` exceeds Windows command-line length in this path; per-package checks also exposed unrelated existing formatting.

The test drawing host records platform calls; it does not prove browser pixels. Tasks38 and127 retain actual protocol/browser assembly and image renderer integration. Overall image equivalence is not proven.
