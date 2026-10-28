# Dynamic identifier cases for task 214

The authoritative order is `InputHandler.registerCsiHandler` at source line 678 followed by `EscapeSequenceParser._identifier`. Do not derive the security decision from the final packed key.

## CSI guard decision must precede packing

Use getters whose return values change on each read, log all get operations, register through the actual public parser and then write the matching sequence.

1. `final` returns `m`, then `t`, then `t`; prefix/intermediates always return undefined. Initial classification is not guarded, but packing produces bare `t`. Writing CSI 8 t must invoke the custom callback even when window options disallow that operation. Expected field reads are final, prefix, intermediates, final, final.
2. `final` always returns `t`; prefix returns undefined for the initial guard, then `?` for all packing reads; intermediates returns undefined. Initial classification is guarded, but packing produces private `?t`. Writing CSI ? 8 t must remain suppressed when window options disallow it. The extra initial prefix/intermediate reads are observable.
3. `final` always returns `t`; prefix is `?` for the initial guard, then undefined during packing. Initial classification is not guarded, but packing produces bare `t`. The initial guard short-circuits before reading intermediates. Writing CSI 8 t must invoke the callback.
4. An initial final getter throws a sentinel while prefix would fail validation. The sentinel wins before prefix is read. Preserve its object identity.

## Repeated reads within the packer

- Prefix is read for truthiness, read again for `.length`, and read again for `.charCodeAt(0)`. A truthy first value followed by an empty string can produce NaN during the last read. Source bitwise conversion, not a static snapshot's validation result, controls the key.
- Intermediates are read for truthiness, length validation, every loop condition, and each character read. Example values `" "`, `"  "`, `""` cause zero loop iterations despite a truthy initial value. A snapshot would register a different command.
- Final is read once for `.length` and again for `.charCodeAt(0)`, in addition to any CSI classification read. Error precedence must follow those actual reads.
- Test getter callbacks that register or dispose unrelated handlers. Cached weak registrars must allow this without reborrowing the engine. The new registration must obey the current versus next reverse-loop behavior.

## APC mutation is observable

APC writes `id.prefix = undefined` before `_identifier` reads the object. Test a Proxy set trap, a readonly/frozen property that throws, a setter with its own sentinel failure, and a setter that reports success without clearing its getter's `?` value. The latter can register the packed `?X` key even though ordinary APC X input cannot invoke it. A binding that unconditionally drops prefix after taking a snapshot would incorrectly invoke that handler.

## Native route

`CommandRegistrar::register_packed` and `clear_packed` accept the already-computed bits. They do not read an identifier object or repeat conversion. Public CSI packing must additionally carry the earlier guard decision to its facade. DCS/APC numeric registrars already accept packed identifiers. Reuse callback converters and ordering logic rather than introducing a second dispatch engine.

These are proposed adversarial source cases, not recorded passes. Capture the unchanged JS access logs, packed keys, errors and actual dispatch; then execute the same cases against the Rust-backed WASM API. Ordinary static strings still need every range, UTF-16 length and exact validation-message assertion.
