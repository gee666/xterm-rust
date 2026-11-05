# Isocrates 470 handoff

Task 470 is accepted. Branch `swarm/IsocratesMouse470` starts at `32bf9544`. Pick source commit `89ff4693`, then its review/cleanup receipts, as a new producer contribution. Do not replay any Lilly461 or other producer ancestors. Integration belongs to task 309; no main-ref operation was performed by this lane.

The only production change is `crates/xterm-web/src/terminal/mouse_api.rs`. The actual native-backed facade now exposes `restrictMouseEvent` and `encodeMouseEvent` for its existing built-in modes. It selects shared raw Rust policies from the current native active name without another mode store. Selection happens before field access, so getters can recursively switch modes while the current call finishes its original policy. Arbitrary throws and UTF-16 strings preserve their identities/units. Raw custom registries and full receiver shape remain task 305.

The runner change in `scripts/mouse-mode-source.mjs` overlaps pending accepted Lilly461 only at case-family selection. Preserve both additions when composing that file: `modes` maps to `browser_mouse_mode_cases.js`/`mouse-modes`, `wheel-receiver` to `browser_wheel_receiver_cases.js`/`wheel-receiver`, `builtin-methods` to `browser_mouse_builtin_cases.js`/`mouse-builtin-methods`. The native mode producer and wheel receiver production files are not changed here.

`docs/mouse-builtin-methods.md` and `results/mouse-builtin-methods` retain commands and evidence. Four unchanged-source cases and four actual Rust test functions passed Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Existing five mouse-mode functions passed all engines. Only replacing mouse_api.rs with exact32bf caused all four new Chromium assertions to fail; a finally restored production bytes exactly before the final passing matrices. Native24 tests and scoped strict native/WASM lint/checks passed. Full workspacefmt OS206 and all five memberfmt passes are recorded honestly.

The isolated reviewer independently verified 159 retained artifact pins, 11 source pins and reference/case hashes, then executed all 24 native tests. It inspected, rather than repeated, compiler/browser receipts. Review receipt is `results/mouse-builtin-methods/independent-review.md`. All heavy grants were released in post409/3231. Rust/Cargo1.85.1 and Node24.15.0 were used. This was built on32bf; later main3a4 changes still need integration validation.

The only owned temporary tree is `O:/xterm-rust-swarm-16/tmp/worktrees/IsocratesMouse470`. Runners closed browsers/servers through finally. Before deleting the tree, verify no live process or link/redirect, clean tracked state, durable commits, all artifact pins and original776 reference hashes. Delete only own generated untracked browser-host files, then use normal worktree removal. A final cleanup receipt follows. Preserve source branches and do not traverse any reference junction.

Overall equivalence remains NOT PROVEN.
