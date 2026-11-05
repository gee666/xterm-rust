# WebGL incremental renderer engine

Task 449 implements the missing typed `_updateModel` and `renderRows` algorithms from the supplied `addons/addon-webgl/src/WebglRenderer.ts`. It reuses the existing `RenderModel`, `CellColorResolver`, `SelectionRenderModel` and `GlyphCell` contracts. There is no new dependency, unsafe code or JS terminal-engine delegation.

## Contracts

`webgl::renderer_update::RendererState` owns the packed model, reusable work cell, resolver and per-row blinking history. `resize` allocates the model and resets blinking history. `clear_model` intentionally leaves resolver colors and blinking history intact. `reset_blinking_rows` is the separate clear/resize operation required by the source.

`UpdateHost` supplies lines, character joins, color resolution and actual glyph/background/cursor consumers at their algorithmic call sites. Errors propagate immediately, without rolling back earlier model or consumer writes. `FrameHost` supplies attachment, layer notification, atlas probes and draws. Neither trait holds an engine-owned `RefCell` borrow across callbacks. No implementation of the public browser addon is implied by these traits.

The engine preserves:

- Clamped/reversed row updates, missing-row null glyphs, nonzero viewport offsets and line draw lengths.
- Selection-uniform joins, cursor exclusion from joins, joined-cell fg/bg inheritance without extended attributes, clearing joined continuation cells and UTF-16 combined-code sentinels.
- The source's surprising unchanged-join behavior. When the cache check continues before the joined-cell reset, subsequent loads still target the joined-cell object with its fixed width and fake code. This state persists across rows within that call, but not across separate calls.
- Cursor width/style/focus and block-color overrides, text-blink invisibility, row blink counts across partial updates, and the conditional background update followed by unconditional cursor/blink updates.
- Initial atlas invalidation, full-model retries after merges, one texture invalidation after any retry and background/glyph/cursor draw order. The 32-retry limit still performs the source's final `beginFrame` probe, giving 34 probes when the initial probe is false and all later probes are true.

`CellColorResolver::result_numbers` retains JavaScript signed numeric results alongside the existing packed words. BufferLine supplies unsigned numbers; bitwise overrides can produce negative numbers with identical bits. Comparing only u32 words would incorrectly suppress glyph updates. `GlyphColorNumbers` preserves these exact callback numbers, including the previous background. A future GPU adapter must use their numeric inequality for overlap clipping rather than comparing only packed words. `override_cursor` and `hide_foreground` preserve the numeric representation of the renderer's own bitwise writes. Read numeric results after these methods or `resolve`, not after arbitrary direct writes to `result`.

## Verification

`scripts/webgl-renderer-oracle.mjs` bundles unchanged source modules in memory, creates an object with the actual `WebglRenderer` prototype, and executes the original methods. Only DOM/GPU construction is bypassed. Real source BufferLine, CellData, RenderModel, SelectionRenderModel and CellColorResolver run against typed recording dependencies. No source file or original assertion is edited. The fixture pins all 35 imported source files, the generator, wrapper and bundle.

The fixture contains 164 directed/seeded cases and 459 stateful steps. There are 128 seeded cases, seed `0x54b7d329`. Each step compares every packed cell, line length, cursor field, blinking row/count, resolver word, signed callback argument and callback order. Four injected consumer errors compare partial state and recovery. Directed cases include empty combined strings/NaN, grid variants, signed cache misses, unchanged joined cells, selection/cursor joins, blink transitions, detached/not-ready frames, bounded repeated merges and large positive/negative row arguments through the maximum safe JS integer. A second native test rejects invalid grids and nonprogressing/out-of-grid native joins without a panic. Such invalid native joins are an explicit boundary error, not a claim of arbitrary raw JS range equivalence.

The existing cell-color oracle now records original numeric results as well as all packed words for its 2,956 cases, twice with a reused resolver. This is an additive assertion, not a replacement for the prior tests. Old `results/webgl-cell-colors` receipts describe task 441's earlier fixture/generator; the fresh hashes and receipts for this extension are under `results/webgl-renderer`.

Observed local Windows/MSRV 1.85.1 results:

- Seven focused native WebGL binaries, 62 default and 62 all-feature tests passed, zero ignored/filtered.
- Release renderer and color binaries, four tests passed.
- Strict Clippy passed for the library and both changed test targets natively and for wasm32, with all features.
- A real negative control replaced numeric cache equality with unsigned-word equality. The renderer binary failed exactly one of two tests. The exact mutant and log are retained. Source restoration was byte-identical before fresh default/release passes.
- Initial independent review found positive i64-to-usize overflow selected the first row on WASM instead of clamping to the last row. The fix saturates positive conversion overflow after clamping the negative bound. The original production clamp was then restored as a real negative control in the actual renderer WASM test. Chromium reported exactly one pass and one failure at `large-positive-rows`. The helper restored and hash-verified fixed source in `finally`; the full 164-case/459-step fixture plus invalid-boundary test then passed as two wasm-bindgen tests in each of Chromium, Firefox and WebKit, zero ignored/filtered. This is 32-bit typed engine runtime evidence, not a DOM/WebGL addon test.
- Exact `cargo fmt --all -- --check` failed on Windows OS error 206. `cargo fmt -p xterm-web -- --check` passed. The latter is not a full-workspace formatting pass.

Commands are in `.pi/swarm/skills/webgl-renderer-check/SKILL.md`. `results/webgl-renderer/checkpoint.json` records source/artifact hashes, commands, tools and scope. No public-addon/GPU browser runtime, full workspace test/Clippy gate, unchanged full WebGL/custom-glyph/demo suite, Linux or macOS execution was performed for this task. The two typed engine tests did run as 32-bit WASM in all three browsers. Browser receipts pin ephemeral compiler/binding artifacts; those disposable binaries are removed during owned cleanup.

## Remaining task 306

`UpdateContext` is a native typed snapshot, not a raw JS getter adapter. Do not wire it to browser services by pre-reading observable getters, decoration callbacks or joiner arrays and claim source phase equivalence. Task 306 still owns that live host work, actual WebglRenderer construction/layers/dimensions, shared atlas cache/limits/clock, cursor/text browser timers, WebglAddon activation/context/atlas events, Root renderer replacement/restoration and complete original browser/demo suites. Existing actual GPU/atlas/custom-glyph components must be reused. The repository still does not export a working public WebglAddon. Full equivalence remains NOT PROVEN.
