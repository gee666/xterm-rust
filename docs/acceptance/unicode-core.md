# Unicode core acceptance

These checks cover `UnicodeV6.ts`, Unicode property helpers/types in `Services.ts` and `UnicodeService.ts`, `UnicodeService.ts`, and `common/public/UnicodeApi.ts`. Unicode11 and grapheme addons have separate acceptance owned by Nuwa. Terminal proposed-API gating is owned by the public terminal tasks, not silently treated as passing here.

Each row is required. A mapped test must execute Rust logic. A JavaScript baseline, fixture generator, compiled API or broad covered vote alone is not a pass. Unwritten cases remain unrun.

## U6. Width classification

1. `version()` is exactly `6`. Construction and the unit/default provider initialize the same immutable BMP widths safely, including concurrent first use.
2. For every integer codepoint 0 through 0xFFFF, Rust widths equal the original implementation and the old implementation exercised in unchanged `UnicodeV6.test.ts`. Preserve all 65,536 assertions. NUL/C0/C1 controls occupy zero columns; printable ASCII occupies one.
3. For every integer 0 through 0x10FFFF, including surrogate codepoints, widths equal the actual JS provider. Never substitute a current system Unicode library or normalize surrogates away.
4. BMP wide ranges preserve half-open table fill boundaries. U+303F stays narrow. Combining ranges override initially wide classification, specifically U+302A..U+302F and U+3099..U+309A.
5. Supplementary combining ranges are zero-width at both inclusive endpoints and narrow immediately outside unless a wide range applies. U+20000..U+2FFFD and U+30000..U+3FFFD are wide; U+2FFFE/U+2FFFF/U+3FFFE/U+3FFFF are narrow.
6. Representable integer codepoints beyond 0x10FFFF remain narrow as upstream; include 0x110000 and u32::MAX. Negative/fractional/non-finite JS values cannot enter the u32 Rust provider contract and must be documented as typed-boundary differences, not normalized silently.

Evidence: `scripts/unicode-v6-oracle.mjs` executes the unchanged original assertion body and records source hashes. `tests/unicode_v6.rs::{wcwidth_should_match_all_values_from_the_old_implementation,supplementary_and_surrogate_codepoints_match_javascript,lazy_table_initialization_is_shared_and_thread_safe}`. Keep fixture coverage contiguous with no omitted interval.

## UP. Packed properties and joining

1. Creating a property masks state to 24 bits and width to two bits, then packs state in bits 3+, width in bits 1..2, and should-join in bit 0. State overflow, widths 0..255 and both join flags are checked, not just normal 0/1/2 widths.
2. Raw values retain all 32 bits, including reserved width 3. Extraction preserves signed arithmetic-right-shift semantics for char kind, including raw 0xFFFFFFFF and 0x80000000.
3. A nonzero-width codepoint never joins regardless of preceding state. A zero-width codepoint with raw preceding zero does not join. Nonzero preceding with packed width zero also does not join. Otherwise zero-width characters join and retain the preceding width, including reserved width 3.
4. Returned V6 state is zero. Joining does not preserve the preceding char kind. Multiple combining characters preserve the retained width and join flag consistently.
5. Shared V6/V11 helpers may remove duplicated logic only if all V6 oracle fixtures remain unchanged and the independent reviewer reruns them.

Evidence: `tests/unicode_v6.rs::{packed_properties_match_javascript_for_all_width_states,property_construction_masks_state_and_width_like_javascript}` plus service joining tests. Oracle property fixtures include all preceding widths, zero and nonzero state, high-bit raw values and zero/narrow/wide codepoints.

## UR. Registry and selection

1. A new service has no providers, empty active version and empty versions list. Width/property calls before registration return an explicit no-active-provider error. Empty-string width is zero even without a provider, matching the source's empty loop.
2. Registering the first provider makes its version active and fires change. Registering another name only extends the registry. Duplicate version registration replaces that registry entry without changing key order or the already-selected provider object.
3. Re-selecting a replaced version activates its new provider object. Setting the same version twice fires twice. An unknown name returns the exact unknown-version message and changes neither active version nor provider, and emits no event.
4. Versions match ECMAScript `Object.keys`: canonical array-index names 0 through 4294967294 sort ascending before other names; named keys retain insertion order. Include `0`, `2`, `6`, `11`, `01`, `-0`, `4294967294`, `4294967295`, empty string, `__proto__` and `constructor`. No inherited-prototype names leak into the registry.
5. Empty version names remain valid. Because upstream uses the empty active string as a sentinel, a later registration after an empty-name activation auto-selects again. Do not replace this behavior with a different first-registration flag.
6. Provider `Rc` identity is retained; no copying of provider state. The active provider snapshot survives replacement of the registry entry until explicit selection.

Required tests in `tests/unicode_service.rs`: `empty_service`, `default_to_v6`, `unknown_version_is_transactional`, `registry_replacement_retains_active_snapshot`, `version_keys_match_object_keys`, `empty_name_registration_follows_source` and baseline service cases. JS operation fixtures must cover duplicates and ordering, not only static V6 widths.

## UW. UTF-16 width accumulation

1. Preserve the five original service tests: V6 default/version list/selection/hello width, unknown version, change notification, dummy provider width changes, and ten U+1F923 emoji width 10 under V6.
2. Valid high/low surrogate pairs combine into one supplementary codepoint. BMP, supplementary, NUL, controls, narrow/wide and combining sequences match JS across arbitrary strings.
3. A trailing high surrogate adds its standalone `wcwidth` and returns immediately, without calling `charProperties` for it. A lone low surrogate calls `charProperties` normally.
4. A high surrogate followed by a non-low unit consumes both. It adds the second unit's standalone `wcwidth` before calculating the first unit's properties; it does not retry the second unit for joining or pair it with a later low surrogate. Cover high+ASCII, high+combining, high+high+low and high+wide.
5. Each joined character contributes current width minus preceding packed width. Nonjoining characters contribute the current width. The preceding property is updated after the current character, not after the malformed pair's second-unit standalone width.
6. Custom provider observations verify codepoint call order, property state propagation, zero-width chains, width shrinkage and provider switching during callbacks. Accumulation is signed because a provider can reduce a joined width. Recoverable numeric overflow must be explicit rather than panic or wrapping.
7. `str` and explicit UTF-16 overloads agree for all valid strings. The UTF-16 overload preserves lone surrogates; do not replace them with U+FFFD. Neither overload allocates a temporary full UTF-16 copy.
8. Differential corpus records seed and iteration count and retains the exact failing UTF-16 units. It includes deterministic malformed boundaries plus seeded random u16 sequences under V6 and a stateful custom provider. Compare results and provider call logs, not widths alone.

Required tests: original five cases in `tests/unicode_service.rs`; `custom_provider_call_order_matches_source`, `joined_width_can_decrease`, `provider_callbacks_can_switch_active_version_without_borrow_panics`, and `seeded_utf16_traces_match_javascript` in `tests/unicode_service_differential.rs`. The seeded test also checks the ten deterministic boundary inputs before its 4,096 random inputs. `registry_operation_traces_match_javascript` compares 531 duplicate/select/order/disposal operations. Fixtures and source hashes live under tracked results/fixtures, temporary module copies under root tmp/.

## UE. Events, disposal and reentrancy

1. Selection commits active version/provider before firing listeners. Listeners observe committed state. Reentrant listeners can register or select another version without RefCell panic or a borrow held across callbacks.
2. Listener order, additions/removals, explicit subscription disposal and errors use the shared Event implementation. A propagated event-dispatch failure does not roll back committed selection or newly registered provider.
3. Disposal clears change listeners but retains registry, active version/provider and functional width calculation. After disposal, selection/registration remains possible without events; repeated disposal is harmless.
4. Capturing a weak service reference in a listener supports reentrant selection without leaked reference cycles. No unsafe code or locks across callbacks are introduced.

Required tests: `selection_commits_before_reentrant_events`, `event_failure_keeps_committed_state`, `dispose_only_clears_notifications`, and subscription-disposal checks.

## UA. Public view

1. A UnicodeApi view forwards registration, versions, active getter and active setter to the same live service. Multiple views see each other's changes; no detached registry copy is introduced.
2. Unknown-version and event errors propagate unchanged. The view's lifetime cannot exceed its service. Registering a provider through the view preserves exact identity/replacement behavior.
3. Public terminal and WASM access must separately enforce upstream proposed-API gating. A passing service test is not evidence that public terminal gating or browser bindings work.

Required tests: `unicode_api_views_share_service` and public terminal/browser gating tests owned by tasks 30/31.

## Commands and accounting

```
node scripts/unicode-v6-oracle.mjs
cargo test -p xterm-core --test unicode_v6
node scripts/unicode-service-oracle.mjs
cargo test -p xterm-core --test unicode_service --test unicode_service_differential
cargo test -p xterm-core --lib unicode_service::tests
cargo clippy -p xterm-core --test unicode_v6 --test unicode_service --test unicode_service_differential -- -D warnings
```

V6 has executed evidence in `results/unicode-v6/oracle.json`, five passing Rust tests and two independent reviews. Service execution now includes five original baseline cases, 15 directed Rust tests and five differential tests in the post-Event debug rerun. One accumulation-boundary unit test and the earlier 14 plus five release tests passed before that shared migration; updated private-unit/release/Clippy logs retain actual current status. Logs under `results/unicode-service/` retain prior shared-compilation failures separately from the passing scoped rerun. See `docs/unicode-service.md` for the current evidence mapping and limits. Native Linux/macOS, full terminal integration and browser bindings remain separate gates.
