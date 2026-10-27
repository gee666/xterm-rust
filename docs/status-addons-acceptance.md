# Program status and progress acceptance

These requirements cover task 43. Pure parsing and a fixture terminal are not proof of browser or live terminal equivalence. Each case must map to an executable assertion and actual command results.

## OSC 7501 parsing

Port every assertion in addon-program-status/src/ProgramStatus.test.ts unchanged in meaning. Parse root and hierarchical ids, state, blocked-only kind, working/blocked-only progress, app, title and msg. Preserve last valid repeated values, ASCII-only whitespace trimming, unknown key handling and pairs without equals. An overlong unknown key rejects the entire report. Reject missing/invalid state and malformed id even on clear. Empty id in a pair is invalid, absent id selects root. Key is lowercase ASCII. Allowed value grammar is ASCII alphanumeric plus `_.,+/=-`.

Exercise exact UTF-8 boundaries at payload 4087, key 16, encoded message 2732, decoded message 2048, encoded title 256, decoded title 192, app/segment 32 and id 128 bytes. Id permits at most eight nonempty segments. Test one below, equal, one above each limit including multibyte and lone UTF-16 surrogates. Progress is digits only, allows leading zeros, 0 through 100, and invalid optional progress is omitted. Base64 accepts optional canonical padding, rejects excess/embedded padding, residual bits, URL-safe characters, invalid UTF-8, C0/C1/DEL, and preserves BOM, bidi and zero-width characters. Decoding errors reject the full report, not only one field.

## Ordered store and snapshot

Whole-record replacement removes omitted fields and moves that id to newest update. At 256 entries, updating an existing id evicts nothing; inserting a new id evicts the oldest update. Clear exact id and descendants but not textual prefix siblings. Empty clear removes all. processExited and promptStarted remove only working/blocked. Effective app comes from the nearest present ancestor including root and changes when ancestors change. No ancestor means absence. Reuse the identical immutable snapshot until a real modification, including after no-op clear. Old snapshots must remain unchanged after mutations.

## OSC 7501 activation and lifecycle

On activation register OSC 7501 and unprefixed ESC c. Query only exact `?`; reply synchronously as ESC ] 7501 ; ? ESC backslash regardless of BEL/ST terminator and before following DA1. Respect disableStdin through terminal.input with user-input false. Invalid reports are consumed without events. Replacement always fires one full snapshot event. Clear/no-op exits fire only on real changes. RIS clears and returns false so core still resets buffer and cursor; soft reset, alternate screen and API reset retain records. Dispose clears silently, unregisters both handlers and emitter; later reports/queries do nothing. Verify callback reentrancy and no borrow held across listeners.

## OSC 9 progress

Port all ten ProgressAddon browser assertions. Only `4;` claims OSC9. More than three fields is consumed without event. Missing value defaults zero. Decimal parsing rejects signs, spaces and non-digits, handles huge values like JS numbers and empty strings as zero. States 0 remove and 3 indeterminate ignore malformed value; 1 set uses zero when absent; 2 error and 4 pause preserve previous value for zero/empty/absent and otherwise update it. Clamp values to [0,100], preserve fractional API setter values, NaN/infinities as JS does. Reject state values outside [0,4] without mutation. Every accepted setter call fires even if unchanged; preactivation setter has no event. Dispose removes handler and listener emitter without resetting progress. Activation must expose onChange before use.

## Evidence and integration

Run the unchanged original pure unit suite and compare deterministic seeded malformed/valid payloads against source TypeScript. Retain inputs, seed and mismatch cases. Record exact discovered, executed and skipped test counts and command logs. Run native registration through actual Rust terminal parser, including split chunks, BEL/ST, abort, query ordering and RIS. Run the Rust-backed WASM browser addon tests in Chromium, Firefox and WebKit through loadAddon and dispose. Fixture-only tests must be labelled fixture tests. Missing native terminal/browser execution keeps integration incomplete. Run fmt, Clippy with -D warnings and all relevant addon tests; independently review actual code and results.
