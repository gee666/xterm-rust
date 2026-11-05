# Public custom link providers

Task 415 adds the missing `Terminal.registerLinkProvider` binding over the existing Rust provider registry and DOM controller. It is available before `open` and is not proposed-API gated, matching the source. The returned ordinary JavaScript disposal token removes the first registration of the original provider identity on every invocation. Ignoring the token does not unregister it.

The browser controller now retains `BrowserLink::JavaScript` payloads rather than copying public provider text, ranges or callbacks. `BrowserLink::Native` retains the existing typed OSC and utility-addon path. Rust callers construct `NativeBrowserLink` and wrap it in the native variant. `current_link` returns the retained payload, `range` reads a fresh selection range, and `snapshot` is a fallible diagnostic read, never the controller's hit-test authority.

Provider priority, cache traversal, overlap removal, current/mousedown state, hover, activation and invalidation remain Rust code. They use the existing generic `ProviderReplies` algorithm and the shared `HoverState` tracker. No new dependency, unsafe code or JavaScript terminal implementation is used. Small host helpers read properties, call functions with their original receiver, define descriptors and preserve JavaScript primitive operator coercions.

Each cached wrapper owns its optional hover state. A throwing decoration getter leaves the current wrapper assigned without inventing a successful state. Decoration accessors are installed after the hover callback. They retain raw values, including undefined and object identity. Old descriptors address the current wrapper as source does, while underline setters use their captured original link. JavaScript owns the link-to-descriptor edges; Rust callbacks retain only weak controller state, avoiding a cross-runtime self-retention cycle. Pending provider callbacks retain only the mouse-out flag and a weak controller reference. Source still evaluates `links.map` on a late reply after disposal if the pointer was not out, and its thrown value is preserved.

## Directed evidence

`scripts/public-link-provider-trace.js` runs unchanged against the original opened Terminal and the exported Rust Terminal. The source runner checks the original bundle source map against the supplied Linkifier, LinkProviderService and public Terminal source bytes. Its fixture is generated only from the original.

The 20 phases cover pre-open registration, hover and descriptor flags, lossless UTF-16 callback text, mutable text/range/callbacks, mousedown equality, same-cell suppression, raw and undefined decoration values, old descriptors, ordered range getters, partial-decoration timing, duplicate disposal, asynchronous priority, ignored mouse-out replies, exact provider/range/decoration/hover thrown-object identity, partial hover state and late disposed replies.

Commands from the repository root, using the existing installed matching wasm-bindgen and Playwright tools:

```sh
node scripts/public-link-provider-source.mjs
WASM_TEST=wasm_public_link_provider node scripts/browser-host-check.mjs
node scripts/e2e-upstream.mjs --suite core --grep registerLinkProvider --timeout-ms 180000
```

The original six Terminal integration tests run in all three browsers, for 18 expanded cases. Source assertions and fixtures are unchanged. Final command accounting and raw receipts are retained under `results/public-link-provider/`.

## Remaining family work

This scoped API repair does not close tasks 315 or 321 or establish full equivalence.

- Typed native OSC and web-links callback range objects still need live mutation parity with subsequent controller queries. Stable callback object identity alone does not supply it.
- Complete raw array/subclass/custom `map`, sparse arrays, malformed payload and prototype/Proxy behavior is not proven. The public adapter currently materializes the mapped reply as retained payloads, rather than preserving arbitrary custom map return containers.
- Drawing and selection adapters still require finite 32-bit integral coordinates. The raw hit-test operators preserve JavaScript arithmetic, but out-of-domain drawing values produce an explicit RangeError rather than proving arbitrary-number equivalence.
- Full descriptor/GC/lifecycle retention, all reentrant getter phases, scroll/reflow selection interactions and randomized live-object traces need the broader family audit. Weak descriptor state does not claim source lifetime equivalence after the controller is dropped.
- Native Linux/macOS runtime, full original/browser/demo coverage and stable final workspace gates remain separate project requirements. A scoped browser pass is not complete coverage.

Accepted tasks 401 and 405 are prerequisites. The branch preserves their screen DOM/geometry and no-current-link mouseup behavior and reruns their component assertions rather than treating them as new fixes.
