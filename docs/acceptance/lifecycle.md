# Lifecycle acceptance contract

Scope is every export and observable branch in `source_repo/src/common/Lifecycle.ts`. Upstream has no Lifecycle.test.ts. This specification supplements the aggregate terminal requirements; passing it does not prove terminal equivalence.

## LC1. Helpers and object identity

- `to_disposable` calls its callback on every explicit disposal, not just the first. Creating or dropping a bare callback handle must not invoke it.
- `dispose` returns the identical resource after calling it. `dispose_optional(None)` returns None without a callback; Some returns the identical resource.
- `dispose_all` invokes every entry in vector order, including duplicates, and returns an empty vector. Empty input succeeds.
- `combined_disposable` keeps the order and duplicates, repeats the complete sequence on repeated disposal, and accepts an empty group.
- `NONE` is intentionally inert, matching Disposable.None. It is not an unsupported-operation placeholder.
- Rust overload names, Rc identity and consumption of the vector overload are documented. No claim is made that Rust can return an alias of a consumed JavaScript array.

Evidence: `callback_disposal_repeats_and_helpers_preserve_identity`, `combined_preserves_order_duplicates_and_repeated_calls`, and the shared fixture checked by both native Rust and unchanged TypeScript.

## LC2. Store transitions and live Set traversal

- New store is open. Adding returns the exact handle; repeated addition of the same object is deduplicated, but separate objects with equal behavior are not.
- Clear disposes in registration order and empties the store without closing it. Empty clear is inert. Readding after clear disposes on the next traversal.
- Dispose closes before invoking any callback. Repeated dispose is inert. Registration after closure invokes immediately every time and is never retained.
- Callback registration during clear participates in that traversal. Nested dispose of the same closed store does not recurse.
- A callback that performs a guarded nested clear and then adds another resource must match the JavaScript Set iterator. The retained counterexample emits `nested-a,nested-a,nested-b`, not two callbacks. Tombstones must not shift the outer iterator cursor.
- No interior borrow may span a callback or destruction of a removed resource.

Evidence: `stores_deduplicate_identity_and_clear_allows_reuse`, `registration_during_clear_is_visited_in_the_same_pass`, `dispose_closes_store_before_callbacks_and_late_registration_is_immediate`, `same_trace_as_typescript_oracle`.

## LC3. Mutable replacement and disposal visibility

- New value is absent. Assigning the identical handle or assigning None to None does nothing.
- Replacing invokes the old resource before storing the new one. An old-resource callback can still read the old value.
- Clear is assignment to None, disposes once, and can repeat without another callback.
- Dispose hides the value before its callback, then clears storage. Repeated successful dispose is inert.
- Assignment after disposal is ignored, and does not dispose the rejected value.
- Callback-visible state and ordering must match unchanged TypeScript, not just final callback counts.

Evidence: `mutable_replacement_uses_identity_and_disposes_before_assignment`, `mutable_disposal_hides_value_and_ignores_future_assignment` and shared fixture.

## LC4. Callback failures

User callback panics are Rust's native unwinding counterpart for synchronous JavaScript callback throws. This contract does not turn untrusted terminal bytes into panics.

- Helpers and combined groups propagate failure and do not invoke later entries. A later explicit call repeats the group from its beginning.
- Clear failure leaves the store open, preserves undisposed entries, and permits a later explicit retry.
- Dispose failure leaves the store closed; another dispose must not retry its retained callbacks. A later registration still invokes immediately.
- Mutable replacement failure leaves the old value in storage. A later successful replacement can retry its disposal.
- Mutable dispose failure leaves its getter hidden. A later explicit dispose retries the still-stored resource, matching upstream's assignment ordering.
- Tests must catch only the deliberately injected panic, explicitly finish cleanup, and compare corresponding TypeScript throw traces. Do not suppress real failures or claim that panicking during Rust Drop is supported.

Evidence is `callback_failures_match_unchanged_typescript_state_and_order` and `node scripts/check-lifecycle-failures.mjs`, comparing the shared 18-callback fixture with six injected failures. These were missing from the first nine-test implementation and were added by task #78.

## LC5. Rust ownership and review

- Default DisposableStore, MutableDisposable and embedded DisposableOwner clean up on drop exactly once after successful explicit cleanup. DisposableStore::new_explicit releases stored handles without invoking their disposal callbacks at drop; explicit disposal still deduplicates and is idempotent. EventUtils.any uses this mode so an ignored returned handle does not unsubscribe. Test both policies and capture release.
- Register forwards object identity; no browser or platform dependency or unsafe operation is introduced.
- Callbacks must not form Rc cycles with owners. Tests use Weak owner references for reentrancy. Drop callbacks must not panic.
- Document composition in place of inheritance and deterministic drop cleanup as deliberate language mappings. Do not represent JavaScript GC as deterministic disposal.
- Native tests, unchanged-source TypeScript checks, rustfmt and warning-free Clippy must pass. An independent reviewer inspects the actual implementation and retained nested-clear counterexample.

Evidence: `rust_owners_clean_up_at_drop`, `docs/lifecycle.md`, `results/lifecycle/`, task #5 independent reviews. Linux and macOS runs remain separate project gates.
