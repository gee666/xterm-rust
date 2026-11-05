# Current stable warning-denied repair

Task362 repairs actual diagnostics, not the complete port. Overall equivalence remains NOT PROVEN. The original failing native CI checkpoint remains in `docs/quality-vesna16.md` and `results/quality/ci-42012763111/`. Final integrated matrix task304 is still required after producer changes settle.

## Failure and change

The original local Rust1.97.1 matrix fails both Clippy configurations on two loop-head forms. The recorded current CI stable version is Rust1.99.1, which additionally reports test-helper warnings. A matching1.99.1 compiler was installed with only minimal/clippy/rustfmt components in owned temporary `O:/xterm-rust-swarm-16/tmp/V/tmp/rustup199`. No shared Rustup installation or credentials were changed. The private installation will be removed with the owned worktree.

`baseline199.log` retains a real failing `cargo clippy --workspace --locked --all-targets --all-features --keep-going -- -D warnings` against the unchanged5ca9f593 source. Keeping independent targets going exposes diagnostics hidden behind CI's first failure. The repair is source commit `da60f31`.

Two production changes replace `loop` plus immediate `let Some(..) else { break }` with `while let` in ligature reverse walking and wrapped web-link row expansion. Each loop evaluates exactly the same expression once at the same point. Checked coordinate increments and subsequent lookup/mutation order are unchanged.

Thirteen test files or shared test helpers replace an empty `Wake` implementation and its allocated `Arc` with standard-library `Waker::noop()`. These fixtures explicitly poll controlled futures; they do not implement an executor. Polls, pending/ready checks, panic paths and callback assertions remain. The real native write pump's nontrivial FIFO waker is untouched. `Waker::noop()` is available at the declared Rust1.85 MSRV.

Other test-only changes preserve expected values exactly: `u32::MAX & !mask` becomes `!mask`, and the two-element byte array becomes the identical dereferenced byte string. The terminal-mode reply helper still visits and validates every data event before returning the last reply. It deliberately does not use Clippy's direct `next_back()` suggestion, which would skip earlier UTF16 validation. No original reference source, expected fixture, test function or assertion was deleted or disabled. No lint allowance, dependency, unsafe code or JS engine delegation was added.

## Verification

All seven bounded checks pass against identical before/after input hash `ff2090a2f82b9fc01ea9b523f4d9b3df8afabf1e0f8ff0a7a48dec0d940aa8a6`. Raw commands, toolchain selection, exit status, timeout state and test summaries are in `results/quality/stable-repair362/summary.json`; each command has its original log. Every check uses Cargo `-j2`, the one regular private target, unchanged profiles, and owned O: temporary paths.

- Rust1.99.1 workspace all-target/all-feature Clippy, warnings denied, with `--keep-going`.
- Rust1.99.1 workspace all-target/no-default-feature Clippy, warnings denied, with `--keep-going`.
- Rust1.99.1 full workspace all-target/all-feature tests, 2,012 passed functions in385 summaries, zero failed or ignored.
- Rust1.85.1 exact `cargo fmt --all -- --check`.
- Rust1.85.1 workspace all-target/all-feature Clippy, warnings denied.
- Rust1.85.1 full workspace all-target/all-feature tests, 2,012 passed functions in385 summaries, zero failed or ignored.
- Rust1.85.1 all-feature WASM library Clippy for core/addons, warnings denied. This is compilation only.

The two full test commands include unchanged ligature graph/source fixtures, wrapped/sparse link cases, parser suspension/cancellation/error paths and all tests using the replaced polling helpers. Counts overlap and must not be added as distinct coverage. The repaired source has not yet established a passing new native three-OS CI matrix. The prior failing1.99.1 CI receipts are preserved, not overwritten or upgraded by local success.
