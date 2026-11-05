# Atlas-page test lint repair

Task 452 repairs one observed warning-denied WASM test failure. It is not full workspace quality or application parity. Final task 304 remains open.

Run [42090157228](https://github.com/gee666/xterm-rust/actions/runs/42090157228), immutable source `3aa04692a409bc147670d7c61a9e2ac37b5ea9f8`, passed full workspace WASM all-target/all-feature compilation on actual stable Rust 1.99.1. Its subsequent all-target Clippy failed at `wasm_webgl_atlas_page.rs:217` with `cloned_ref_to_slice_refs`. The original log is retained as `results/quality/atlas-page-lint452/ci-old-stable199.log`; checkpoint 444 owns the original artifact ZIP and complete run evidence.

Source commit `bc80e585` changes only `&[page.clone()]` to `std::slice::from_ref(&page)` in the merge-constructor test. `CanvasAtlasPage::new` takes `&[Rc<Self>]`, reads each page and clones its glyph handles into the new page. Borrowing the existing Rc supplies the same page identity without an unnecessary temporary reference-count increment. The constructor, all assertions, source fixtures and browser code remain unchanged. No dependency, unsafe operation or lint allowance was added.

## Observed local checks

The focused command passed on Windows with Rust 1.85.1:

```sh
cargo +1.85.1 clippy --locked -j2 -p xterm-web --all-features --target wasm32-unknown-unknown --test wasm_webgl_atlas_page -- -D warnings
```

Changed-file rustfmt also passed. Logs and before/fixed/restored SHA-256 values are under `results/quality/atlas-page-lint452/`.

Two checks did not establish the intended extra proof and remain explicit:

- The old-call MSRV control passed. Rust 1.85.1 does not reproduce the current-stable diagnostic. Its receipt is retained, not called a negative-control failure. The observed Rust 1.99.1 CI failure is the old-call failure evidence.
- The attempted fixed Rust 1.97.1 WASM check failed with E0463 because that toolchain lacks the WASM target. No target installation or shared toolchain modification was performed. This is not a repaired-source compiler diagnostic and is not a successful check.

The fixed source was restored in `finally` after every attempt. The failed control-attempt receipt remains separate from `fixed-checks/`; neither was overwritten. Actual stable Rust 1.99.1 repaired CI is UNRUN here and must be observed on a later integrated snapshot. No browser runtime check was granted or run for this test-only argument correction.

The coordinator granted one scoped compiler lane in post 566 comment 2883. It is released, with zero owned compiler/browser/server jobs. The only temporary work remains the owned regular `O:/xterm-rust-swarm-16/tmp/S304` worktree, target and tmp directories. Review is source/light only unless a new heavy grant is issued. Task 444 continues observing the independent immutable checkpoint; none of these changes alters its source or result.
