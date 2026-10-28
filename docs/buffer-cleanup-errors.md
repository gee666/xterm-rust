# Fallible Buffer cleanup

Task 228 connects Buffer memory cleanup to the task queue's fallible callback path. It does not classify a cleanup failure as a completed Boolean task.

## Error domains

`BufferCleanup::new` wraps the supplied scheduler privately. Its queue error type distinguishes native `BufferError` failures from the scheduler's original error type. This keeps the existing standalone constructor usable without requiring an infallible scheduler to manufacture a Buffer error.

`with_queue` accepts an empty dedicated configured queue whose error type implements `From<BufferError>`. `with_queue_map` accepts explicit conversion and scheduler-wrapping functions instead. The latter has no `std::error::Error` bound on the opaque host value, so a browser `JsValue` can retain its original identity without string conversion. Actual browser/native host ownership and constructor factories remain task 179 and application integration remains task 99.

Both queue forms use `enqueue_try`. A callback failure leaves the current task/index intact and returns before the next duration/deadline observation. A caller may explicitly retry after repairing temporary state. No unbounded retry loop is installed.

Resize-time request failures still propagate immediately through `BufferError::Dispatch`. `take_error` exposes another reference to the same scheduler payload for diagnostics. It is not a latent error channel for callback failures, and a successful later dispatch does not replay an old request failure.

## Cleanup resume position

The original code evaluates `lines.get(position++)` before accessing `cleanupMemory`. An undefined row therefore advances the position before throwing. The Rust method now advances at the same point. A retained source probe under `results/buffer-reentry/cleanup-error-source.json` observed failed position 1, then resumed position 3 and result true.

## Observed checks

The controlled native receipt is `results/build-slots/1793172557926-41596-004d3796-d9f1-4044-8a12-0d4a81d45dc5/receipt.json`. It records six queue injection/error tests passing, alongside eleven host contracts, five raw PTY tests, eleven source callback traces and the 2,222-transition owned/shared corpus.

The six injection/error checks cover configured-queue dispatch, rejection of unrelated pending work, idempotent disposal after late marker registration, task/index retention and one deadline query on failure, source post-increment resume state, and opaque non-Error host payload identity with no late rethrow after recovery.

Scoped Clippy currently reports unfinished task-179 cleanup owner-slot consumption, not an error in this driver. Full workspace formatting and same-snapshot project gates are separate requirements. No browser, WASM runtime, release or other native-platform pass is inferred from these native component checks. Overall equivalence remains NOT PROVEN.
