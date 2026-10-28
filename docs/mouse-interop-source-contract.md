# Raw mouse interop source contract

Task234 supplies source-only observations for implementation task171. It does not
implement a Rust facade and does not prove raw JS/Rust parity. The original eight
MouseStateService tests remain tracked separately. This probe reports zero
original suites and zero Rust cases executed.

Run with source_repo's existing locked esbuild dependency:

```sh
node scripts/build-slot.mjs --owner mouse-interop-source --lane native -- node scripts/mouse-interop-oracle.mjs
```

The shared loader `scripts/mouse-reference-module.mjs` bundles unchanged
MouseStateService.ts and its actual dependencies in memory, hashes every input
and generated module, evaluates it in a fresh VM realm, and stops esbuild in a
finally block. It is also used by the raw4200-button oracle so reference loading
is not copied. No reference source or assertion is rewritten.

Each attempt is retained under `results/mouse-interop/runs/<id>/summary.json`;
`latest.json` points to its status. Setup and assertion failures produce a failed
receipt and nonzero exit, not a reused successful artifact. UTF-16 strings are
recorded as code units so lone surrogates survive JSON transport.

## Nine source check groups

1. Wheel calls receive the original event object and actual MouseStateService
   receiver. Active protocol/encoding getters are live across calls. Adding an
   extra property retains identity rather than serializing a new object.
2. Of fifteen callback result classes, only the literal boolean false vetoes.
   Undefined, null, zero, negative zero, NaN, empty string, symbol, bigint zero,
   objects, arrays and boxed false all allow. Do not use ordinary truthiness.
3. Wheel's direct method call ignores a poisoned Function.call property. It must
   not read that property at all.
4. The Event callback control case deliberately DOES read Function.call, matching
   Event.ts's `listener.fn.call`. The thrown getter sentinel survives, listener
   body does not run, and the selected protocol was committed before dispatch.
   Do not globally replace all callback invocations with one convention.
5. Eight thrown-value classes preserve original identity. Throwing false is an
   exception, not a wheel veto. Throwing undefined is not successful absence.
6. A restriction's receiver is its protocol object. An encoder's receiver is the
   encoding registry, not MouseStateService, Terminal or undefined. Both see the
   same raw event and extra property. A custom encoder can return a lone surrogate.
   External protocol event-mask edits affect the next active-state read.
7. All twelve inherited Object.prototype names are looked up by source registries.
   They do not follow the ordinary unknown-name branch. Setter success can emit
   an undefined event mask, while later restriction/encoding may throw or return
   a non-string value. Missing and lone-surrogate names retain exact error text.
8. Numeric keys use Object.keys ordering, not insertion order alone. Observed
   prefix is0,2,10,4294967294. The4294967295 and01 keys follow ordinary string-key
   insertion order after builtins.
9. Registering __proto__ changes the registry prototype and does not create an
   own enumerable key. Protocol and encoder behavior remains live through that
   prototype, including the encoder's actual registry receiver.

## Receiver shape

The receipt inventories exact own keys before hook registration, own keys at the
first actual wheel callback, constructor name, and prototype method/getter/setter
descriptors. A live receiver is not an empty object with a convenient this value.
Its source methods include addProtocol/addEncoding, protocol and encoding
getters/setters, activity flags, reset, wheel hook control, restriction and
encoding. Lifecycle is inherited from Disposable. A full raw adapter must preserve
its observable state and behavior, not merely expose a matching constructor name.

The source uses plain objects for registries. In particular constructor encoding
can return the event object, valueOf can return the registry, and __proto__ can be
noncallable. These outcomes cannot be claimed equivalent to a typed Rust Vec<u16>
return without an explicit compatible value representation and caller handling.

## Remaining implementation and proof

Task57 covers ordinary typed/shared Rust behavior. Task171 retains inherited
lookups, raw identities/extra properties/returns, strict-false result classes,
receiver behavior and original exceptions. Task172 covers live event/host reads;
Root input bindings must use the real receiver and source invocation convention.
The nine successful source groups are expected behavior, not a Rust pass.

An independent reviewer must inspect the loader, assertions and receipt, rerun
this probe under a controlled slot, and compare future Rust/WASM observations
against the exact retained data. Real Chromium/Firefox/WebKit coverage remains
required. Full terminal equivalence is NOT PROVEN.
