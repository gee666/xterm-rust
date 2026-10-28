# Buffer state and resizing

`buffer::Buffer` ports `source_repo/src/common/buffer/Buffer.ts`. It uses the actual Rust BufferLine, CellData, CircularList, options, events, markers and reflow helpers. It does not delegate terminal operations to JavaScript.

The detailed acceptance checklist is `docs/acceptance/buffer.md`. Passing these component checks does not establish full terminal equivalence.

## API and ownership

`Buffer::new(has_scrollback, cols, rows, Rc<OptionsService>)` returns `Result<Buffer, BufferError>`. A new buffer is empty. `fill_viewport_rows` fills it once; alternate-buffer creation therefore does not allocate rows until activation.

- `lines` is `CircularList<LineRef>`, where `LineRef` is `Rc<RefCell<BufferLine>>`. Cloning a reference preserves the row identity, combined text and shared extended attributes. Reflow reorders these references rather than replacing surviving rows.
- Cursor, display/base, scrolling region and saved fields retain source names in snake case. Coordinates are Rust nonnegative integers except `saved_y`, which is signed. The source can decrement saved Y below zero when shrinking an empty wrapped row. Treating the number of new rows as an unsigned subtraction incorrectly rejected source-valid inputs.
- `host_dimensions` shares the service's live columns/rows. A standalone `Buffer::resize` updates internal dimensions only. Blank-line construction uses the host columns, exactly as source `getBlankLine` reads `_bufferService.cols`.
- `options` remains a shared live OptionsService. Scrollback, tab width, Windows PTY configuration and cursor-line reflow settings are read at the relevant operation. `WindowsPty` is the typed payload carried by the options adapter.
- `get_null_cell` and `get_whitespace_cell` return owned scratch-cell values rather than aliases of JavaScript's reused scratch object. Their extended attributes still share source assignment identity. This representation avoids an alias to a scratch object being mutated through another Rust borrow.
- Text translation returns UTF-16 units. Wide-cell boundaries, combined strings, astral values and lone surrogates do not pass through a lossy UTF-8 conversion.
- `saved_charset=None` matches source `DEFAULT_CHARSET=CHARSETS['B']`, whose value is `undefined`, not an empty map.
- Marker handles clone shared identity. CircularList insert/delete/trim events update actual markers, and marker disposal removes them through weak collection references. Pointer identity is used rather than numeric IDs, which can stop advancing at the JavaScript number limit.

Recoverable options, dimension, missing-row, borrow, storage, reflow and event failures return typed errors. Actual callback causes survive `EmitError::Callback`. They are not recast as successful empty strings or cleared-listener errors.

## Resize behavior

Rows and columns follow the source ordering: grow capacity, grow columns, adjust rows/viewport, trim top on capacity reduction, clamp cursors, apply reflow and then shrink physical rows. Reflow observes the ConPTY build 21,376 boundary and cursor-line exclusion option.

Smaller reflow copies backwards, batches new rows, emits corrected insertion events, then trims. Larger reflow emits deletion events before rearranging references. Both preserve combined cells and wide-character wrap gaps. Internal ASCII buffers support one column because the upstream tests use that path. The public BufferService constructor still enforces the source minimum of two columns.

Signed insertion counts matter for an empty wrapped row. It can produce zero destination rows and a source count of minus one without removing the original physical row. The saved cursor adjustment still observes that negative count. This behavior is retained rather than silently repaired.

## Host-scheduled memory cleanup

`buffer::BufferCleanup<S: Scheduler>` attaches the existing Rust TaskQueue to a shared buffer. The host supplies real idle/priority scheduling and dispatches `process_idle` or `process_priority` when the requested callback fires. Native and browser hosts must retain one adapter per buffer for automatic cleanup.

Resize cancels old queued work and schedules cleanup only when more than 10% of rows report excess capacity. Each cleanup task stops after 101 cleaned rows, matching the source's `counted > 100` condition, and performs the final rescan. There are no sleeps or background threads in the core.

Scheduler failures propagate from resize through the actual `EmitError::Callback` object. `take_error` retains another reference to the same host error object. `retry_pending` is explicit recovery, not an unbounded retry loop. A buffer borrow conflict during callback processing is also returned. Dropping the adapter removes its subscription and cancels outstanding tokens; queued tasks hold only a weak buffer reference.

`poll_cleanup` remains available for an application's explicitly managed cooperative loop. It is not evidence that a browser/native host attached automatic scheduling. Task 99 covers that application integration.

## Verification commands

From repository root:

```sh
node scripts/buffer-baseline.mjs
node scripts/buffer-resize-oracle.mjs
cargo test -p xterm-core --test buffer_screen_upstream --test buffer_screen_edges --test buffer_resize_differential --test buffer_contracts
cargo test -p xterm-core --release --test buffer_screen_upstream --test buffer_screen_edges --test buffer_resize_differential --test buffer_contracts
cargo clippy -p xterm-core --lib --test buffer_contracts -- -D warnings
cargo fmt --all -- --check
```

The independent Buffer oracle is task 67. It executes all 61 original Buffer tests unchanged, captures every resize before/after snapshot and compares full cells, canonical text, row identity, markers and ordered line events. Its corpus adds 128 seeded traces with eight resizes each, for 1,110 resize pairs including the 86 original calls. Seed is `0xb0ff67`. Source hashes, test names, fixture hashes, failures and command results are under `results/buffer`.

`buffer_contracts.rs` adds eleven tests for real host requests/cancellation, fallback scheduling, same-object error preservation, borrow recovery, adapter ownership, batch thresholds, tab setup, saved-state reset rules, marker events, cell attributes and safe bounds. These pass on native Windows. Debug/release and full corpus results must be read from the current command logs rather than inferred from this description.

## Remaining integration work

Task 122 tracks source-legal observer reentrancy during resize/scroll/marker disposal. Holding an outer mutable Buffer borrow across a nested marker callback can currently reject a read or nested mutation that JavaScript permits. This is an implementation gap, not an external blocker or a waived language difference.

Task 99 tracks actual parser/InputHandler/addon writes and real host cleanup scheduling, plus observed platform runs. Native Linux/macOS and browser evidence are not supplied by these component tests. Full equivalence remains NOT PROVEN.
