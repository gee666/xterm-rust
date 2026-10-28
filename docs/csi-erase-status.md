# CSI erase status

ED/EL/ECH/DECSCA are implemented in `terminal/csi_erase.rs`. The corrected-dimension native suite passes all 24 tests, including 337 differential traces, 3,937 full-state snapshots and all five preserved VT inputs. Scoped Clippy and rustfmt pass.

The unchanged InputHandler baseline passed 194 tests. The source mapping covers 22 common cases and six browser-content cases. One negative private-cursor ECH precondition remains in task 152. Browser-content tests running natively are not actual browser proof.

ED3's incorrect public scroll event was fixed with private `InputScroll`; its regression passes. Task 183 still owns actual browser selection/viewport effects. Task 176 owns terminal-level reentry from marker callbacks. Original EL legacy-xterm output remains intentionally different, but Rust matches the real JS result for EL and the other four fixtures.

Commands: `cargo test -p xterm-core --test csi_erase_upstream --test csi_erase_cases --test csi_erase_playwright_assertions --test csi_erase_edges --test csi_erase_differential --test csi_erase_vt`; the same targets with `cargo clippy ... -- -D warnings`.

Overall equivalence is not proven. Acceptance details remain in `docs/acceptance/csi-erase.md`.
