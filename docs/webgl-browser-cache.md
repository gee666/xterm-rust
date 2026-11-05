# Shared browser WebGL atlas cache

`terminal::runtime::RuntimeContext::webgl_atlas_cache` owns one `BrowserAtlasCache<headless::Terminal>` per explicit browser runtime. The existing `browser/xterm.mjs` bootstrap already passes its shared `BrowserRuntime` to each default Terminal. Renderers and canvas layers must acquire the cache from that runtime rather than construct independent pools. The terminal's existing native `Rc<Terminal>` is the stable owner identity. Replacing a renderer must reuse that identity and explicitly release it on renderer disposal or WebGL context restoration.

The pool composes the existing portable `AtlasCache` with the actual browser `TextureAtlas`, not a stand-in factory. Every created atlas receives the same `AtlasRuntime`. That shares the NULL color/glyph identities, page upload clock and texture/page limits across otherwise different configurations. Equal configurations share the same atlas object, pages, rasterized-glyph aliases and contrast caches. Configuration creation copies the ANSI list and retains its color object identities, as before.

`AtlasRequest` contains validated Rust terminal options, colors and dimensions. The service closure runs only after an atlas miss and after releasing a previous single-owner entry. It returns the real Unicode/log services or an error. No pool borrow spans that callback or atlas construction/disposal. Creation failure does not insert an entry. Disposal failure propagates the same JavaScript error object and preserves the current owner, subject to source reentrant list mutations.

The first runtime call captures the source-global construction document. Moving a terminal into an iframe or another window does not switch the document used by the cache, matching `CharAtlasCache.ts`'s use of global `document`. Runtime cache lookup itself does not invoke DOM methods or callbacks while its lazy slot is borrowed.

## Representation and remaining work

This is a typed internal Rust service. It does not accept arbitrary JavaScript configuration objects, custom property getters or substituted JS Unicode/log services. Existing typed configuration comparison and `Weak` owner identities remain deliberate Rust representations. Explicit `remove_terminal` still works after the terminal's final strong reference drops. Dropping a cache or terminal does not invent source disposal events.

The actual browser renderer, public `WebglAddon`, Root renderer replacement/restoration and complete unchanged WebGL/custom-glyph/demo suites remain task306. Adding this service to the real `RuntimeContext` does not install a WebGL renderer or finish that task.

## Verification

`scripts/browser-atlas-cache-oracle.mjs` appends a test observer to the unchanged `CharAtlasCache.ts` source and bundles it with the real source `TextureAtlas` in memory. It never writes source inputs. The separate wrapper follows the existing atlas oracle pattern because this probe must observe the private cache entries and module clock as well as atlas behavior. No new dependencies are introduced.

```
SOURCE_ROOT=/path/to/original node scripts/browser-atlas-cache-oracle.mjs
WASM_TEST=wasm_webgl_browser_cache WASM_PRELUDE=tmp/browser-atlas-cache/source.js node scripts/browser-host-check.mjs
cargo test -p xterm-web --test webgl --all-features -j2
cargo clippy -p xterm-web --lib --test wasm_webgl_browser_cache --target wasm32-unknown-unknown --all-features -j2 -- -D warnings
```

The source runs only as the test oracle. The Rust path creates real native terminal owners and real Rust canvas atlases. Tests compare owner/atlas identity traces, lazy service phases, actual canvas removal/error identity, creation/disposal reentry, page version deltas, glyph pixels and shared clearing. Version deltas subtract the first source page's creation version because its module clock survives between scenarios. `results/browser-atlas-cache/` retains the executed commands, reports and input/artifact hashes. Scoped passes do not prove full equivalence.

On native Windows with Rust/Cargo 1.85.1, the focused native `webgl` target passed46 tests. Native and WASM all-feature lib/test Clippy passed with warnings denied. The final nine-test WASM target compiled with all features; browser execution used the default feature set.

| Matrix | Chromium148 | Firefox150 | WebKit26.4 |
| --- | --- | --- | --- |
| Initial completed cache suite | 9passed | 9passed | 9passed |
| Restored final cache suite | 9passed | 9passed | 9passed |

Each browser executed all eight unchanged-source differential scenarios and the real runtime/iframe test, with zero failed, ignored or filtered tests. Directed scenarios cover sharing and shared clear pixels, owner movement, same-error disposal failure, actual canvas creation failure, lazy service failure, disposal reentry, service lookup reentry and canvas-constructor reentry. The latter preserves page-version deltas `[0,-1,0]` when a nested atlas finishes before the outer page. This is finite directed coverage, not randomized or complete cache equivalence.

The production negative control replaced shared runtime forwarding with a fresh `AtlasRuntime` for every atlas. Chromium produced1passed/8failed. Exact original bytes were restored in `finally`; the full final three-engine matrix and strict all-feature WASM Clippy then passed. `negative.json` records the original, mutant and restored hashes.

Two development matrices each produced8passed/1failed per engine because the source fixture first lacked `wcwidth`, then lacked an active Unicode provider. Both failed reports remain. The fixture now uses the unchanged source `UnicodeService` and registers the unchanged source `UnicodeV6`, matching the Rust Unicode service. No production behavior or assertion was weakened to fix fixture setup.

Final package formatting passed. The exact required workspace `cargo fmt --all -- --check` failed with Windows OS206; its raw log remains and is not counted as a pass. Full workspace/release/native Linux/macOS and full addon/renderer acceptance are not established by this task.

Run `SOURCE_ROOT=/path/to/original node scripts/browser-atlas-cache-audit.mjs --runtime` before cleanup to validate original/source/durable/runtime pins. Default audit omits disposable runtime files after cleanup. `--record` creates a producer pin snapshot; it is not independent execution or review.
