# Late event registration collection gap

This is a static review finding against current `crates/xterm-core/src/event.rs`
and unchanged `source_repo/src/common/Event.ts`. It is not a claim of a newly
executed runtime test. Resource controls currently prevent a build attempt.

In the source event registration closure, `_disposed` is checked before the
optional `disposables` array/store branch. A disposed emitter returns a fresh
no-op disposable without adding it to the supplied collection.

Current Rust `subscribe_in` always calls `store.add(self.subscribe(listener))`.
Current `subscribe_in_vec` always pushes the returned handle. A late emitter
subscription returns a NoopDisposable, but these adapters still collect it.
Thus an observable vector length differs even though both callbacks are silent.
This also affects mapped/filtered events if registration metadata is lost.

## Required regression cases

1. Dispose an emitter, subscribe through `subscribe_in_vec`, and check that the
   vector stays empty. The returned handle must still support idempotent dispose.
2. Repeat through a mapped event. The vector stays empty and the mapper never
   runs, including after a fire on the disposed emitter.
3. Repeat with a live DisposableStore. Its membership must not change. This is
   distinct from explicitly calling `store.add(event.subscribe(...))`, which is
   user-requested collection and may add the no-op handle.
4. Create EventUtils.any over disposed source events. Supplying an outer vector
   must still add exactly one aggregate store. Source any always creates and
   registers its own store, even if every inner source registration is a no-op.
5. Repeat any with zero source events, and map over any. The aggregate still
   registers once. Do not suppress this by classifying every empty group as a
   no-op registration.
6. Repeat any over mapped disposed emitters. Its inner no-op handles and outer
   group have different collection behavior from direct/mapped emitter calls.

The implementation needs to preserve whether source registration itself reached
its collection branch. Merely checking the final handle's apparent no-op nature
would conflate direct late subscriptions with intentionally collected aggregates.
Keep current public handle APIs and dispatch context behavior intact. Captures
must continue to drop outside state borrows.

Task 187 is open and owns the relevant Event/context adaptation. This finding
should be explicitly covered there or by a linked followup before criteria
221/223/230 are declared complete. The old 70-test result did not exercise these
specific late collection observations.
