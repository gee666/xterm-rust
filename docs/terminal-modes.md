# Terminal mode implementation and evidence

Task102 ports InputHandler standard/private set/reset, DECRQM and Kitty keyboard negotiation. The path-by-path contract is `docs/acceptance/terminal-modes.md`.

## Borrow and callback boundaries

`modes.rs` contains state-only mutations. `mode_callbacks.rs` performs option setters/getters, mouse protocol callbacks and buffer activation without a terminal-state borrow. `mode_handlers.rs` connects parser callbacks, flushes each main parameter before the next and logs keypad/unsupported-mode messages before mutation. `mode_query.rs` and `kitty_modes.rs` own their protocol calculations. `commands.rs` registers those actual factories.

DECRQM captures the source numeric columns before observable getters. Query9001 uses CoreService's read-only capture guard from task198 so in-place changes remain visible while reset preserves the old mode object. No RefCell borrow survives the getter. Native tests capture weak service/terminal references rather than introducing callback cycles.

Mode3 and RIS use the common reset request phase owned by the engine. The headless wrapper must not execute a second reset. Actual browser focus/scrollbar/render effects remain task197 and browser integration work, not a native substitute.

## Current observed checks

Native Windows debug tests passed with zero failures/ignored cases in the controlled receipt `results/build-slots/1793173753335-45516-acc15ea4-2c12-4d67-b7ac-5b1620953930/receipt.json`:

- 17 standard/private/query/Kitty tests.
- Six faithfully preserved original alternate-buffer/cursor tests, including SGR colors and per-buffer saved cursor.
- Nine live callback/option/query tests, including disabled stdin, prior-parameter state, logger failure, reset identity and getter resize.
- One differential test comparing2,136 complete mode/cursor/buffer/stack/reply snapshots at seed0x6d0de5.

That is33 mode tests. The same command also passed27 separate mouse tests; they are not counted again as mode tests.

`node scripts/terminal-modes-oracle.mjs` ran all20 selected original setMode/DECRQM/alt-screen/Kitty cases without changing assertions and checked the deterministic2,136-operation corpus. `node scripts/terminal-mode-callback-oracle.mjs` independently ran the added getter/log/callback cases against the actual source headless Terminal and Params. Its original8 cases passed in a fresh source build; the explicit ninth disabled-stdin case subsequently passed with the verified cached source bundle. Current callback report is `results/terminal-modes/callback-reference.json`; it states cached versus fresh provenance rather than inventing a rebuild.

The first alternate saved-cursor assertion failed because general CSI registration was absent. Its assertion was retained; the engine registration fix made all six original cases pass. Full-engine comparison then exposed immediate refresh ordering and activation dirty ranges. Those were fixed against the original event sequence, not normalized away. Historical failures remain under `results/terminal-modes/`.

Current scoped Clippy attempts fail in unrelated unfinished Buffer cleanup ownership code, not in these mode files. Exact failure receipts include `results/build-slots/1793173304020-25696-f1cb3c94-0e76-4e98-8310-1990d1d79e14/receipt.json`. No warning suppression was added and no whole-workspace lint pass is claimed. Release/full matrices follow the coordinated quality task rather than being hidden inside local debug leases.

## Runtime boundaries

Task197 implements vt28/29/30/31/34/40/41 using actual Rust-backed WASM and independent original JavaScript pages in Chromium/Firefox/WebKit. Its current attempts reached the real WASM compiler but failed browser-service/scheduler assembly before any browser comparison. `results/browser-capture/compile-vt28.md` lists the concrete errors. No browser mode, visual or pointer/keyboard integration pass follows from the native tests.

Overall functional equivalence remains NOT PROVEN. Full quality, browser integration, native Linux/macOS and remaining application/addon gates are still required.
