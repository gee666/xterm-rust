# OptionsService acceptance checklist

Reference: `source_repo/src/common/services/OptionsService.ts` and its unchanged `OptionsService.test.ts`. Implementation: `crates/xterm-core/src/options/`. Tests: `crates/xterm-core/tests/options_service.rs`. Oracle: `node scripts/check-options.mjs`. Full terminal equivalence remains NOT PROVEN.

## Defaults and construction

- Compare all 48 default keys, enumeration order and primitive values against `DEFAULT_OPTIONS`. Do not only count keys. Include wordSeparator punctuation, scrollbar.showScrollbar, each empty object, null external references, font choices, booleans and numeric defaults.
- Adapters supply the platform's Mac flag. Only rightClickSelectsWord changes with that flag. Core must not detect a browser or operating system itself.
- Preserve constructor input order, including inherited enumerable properties supplied by the adapter's for-in walk. Unknown names not inherited from Object.prototype are ignored. Valid entries replace defaults without firing events. Invalid entries retain their default, report one diagnostic each, and do not prevent later valid entries from applying. Undefined rows/cols and zero tabStopWidth are explicit regressions.
- Preserve object default identity across shallow copies and services on a UI thread. Primitive defaults are independent values. User-supplied objects retain identity and payload ownership.
- Rust returns constructor diagnostics instead of writing console.error. Adapter documentation must state how these become host diagnostics.

## Sanitization and rejection

- cursorStyle: every falsey value resets to block; only block, underline and bar are valid truthy values. Reject outline and object inputs with the source error wording.
- wordSeparator: falsey inputs use the exact default; nonempty UTF-16, including lone high/low surrogates, is retained.
- fontWeight/fontWeightBold: accept normal, bold and each 100-through-900 string in hundreds; numeric endpoints 1/1000 and fractional numbers inside the interval are retained. Reset out-of-range numbers, NaN, infinities, booleans and invalid strings to the correct individual default.
- blinkIntervalDuration and cursorWidth: perform JS ToNumber then floor. Blink rejects values below zero; width rejects values below one. Preserve NaN and positive infinity exactly as upstream. Check signed zero and fractions on both sides of each boundary.
- lineHeight/tabStopWidth: numeric comparison rejects values below one without replacing the original accepted value. Numeric strings remain strings. Undefined/NaN pass the upstream comparison.
- minimumContrastRatio: JS ToNumber, Math.round at tenths with ties toward positive infinity, clamp to [1,21], propagate NaN. Test half-tenths, both infinities, nonnumeric strings, null and undefined.
- scrollback: ToNumber, cap at 4294967295, reject negative results, retain fractions, propagate NaN. Test boundary neighbors, Infinity, negative zero, undefined and numeric strings.
- scrollSensitivity/fastScrollSensitivity: reject zero and negatives, retain accepted input type, permit NaN as upstream does.
- cols/rows: reject falsey values other than numeric signed zero. Preserve the source's truthy nonnumeric values rather than silently imposing extra validation.
- windowsPty: null/undefined create a fresh empty object each assignment. Other objects retain identity. All unsanitized keys preserve the given value and type.
- JS string-number conversion covers decimal/exponent signs, binary/octal/hex, BOM/ECMAScript whitespace, invalid digits, non-JS inf spellings, lone surrogates and nonnumeric text. Object ToPrimitive belongs to the host adapter. Do not substitute an unrelated Rust Debug string.
- On rejection, the existing value and event count remain unchanged. Compare exact error messages for supported primitive inputs.

## Events, mutation and cleanup

- Compare strict equality before emitting. Equal primitive values and the same object do not emit. Fresh objects and repeated NaN do. Positive/negative zero compare equal.
- Commit the new value before the general event, and read the current raw value when each specific listener runs. A preceding listener may change it again.
- General listeners receive the exact key. Specific listeners filter the exact key and receive the current value. Multiple-key listeners fire once if a key occurs in their list, including duplicate list entries.
- Preserve listener registration order, explicit unsubscribe behavior, recursive updates, subscription changes during dispatch and disposal during dispatch using the shared Event implementation.
- Raw writes bypass both validation and events, but ordinary getters immediately observe them.
- Dispose the event emitter before clearing linkHandler/documentOverride to null. Clearing does not emit. Release external references. Repeated dispose is harmless. Other values survive; subsequent writes still update values, but no disposed listener fires. Drop also cleans resources in Rust.
- No RefCell borrow crosses a callback. Listener closures must not create hidden strong ownership cycles.

## Prototype constructor and binding cases, task 109

- Source uses `key in defaultOptions`, not an own-key test. Explicit input properties named constructor, toString, toLocaleString, valueOf, hasOwnProperty, isPrototypeOf, propertyIsEnumerable, __defineGetter__, __defineSetter__, __lookupGetter__ and __lookupSetter__ therefore participate. Compare numeric, string, null, undefined and object values, resulting own keys, getters/setters and later change events.
- Test __proto__ with primitive, null and object inputs. Preserve its setter behavior, rawOptions prototype versus spread-copied options prototype, inherited enumerable names, shadowing and order. A successful no-op for all prototype cases is not acceptance.
- Test constructor objects with inherited ordinary option keys and shadowed prototype names. Adapter enumeration must use for-in order, not Object.entries. Retain input object descriptors and post-construction descriptors in the JS/core/binding oracle.
- Rust representation changes must be explicit and tested at the browser binding. Do not describe representable string-key inputs as impossible merely because Rust lacks object prototypes.

## Evidence gate

- Execute all ten original cases without changing their assertions, including the duplicated specific-option case. Count every original assertion and report failed/skipped/unrun cases.
- Port those same ten cases and retain the extra Rust edge cases as separate tests.
- Differential traces cover every key with null/undefined/bool/number/string/object values, exact errors, resulting value kind/content and cumulative events. Retain the seed, operation count, source/input/output hashes and any failing operation.
- Extend oracle checks to every default value and to constructor, disposal, reentrant updates and object identity. A setter-only randomized trace does not establish these other paths.
- Run rustfmt, warning-free Clippy and Rust tests on changed files and the integrated workspace. Independent review must inspect source branches and reproduce commands. Native/browser integration evidence is separate and must not be inferred from this component's tests.
