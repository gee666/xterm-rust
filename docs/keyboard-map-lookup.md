# Keyboard object-map lookup and coercion

Tasks 184 and 185 cover Win32. Task 207 covers Kitty. This is a contract and design note, not an implementation or passing evidence. The retained Win32 counterexamples are `results/win32-input/prototype-counterexamples.json`.

## Required semantics

The upstream maps are ordinary objects, not own-property dictionaries. String keys such as toString, constructor and __proto__ are valid synthetic event strings. They retrieve inherited functions or objects rather than undefined. Win32 can place those values in Vk, Sc and Uc. Kitty can place them in legacy suffixes or CSI parameters.

- Preserve the raw result of each lookup until its actual coercion expression. Win32 resolves Vk, Sc, Uc and control state first, then interpolates the cached values into the final template. Converting a function to text at lookup time changes observable getter/coercion ordering.
- Vk and Uc reject only undefined. Sc applies `value || 0`, so false, zero, empty string, null, undefined and NaN take the zero branch. Do not treat all absent/falsy cases alike.
- A prototype getter receives the actual lookup table as `this`, including its own entries and stable identity. Calling `Reflect.get(Object.prototype, key)` is not equivalent. A getter may inspect, mutate or retain its receiver.
- Keep per-encoder table identity across events. Do not construct a temporary table for each lookup. Mutations made through an inherited getter's receiver must affect later own-property lookups too.
- Preserve the conversion hint. Binary string concatenation in Kitty and template interpolation in Win32 can invoke different ToPrimitive hints on objects. Symbols and throwing conversions must produce the source error at the source phase, with the original thrown value preserved.
- Browser native-function strings are runtime-defined. Node 24/V8's observed one-line text is a native oracle assumption, not a Firefox/WebKit implementation. Capture the actual host realm's representation rather than normalize its whitespace.
- Keep UTF-16 output lossless, including an inherited string containing a lone surrogate. Avoid routing arbitrary inherited text through Rust String::from_utf16_lossy.

## Suggested shared boundary

A small portable lookup-value abstraction can distinguish undefined, null, bool, number and UTF-16 text, plus an opaque foreign value whose synchronous coercion methods return the event source's generic error type. Borrowed native events use infallible native values. Browser adapters retain raw JsValue objects and perform only the required language primitive at the correct call site. This avoids forcing JsValue or browser dependencies into xterm-core.

The browser provider needs stable table objects with the source's own properties and ordering. A provider must perform all lookups against those objects, not only inherited misses, because a prototype getter can mutate their own entries. Creating the tables must not invoke inherited setters that an object literal would bypass. Shared generated mapping data or small binding data literals can preserve the source own entries without running the JavaScript terminal engine. Explain intentional data duplication and preserve attribution if generated Rust and binding tables are both needed.

Use the project's existing wasm-bindgen/js-sys dependency for host primitives. Do not add a JavaScript terminal engine or replace the Rust encoding decisions with a call to xterm.js. Task 202 separately covers event fields whose values themselves violate the declared DOM primitive types; do not silently conflate that larger boundary with map-result coercion.

## Evidence

Use unchanged source for each runtime and compare full outputs, ordered field reads, table receiver identity/mutations, conversion hints and exact thrown markers. Include Node/native plus actual Chromium, Firefox and WebKit. Keep ordinary 64-test Win32 and 508,249-case Kitty regressions green, and retain every discovered counterexample. A hardcoded function-text table alone cannot satisfy the browser portion.
