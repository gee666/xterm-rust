# Buffer cleanup host contract

Criterion 413 and tasks 179/180 close the missing production consumers of BufferCleanup. Task 99 verifies their live terminal integration. The portable factory is implemented by task 236. Native host execution is implemented by task 179; browser execution remains task 180. The factory tests do not substitute for those host checks.

## Required ownership and event flow

- Own one cleanup adapter for each actual normal/alternate Buffer pair, not just the active buffer. Inactive normal scrollback can still need cleanup.
- BufferSet activation provides the actual active/inactive handles. Ordinary activation keeps existing adapters. Do not rely only on a successful final reset event: nested reset or a throwing disposal callback can leave raw Set fields and owner slots different, as documented in buffer_owner.rs. Attach at buffer creation/replacement or prove source-equivalent recovery for those partial states. Cleanup ownership follows the Buffer lifetime, not only the currently visible pair. Do not retain strong old Buffer owners merely to make callbacks succeed.
- Queue requests must result in real native event-pump or browser callbacks. A host that merely exposes poll_cleanup or process_idle to a test is not attached scheduling.
- Native hosts without idle callbacks use the existing priority fallback and its deadline policy. Browser hosts use the real idle deadline where available and priority fallback otherwise. Do not invent sleeps or a new independent timing policy.
- Resize schedules/replaces deferred cleanup at the same source checkpoint. Buffer disposal/reset and host drop cancel the actual pending callback token. Cancellation must not hold a Buffer borrow across a host callback.
- Call process_idle or process_priority from the requested host callback, not inline from Scheduler::request. Reentrant resize/reset/disposal must remain possible at host request/cancel/report checkpoints.
- Report asynchronous failures through the actual host error path. Keep the original typed/JS error object and partial state. The explicit recovery API is not an excuse for swallowed errors or automatic unbounded retries.

## Portable attachment API

```text
type CleanupFactory = Rc<dyn Fn(&BufferRef) -> Result<Box<dyn Any>, BufferError>>;
BufferSet::install_cleanup_factory(CleanupFactory) -> Result<CleanupFactoryHandle, BufferError>;
BufferService::install_cleanup_factory(CleanupFactory) -> Result<CleanupFactoryHandle, BufferError>;
CleanupFactoryHandle::dispose();
```

Store one optional factory in BufferSet. Invoke it immediately after every new Buffer is wrapped in Rc, before raw Set assignment or viewport filling, including reset paths. On installation, synchronize both raw fields and actual owner slots, deduplicated by Buffer identity, because nested resets can leave them different.

The returned adapter box belongs to a dedicated cleanup-owner slot in that Buffer. A shared slot handle lets factory detachment release adapters without borrowing an outer Buffer that a callback may already hold. Keep only weak slot handles in the factory registration. Explicit registration disposal and Drop remove the factory first, then release all adapters outside state/slot borrows. This prevents a retained terminal from continuing to use a dropped native pump, and does not keep obsolete buffers alive.

The factory callback and adapter destruction may reenter; do not hold the Set or owner-slot borrow across either. A partial installation failure detaches adapters installed by that attempt and removes that registration. It must not remove a replacement installed by a reentrant destructor. Duplicate installation returns CleanupFactoryError::AlreadyInstalled inside BufferError::Dispatch. Detachment during a creation callback returns DetachedDuringCallback before the new buffer is published. Opaque ownership here is only the concrete host adapter lifetime, not a second terminal-state or JavaScript shadow model.

## Host adapters

Scheduler::request receives a CallbackKind, not a callback argument. The native adapter keeps a shared cell containing a weak cleanup-adapter reference. Requests use NativeWriteScheduler's callback timer on the same FIFO/error lane as writes. The adapter target is filled before any posted callback can run. The scheduler's no-inline-callback rule makes that order explicit.

The pending host callback should capture a weak adapter target, not a strong Buffer or Terminal. Otherwise a scheduled task can keep an entire disposed terminal alive. Keeping the registration cell separate from the owning adapter also avoids a self-cycle. A common pair-lifecycle helper is reasonable if both hosts need it, but no additional framework or executor is required.

Use the existing native global task ordering and browser task queue adapters. Cleanup must not create an isolated queue that bypasses the established timer/microtask priority rules. Native and browser ownership/error representations can differ, but their observed cancellation and callback ordering must be compared to the source.

## Execution evidence

A real-host test should write through the actual parser, shrink a terminal that has populated buffers, observe retained backing storage before the requested callback, run the real host callback/event pump, then observe cleanup completion. Repeat across normal/alternate switches, reset with pending work, resize replacing pending work and terminal/host disposal. Exercise request failure, cancellation and error identity. No manual poll substitute, sleep-based assertion or successful no-op is sufficient.

The portable factory currently passes10 unit/reentry tests, and the native host integration passes5 tests with it. `cargo test -p xterm-core --test cleanup_factory --test cleanup_factory_reentrant --test native_buffer_cleanup` and focused Clippy both passed. `cargo test -p xterm-core --lib native_cleanup` also passed two host/logger error-identity tests, and `--lib cleanup_owner` passed two reentrant ownership tests. Browser runtime remains separate.

Exact private backing-byte assertions are already available in the BufferLine child test. Host integration may use public cleanup/dirty-storage observations without adding a production introspection API, but it must demonstrate that the requested host callback performed the work. Retain source hashes, host tokens/callback order, commands, counts and failures. Native Windows execution is not native Linux/macOS or browser evidence.
