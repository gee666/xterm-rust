# OptionsService verification

Full terminal and browser equivalence is NOT PROVEN. This report covers ordinary OptionsService keys and the lossless Rust boundary types. Constructor prototype interactions remain assigned work in task 109, not an external blocker.

## Implementation and representation

`crates/xterm-core/src/options/` contains all 48 defaults, sanitization, diagnostics, raw writes, event subscriptions and external-reference cleanup. It uses shared `Event`/`Lifecycle` implementations, not another event queue.

`OptionValue` retains JS undefined/null, booleans, binary64 numbers and UTF-16 strings. Object clones preserve Rc identity. `OptionObject::new` takes a fallible ToPrimitive callback with Number/String hints. Invalid object conversion returns a recoverable error rather than silently becoming NaN. `OptionError::message_utf16()` retains unpaired surrogates. Display necessarily substitutes invalid UTF-16 because Rust's formatter accepts UTF-8 only.

`OptionObject::with_properties` forwards dynamic host property access and exceptions. `OptionValue::property_map` provides a plain structured bag. The WindowsPty payload exposes backend and buildNumber and retains a typed payload for buffer consumers. No JS engine runs core validation logic.

The native constructor returns diagnostics rather than writing console.error. It accepts declared options and standard Object.prototype data names. The browser constructor uses actual JS for-in membership, assignment and prototype mechanics, with exactly one Rust sanitization pass. It writes source-equivalent console diagnostics and attaches RawOptionsStore to the resulting object. Task109's directed browser corpus remains unexecuted, so this implementation is not yet proof of those paths.

Fallible option listeners preserve the original error, commit the option first, and stop traversal. The shared Event implementation retains callback errors through EmitError::Callback. OptionsService recovers typed OptionError without a second event queue or dispatch stack. Nested/caught failures retain their original error. Rust unwinding is not caught or hidden. LogService uses this path for live property-key conversion. This replaced an interim local dispatch-error stack after Event gained typed callback errors.

## Observed commands and counts

Environment: native Windows x64, Node v24.15.0, rustc 1.85.1. Shared `.cargo/config.toml` profiles were retained. The final scoped run observed 2.0 GiB free on P: after checks. It started no services or alternate target directories.

- Initial `cargo test -p xterm-core --test options_service --test options_boundary`: 28 passed, 0 failed, 0 ignored. Passing post-Event receipt: `results/options/tests-after-event.log`. Some earlier attempted `tests.log` content is a retained integration failure, not the passing receipt.
- `node scripts/check-options.mjs`: all original 10 cases and 22 assertions passed. The duplicate original specific-option case remains two separate Rust tests. Log: `results/options/oracle.log`.
- Same oracle: 48 ordered default comparisons, 3,456 constructor cases, 28,456 setters, and one multi-operation recursive/disposal trace. It compared 31,961 output lines with zero mismatches. Seed `1330664531`, source/input/output SHA-256 hashes and exact counts are in `results/options/summary.json`.
- `cargo clippy -p xterm-core --lib --example options_oracle --test options_service --test options_boundary -- -D warnings`: passed. Log: `results/options/clippy.log`.
- `rustfmt --edition 2021 --check` with the four options modules, both options tests, options_oracle.rs and options_oracle/scenarios.rs: passed. Log: `results/options/fmt.log`.

These initial receipts predate the latest host-storage/operator extensions. They are component checks, not current full-workspace all-feature quality, native Linux/macOS execution or browser proof. Team quality and integration tasks own those gates.

## Host extension checks

The queued native command `cargo test -p xterm-core --test options_service --test options_boundary --test options_host_store` passed36 tests on 2026-10-28. Exact command, disk observations, exit status and suite counts are in `results/build-slots/1793150119941-4020-8697885b-7d98-4332-a1d3-545e416d9b8c/`. An additional opaque-NaN equality regression brings the current expected count to37, but that last case has not run yet.

The first browser attempt did not reach tests. Full xterm-web compilation failed in unfinished Viewport/renderer/utility integration, not in the owned options module. Retained receipts are `results/build-slots/1793150119944-49196-2042010b-c56b-4ca3-b347-ad0549f5e55f/` and `results/browser-host/wasm_options/2026-10-28T01-15-20-000Z/`. No browser test is counted as executed or passed.

Task225 proposes a single canonical DOM-free WASM support crate with old web paths reexported. It does not waive actual Terminal integration tests. Pending directed cases compare original OptionsService with Rust for inherited/unknown/prototype keys, descriptor flags, dynamic events, Proxy throws, nested live bags, shared defaults, JS operators, cleanup order, reentrancy, readonly assignments and partial batch assignment. The generated test-only reference and source hashes are described by `results/options/browser-reference.json`.

Builds now require a lane in `docs/build-slots.md`, use the shared target/profile, and stop below1GiB. Do not bypass the guard. Resource failures and code that has not run are not passing evidence.

The oracle loads the original Lifecycle.ts and evaluates original Event.ts/OptionsService.ts after removing import/export plumbing and stripping TypeScript. The original test file retains every assertion; only imports and runner hooks change. Node reports its experimental type-strip/module-detection warnings. The source reference was not edited to suppress them.

## Original-case mapping

| Original test family | Original cases | Rust test names |
| --- | ---: | --- |
| Constructor undefined dimensions | 1 | constructor_undefined_dimensions_use_defaults |
| Constructor explicit dimensions | 1 | constructor_valid_dimensions |
| Constructor invalid option fallback | 1 | constructor_invalid_value_uses_default |
| Enumerable option keys | 1 | object_keys_are_nonempty |
| Valid font weights | 1 | valid_font_weights |
| Invalid font weights normalize | 1 | invalid_font_weights_normalize |
| General change events | 1 | on_option_change |
| Specific change events, repeated upstream | 2 | specific_option_change_first_upstream_case, specific_option_change_duplicate_upstream_case |
| Multiple-key change event | 1 | multiple_option_change |

Additional tests cover coercion/NaN/bounds, exact default sharing, lone surrogates, once-rounded large radix integers, constructor diagnostic ordering, raw writes, reference cleanup, hints and thrown conversion errors, typed/dynamic properties, fallible nested callbacks, unwinding and live listener mutation. The seeded oracle includes every known key against primitive/object/custom-coercion values and 5,000 long binary/octal/hex strings.

## Findings and retained regressions

- The first differential run found cursorStyle error text replaced a lone surrogate with U+FFFD. `results/options/counterexample.json` retains that historical failure. It is fixed by UTF-16 error storage and the `invalid_utf16_error_is_lossless` test. A retained file is not a claim that the latest run failed.
- Independent review by Aaliyah found missing ToPrimitive hints and swallowed object-conversion errors. These were fixed with explicit hints and Result propagation, then exercised in both Rust and JS/Rust traces.
- Review found constructor Object.prototype membership differs from the initial fixed-key implementation. Task109 implements host-backed construction and has directed browser assertions, but those assertions have not run. The acceptance criteria remain incomplete until independent verification.
- Options/Terminal cross-GC ownership cycles remain explicit task208/234 work in `docs/wasm-ownership.md`. Do not claim explicit disposal alone proves GC equivalence.
- Several intermediate builds collided with other agents' active parser/serializer/buffer API migrations. The final commands above passed against the integrated files observed at that run; they do not erase unrelated outstanding suite failures.
