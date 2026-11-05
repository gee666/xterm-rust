# Shared browser WebGL atlas cache

`terminal::runtime::RuntimeContext::webgl_atlas_cache` owns one `BrowserAtlasCache<headless::Terminal>` per explicit browser runtime. Renderers and canvas layers must acquire it from that runtime rather than construct independent pools. The terminal's existing native `Rc<Terminal>` is the stable owner identity. Replacing a renderer must reuse that identity and explicitly release it on renderer disposal or WebGL context restoration.

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

The source runs only as the test oracle. The Rust path creates real native terminal owners and real Rust canvas atlases. Tests compare owner/atlas identity traces, lazy service phases, actual canvas removal/error identity, creation/disposal reentry, page version deltas, glyph pixels and shared clearing. Version deltas subtract the first source page's creation version because its module clock survives between scenarios. Actual run evidence will be listed in `results/browser-atlas-cache/` after execution. Scoped passes do not prove full equivalence.
