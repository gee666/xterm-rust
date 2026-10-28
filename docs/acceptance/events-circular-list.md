# Event and CircularList acceptance matrix

Reference files are `source_repo/src/common/{Event,CircularList}.ts` and their unchanged `.test.ts` files. These are internal common APIs, not a substitute for public terminal or browser coverage. Each case below is required. A green random corpus does not replace the named edge cases.

## Event registration and ownership, E1

- E1.01: Firing with zero listeners returns successfully. One listener receives the exact payload. Multiple listeners run synchronously in registration order.
- E1.02: Repeated access to `event` returns the same registration identity. A retained event keeps registration state alive after the original Rust emitter owner goes out of scope.
- E1.03: Registering the same callback twice creates independent entries. Disposing one does not remove the other. Repeated handle disposal does not unregister a different entry.
- E1.04: Capture the receiver instead of `thisArgs`; state changes target that same receiver. No deep copy of an event payload or receiver is introduced.
- E1.05: Discarding a subscription handle does not remove the listener. Keeping a handle after emitter destruction does not leak a listener or panic on disposal.
- E1.06: A handle added to a disposable vector or store is the exact returned object. Disposing either container unregisters it. Registering into an already disposed store unregisters immediately.
- E1.07: Emitter disposal is idempotent. It clears listeners, prevents future firing, and makes subsequent registration return a safe, repeatably disposable empty subscription.

## Event traversal and errors, E2

- E2.01: Listener additions do not join the active dispatch. The next fire includes them.
- E2.02: Removing the current or a later listener does not remove it from an already captured dispatch array. The next fire reflects removal.
- E2.03: Recursive fire sees the current listener array and completes before the outer callback resumes. No RefCell borrow or lock survives entry into user code.
- E2.04: Disposing the emitter during a one-listener dispatch completes successfully.
- E2.05: Disposing during a multi-listener dispatch clears the active array and stops traversal with the upstream TypeError represented by `EmitError`. The error occurs only when there is a subsequent entry to read.
- E2.06: Adding/removing a listener before disposal replaces the array. Disposal clears only the current array, not a separately captured outer array. Verify both paths, including nested fire.
- E2.07: Fallible callbacks propagate errors synchronously and stop later callbacks. A user panic propagates rather than being swallowed or converted into success. Forwarded errors stop the source traversal too.

## Event utilities, E3

- E3.01: `forward` preserves payload identity, synchronous order and disposal. A failed target fire propagates to the source fire.
- E3.02: `map` runs once per source event, preserves ordering, forwards mapped values, and stops mapping after subscription disposal. Separate subscribers each invoke their mapper as upstream does.
- E3.03: `any` accepts zero, one and several events. Registration order matters when sources are identical. The combined handle unregisters all constituent subscriptions on repeated disposal.
- E3.04: Combining heterogeneous events requires an explicit Rust payload-erasure map. Document that representation; do not drop an upstream overload without a callable replacement.
- E3.05: `runAndSubscribe` calls the handler before registration, with the explicit initial value or undefined represented by `None`. Events fired inside that first callback are not replayed. An initial callback failure must not leave a subscription behind.
- E3.06: Mapping/combining/forwarding must preserve vector/store disposal behavior. Dropping all owners must release captured objects; no strong reference cycle is introduced by event infrastructure.

## Ring storage and identity, L1

- L1.01: Construction starts empty; `length`, `maxLength`, `isFull`, and wrapped `get` agree at every push boundary. Test capacities one, two and larger.
- L1.02: Push below capacity extends length. Push at capacity overwrites the oldest slot, advances the start index, emits exactly one trim and leaves length capped.
- L1.03: `get` and `set` wrap independently of logical length. Undefined slots stay absent. `pop` decrements length but does not clear the stale backing slot.
- L1.04: `recycle` errors before any mutation when not full. When full, it advances first, emits trim, then reads the recycled last element. Shared mutable element identity survives copies/recycling with `Rc` storage.
- L1.05: Cyclic-index arithmetic cannot overflow even for large representable indexes. The maximum source capacity, 4,294,967,295, must not cause eager capacity-sized allocation.

## Capacity and raw length, L2

- L2.01: Setting the same capacity is a true no-op, retaining wrapped/stale backing slots.
- L2.02: Growing/shrinking capacity resets start to zero and copies only the first `min(new capacity, logical length)` cyclic values, including holes. Logical length does not change, even if it now exceeds capacity.
- L2.03: Length shrink does not clear slots. Length growth clears physical indexes between old and new length, not cyclic indexes. Test wrapped starts and a backing array longer than capacity.
- L2.04: Sparse extreme capacities/lengths and high-index writes use bounded storage proportional to stored entries. Recoverable reserve failures produce an explicit error, never a panic or successful no-op.
- L2.05: Resize after logical length exceeds capacity must reproduce duplicate cyclic values when the new capacity exposes them. Test both populated and absent slots.

## Splice and trim, L3

- L3.01: Delete-only operations shift survivors left, decrement length, then emit delete. Inserting without deletion shifts backward and writes in source order.
- L3.02: Replacement deletes before insertion. Insert fires after physical writes but before length adjustment. Overflow advances start, caps length and then emits trim.
- L3.03: Preserve upstream cyclic-overwrite behavior when insertion itself exceeds capacity; do not replace it with a mathematically cleaner Vec splice.
- L3.04: Cover insert/delete at zero/end/middle, empty items, delete zero, exact capacity, multiple wraps and simultaneous delete/insert/trim.
- L3.05: `trimStart` clamps count to current length, advances start and emits even for zero. Trimming more than length leaves an empty list.

## Element shifts, L4

- L4.01: Count zero returns before validating start or offset. Invalid negative/outside start and destinations before zero return the exact upstream error messages through typed errors.
- L4.02: Positive offset copies backward to protect overlapping sources. Negative/zero offset copies forward, does not shorten length and may expose stale wrapped values.
- L4.03: Positive expansion changes length only as needed. Every excess element generates a separate trim-one event after each start/length change, not a combined trim.
- L4.04: Cover one/many entries, overlapping/nonoverlapping moves, expansion below/exact/over capacity, repeated wraps, source count extending beyond logical length and arithmetic overflow.

## Reentrant observers and lifecycle, L5

- L5.01: Delete, insert and trim callbacks may read and mutate the same list. Outer continuation reads state again at the same source statements. No storage borrow spans callback entry.
- L5.02: Capture callback-time length, capacity, slots and event payloads, not only final state. Reentrant push, trim, resize and length changes must have explicit deterministic assertions.
- L5.03: An event error interrupts mutation at the same statement as upstream. Compare retained partial storage, length and subsequent-operation behavior.
- L5.04: List disposal unregisters all three emitter families. Storage operations remain callable afterward without emitting events, matching the source.
- L5.05: Integrate with actual Buffer scrollback/marker observers. Verify recycled BufferLine identity, deletion marker movement, insert/trim ordering and normal/alternate buffer switching, not only integer-list examples.

## Typed boundaries and browser mapping, B1

- B1.01: Document every Rust representation change. Positive capacity, usize length/index and validated splice ranges differ from degenerate JS states such as negative length after empty pop. Tests must assert the rejection without claiming those JS states are equivalent.
- B1.02: The browser binding must retain original callable signatures/errors wherever these internal contracts are observable through terminal behavior. Numeric conversion must reject nonfinite, fractional and out-of-range values before mutation where the original terminal does.
- B1.03: Keep explicit unfinished status for any observable mismatch. A documented difference is not permission to claim full equivalence.

## Evidence requirements, V1

Run `node scripts/check-events-and-circular-list.mjs`, `cargo test -p xterm-core --test event --test circular_list`, and focused Clippy on the library, both tests and both trace examples. Run workspace quality gates after integration. Keep all implementation/test files at most 350 lines or document a cohesive exception.

The baseline has seven Event tests and nineteen CircularList tests containing 95 original assertions. Preserve the assertion bodies and source hashes. Current trace seed is `0xc1ac01a7`; compare every raw/cyclic slot, length, capacity, fullness, result/error, ordered event payload and callback-time snapshot across 20,001 circular states and 10,008 event traces. Retain the shortest available failing operation prefix as a counterexample. Record exact commands, versions, counts and independent reviewer results under `results/events-circular-list/`.

The initial implementation passed those scoped counts but did not yet demonstrate every newly enumerated edge above. In particular, extreme sparse capacity, event getter identity and complete reentrant mutation/error matrices require follow-up assertions. Native Windows utility checks do not prove Linux/macOS, browser/WASM or complete terminal equivalence.
