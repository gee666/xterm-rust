# Keyboard object-map lookup and coercion

Task343 recovers the raw Win32 and Kitty policies retained from historical tasks184/185/207. The retained Win32 counterexamples remain in `results/win32-input/prototype-counterexamples.json`. See `docs/keyboard-recovery343.md` for the recovered implementation, commands and current scoped evidence. Full input/accessibility parity remains task319.

## Required semantics

The upstream maps are ordinary objects, not own-property dictionaries. String keys such as toString, constructor and __proto__ are valid synthetic event strings. They retrieve inherited functions or objects rather than undefined. Win32 can place those values in Vk, Sc and Uc. Kitty can place them in legacy suffixes or CSI parameters.

- Preserve the raw result of each lookup until its actual coercion expression. Win32 resolves Vk, Sc, Uc and control state first, then interpolates the cached values into the final template. Converting a function to text at lookup time changes observable getter/coercion ordering.
- Vk and Uc reject only undefined. Sc applies `value || 0`, so false, zero, empty string, null, undefined and NaN take the zero branch. Do not treat all absent/falsy cases alike.
- A prototype getter receives the actual lookup table as `this`, including its own entries and stable identity. Calling `Reflect.get(Object.prototype, key)` is not equivalent. A getter may inspect, mutate or retain its receiver.
- Keep per-encoder table identity across events. Do not construct a temporary table for each lookup. Mutations made through an inherited getter's receiver must affect later own-property lookups too.
- Preserve the conversion hint. Binary string concatenation in Kitty and template interpolation in Win32 can invoke different ToPrimitive hints on objects. Symbols and throwing conversions must produce the source error at the source phase, with the original thrown value preserved.
- Browser native-function strings are runtime-defined. Node 24/V8's observed one-line text is a native oracle assumption, not a Firefox/WebKit implementation. Capture the actual host realm's representation rather than normalize its whitespace.
- Keep UTF-16 output lossless, including an inherited string containing a lone surrogate. Avoid routing arbitrary inherited text through Rust String::from_utf16_lossy.

## Shared boundary

`keyboard_event_source::lookup::{Value, HostValue, Hint}` distinguishes undefined, null, bool, number and UTF-16 text, plus an opaque host value whose synchronous conversion returns the event source's generic error type. `KeyboardEventSource::win32_lookup` and `kitty_lookup` provide ordinary lookup at the source expression. Borrowed native events use immutable values and the documented Node/V8 intrinsic text. Browser adapters retain raw JsValue objects and perform only the required language conversion at the correct call site. The core has no browser dependency.

Browser `input::keyboard_lookup::KeyboardTables` retains stable table objects alongside InputBindings, with the source's own properties and ordering. It performs all lookups against those objects, not only inherited misses, because a prototype getter can mutate their own entries. No Rust borrow crosses a getter or conversion callback. The small attributed `kitty_tables.js` and `win32_tables.js` files intentionally duplicate only source mapping literals and language conversion operations. Their generators check exact literal drift. Object literals create own properties without invoking inherited setters; Rust owns all encoding decisions.

The adapters reuse the existing wasm-bindgen/js-sys dependencies. No JavaScript terminal engine or unsafe code is added. Event fields whose values themselves violate declared DOM primitive types are a separate task319 boundary. Map-result coercion does not complete that larger contract.

## Evidence

Use unchanged source for each runtime and compare full outputs, ordered field reads, table receiver identity/mutations, conversion hints and exact thrown markers. Include Node/native plus actual Chromium, Firefox and WebKit. Keep ordinary 64-test Win32 and 508,249-case Kitty regressions green, and retain every discovered counterexample. A hardcoded function-text table alone cannot satisfy the browser portion.
