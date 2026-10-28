# Marker callbacks during CSI

ED, IL, DL, SU and SD run through shared-state command checkpoints. The terminal and buffer borrows end before list events or marker disposal. Callback errors retain their original cause and stop the command after the source-defined partial mutation.

IL and DL cache their absolute indices before disposal callbacks. A public reset can replace the active buffer, so their subsequent insertion deliberately preserves sparse slots beyond its new length. SU and SD instead reload the active buffer and margins between removal and insertion. ED fills a row before disposing markers and then clears wrapping on the current buffer, which may have changed during disposal.

## Checks

`cargo test -p xterm-core --test terminal_marker_reentry` passes six tests. They cover five ED forms, IL/DL, SU/SD and its alias, public reset, live option changes, marker registration, exact callback error identity, partial mutations and following parser input.

`node scripts/marker-csi-oracle.mjs` runs ten reset cases against the unchanged source parser and public reset API. Results are in `results/csi-marker/source.json`. Set `SOURCE_REPO` when the reference and its Node dependencies are outside this worktree.

Run the WASM callback checks with:

```sh
WASM_PACKAGE=xterm-wasm-support WASM_TEST=wasm_marker_reentry node scripts/browser-host-check.mjs
```

The runner also accepts `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` paths. Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 passed both Rust tests. Each browser executed the ten command cases and an ED reset followed by printable input. These use the Rust public headless terminal compiled to WASM, with callbacks crossing JS. They do not prove the incomplete browser Terminal binding or ED3 selection/viewport integration, which remains task183.

The eleven native CSI/edit/erase test executables pass 65 tests. Scoped Clippy with `-D warnings` and rustfmt on the changed Rust files pass. Workspace `cargo fmt --all -- --check` cannot launch rustfmt on this Windows worktree because its generated command exceeds the Windows argument length limit. This is not a formatting pass.

## CI refresh

[Run 40659600619](https://github.com/gee666/xterm-rust/actions/runs/40659600619) tested the earlier branch commit `5f565a4`. Its focused Miri job passed 21 tests. Its native Windows, Linux and macOS quality jobs failed, as did WASM compilation. This is observed failed CI, not missing OS access and not a pass for these new changes. The run metadata and job logs are retained under `results/quality/ci-40659600619/`. Current xterm-web compilation also fails on unrelated missing browser adapters, so the DOM-free WASM package isolates the terminal callback checks without claiming browser binding coverage.

[Run 40724491868](https://github.com/gee666/xterm-rust/actions/runs/40724491868) tested the current implementation at `e310426`. At the last observation five native jobs had failed quality-runner verification before Cargo gates, WASM compilation had failed, focused Miri had passed, and macOS MSRV was still queued. Metadata is in `results/quality/ci-40724491868/run.json`. It is already non-green, but not a completed matrix.

## Integration still pending

Independent `swarm_review` accepted tasks176 and241 and reran all 65 focused tests, six release regressions and scoped lint checks. The canonical merge refused integration because the existing `core_services_integration` logger regression requires every callback to observe x=2. Restoring only the three changed CSI implementation files to main reproduced that same failure. Task264 already owns the source-correct diagnostic phase regression. Task270 tracks rebasing and merging this branch after264 lands. The implementation is reviewed and pushed, but is not yet on main.

Full equivalence remains NOT PROVEN.
