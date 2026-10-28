# Common public adapters

Task 66 ports AddonManager, BufferApiView, BufferNamespaceApi and ParserApi. Task 214 covers dynamic JavaScript identifiers. Task 136 covers Number-based cell lookup and translation; task 191 retains additional host coercion and reentry cases. UnicodeApi belongs to task 23.

The detailed source contracts are in `docs/acceptance/public-adapters.md`, criteria 213-220. Full equivalence is NOT PROVEN.

## Implementation

- `crates/xterm-core/src/public_api/addon_manager.rs` records ownership before activation, returns managed disposal handles, preserves reverse order and partial failure state, and retains callback error causes. Native handles replace JavaScript method mutation.
- `buffer.rs` exposes live buffer/line identities and refreshes namespace views after reset. Wrapped and stale CircularList slots remain visible, as in the source. Replacing a backing buffer releases its payloads only after ending the view borrow.
- `parser.rs` shares weak registrars for all five handler families. Registration works inside callbacks without borrowing the running engine. Parameters and UTF-16 payloads are owned snapshots. Low-level packed routes preserve the source's separately evaluated CSI window-policy decision.
- Browser bindings live under `crates/xterm-web/src/terminal/`. The identifier reader repeats actual JavaScript property reads and preserves UTF-16 charCodeAt, setter behavior and validation order. Small JS helpers perform boundary operations only, not terminal parsing.
- Browser disposal tokens use the shared plain-object factory. The compatible facade owns addon records through a JS WeakMap; Rust releases its strong array reference. This fixes the identified standard addon-to-Terminal ownership cycle, but GC integration tests remain required.

## Checks

```sh
node scripts/public-addon-oracle.mjs
node scripts/public-buffer-oracle.mjs
node scripts/check-public-adapters.mjs
node --test scripts/check-public-adapters.test.mjs
node scripts/public-parser-identifiers-oracle.mjs --reuse-built
WASM_TEST=public_adapters_wasm node scripts/browser-host-check.mjs
node scripts/public-parser-browser.mjs
```

Observed on Windows: 27 native adapter tests pass, including callback-time registration/disposal and payload-Drop reentry. Scoped Clippy passes with warnings denied. The original AddonManager suite passes both cases, and directed addon/buffer checks pass. The identifier oracle passes 195 cases using existing original ESBuild modules whose source-map contents match the reference TS. A fresh-bundle oracle mode is available by omitting `--reuse-built`; `--generate` alone regenerates expected data.

The browser entry groups component assertions in `public_adapter_cases/` to avoid linking duplicate test binaries. It includes actual Terminal DOM cleanup checks. The original-browser runner retains every Parser.test.ts assertion and the Terminal.test.ts loadAddon group; only its fixture transport changes. The ASCII parser fixture explicitly omits unrelated Unicode-graphemes addon setup.

Remaining: execute current WASM and original-browser assertions after Root/Viewport integration compiles, verify GC ownership against the actual facade, finish the additional coercion cases, and obtain independent review. Native component passes do not close those gaps. Generated expected tables and older detailed logs are retained; they are data, not additional implementation modules.
