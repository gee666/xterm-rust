# Buffer activation and scrolling

`buffer::buffer_set` and `buffer::service` port the original BufferSet.ts and BufferService.ts using actual Rust Buffer, BufferLine, CircularList, marker, event and options implementations. The detailed contract is `docs/acceptance/buffer.md`, section E. This is component integration, not a complete terminal or browser claim.

## API and ownership

`BufferService::new(Rc<OptionsService>)` creates the filled normal and empty alternate buffers. Constructor dimensions retain the source minimum of two columns and one row, including source falsy/default and numeric option coercion. Nonintegral/nonfinite/unrepresentable allocation dimensions return BufferError rather than allocating invalid Rust storage. `resize` is a separate internal path and permits one column, matching upstream internal buffer tests.

The service exposes `pub buffers: Rc<BufferSet>`, `cols()`, `rows()`, `buffer()`, `is_user_scrolling: Cell<bool>`, resize and scroll events. Public buffer views can clone this BufferSet handle. `active()`, `normal()` and `alt()` each return `Rc<RefCell<Buffer>>`. References survive activation; reset replaces buffers. `BufferDimensions` shares live service dimensions without confusing them with each buffer's last resize.

Activation fires after cursor copy/fill/clear. Alternate markers dispose before switching back emits activation. Service activation listeners forward scroll before subsequently registered activation listeners. Repeated activation of the current buffer produces no event. Activation callbacks can synchronously switch back; tests assert nested event order and final active identity.

`scroll` preserves cached blank-line width/foreground/background selection, wrapped flags, append/recycle/splice branches, DECSTBM region shifts, base/display tracking and scroll events. `scroll_lines(i64, bool)` uses signed integer line displacement and an explicit suppression flag. It clamps without overflow for i64::MIN/MAX. Browser-number conversion remains a binding responsibility, not an unchecked Rust cast.

Option listeners hold Weak state and are explicitly disposable, avoiding an Options/BufferSet cycle. Callback failure propagates through OptionsService::set as EmitError::Callback, retaining the actual BufferError object. Callers can downcast it without converting to text. The option value remains committed before listener failure. Borrow conflicts return BufferError rather than panic.

`dispose()` propagates marker/event failures and unregisters resources in source order. Native Drop releases subscriptions without invoking fallible user marker callbacks, since Drop cannot return those errors. Applications should explicitly dispose when callback sequencing matters. No unsafe code or new dependency was added.

## Evidence

```sh
node scripts/buffer-service-oracle.mjs
cargo test -p xterm-core --test buffer_set_upstream --test buffer_service_upstream --test buffer_service_edges --test buffer_service_differential
cargo test -p xterm-core --release --test buffer_set_upstream --test buffer_service_upstream --test buffer_service_edges --test buffer_service_differential
cargo clippy -p xterm-core --lib --test buffer_set_upstream --test buffer_service_upstream --test buffer_service_edges --test buffer_service_differential -- -D warnings
```

The JS oracle reuses the existing buffer-baseline esbuild/Mocha method, executes the unchanged original suites with their actual source dependencies, and checks a deterministic fixture without overwriting it. Use `--generate` only for reviewed reference changes. It requires the source_repo development dependencies already used by the complete upstream baseline runner.

Observed Windows results with Node24.15.0 and Rust1.85.1:

- BufferSet original suite6/6 passed; all six tests mapped into Rust. JavaScript instanceof assertions map to compile-time concrete Rust Buffer types plus runtime distinct-handle assertions.
- BufferService original suite7/7 passed; all seven assertion groups migrated unchanged in meaning.
- Ten directed Rust regressions passed for activation/event/marker order, reentrant activation, reset identity, live options/disposal, propagated option errors, dimensions/resize flags, display suppression/clamping, constructor coercion, cached blanks and disposal-failure/post-disposal-reset behavior.
-64 deterministic traces x64 operations,4096 full snapshots passed in debug and release. Operations include scrolling, display movement, margins, cursor, both activations, resize, reset, scrollback/tab option changes, line writes and user-scroll state. Each snapshot compares both buffers' cells, widths encoded in content, foreground/background, UTF16 text, row wrapped flags, cursor/base/display/region/capacity/tab state, active identity and ordered service/activation events.
- Seed99070207, hexadecimal0x05e7b0ff. The generated corpus uses no extended attribute words; CellData/BufferLine/Buffer-specific suites cover those separately. This limitation is explicit, not a claim of exhaustive terminal-state differential coverage.

Raw source hashes, original case names/counts and fixture hash are in `results/buffer-service/oracle.json`. Rust debug/release and Clippy logs are retained beside it. Temporary JS bundles stay in `tmp/buffer-service-oracle`. Tests use current real Rust buffers, not mock terminal implementations.

Task130 releases Buffer borrows before scroll-list and Set marker callbacks. `CircularList::share()` keeps the actual list storage and emitter identities alive without borrowing Buffer. A shared marker loop handles owned and Rc-backed calls, preserving live iteration and failure state without duplicate implementations. Six source-generated traces assert callback-time full buffers and nested event order for marker read/write, nested scroll, marker insertion during alternate cleanup, old/current buffer identity during reset, alternate activation during trim and resize during trim. All 25 current component tests pass in debug and release, including the4096-step corpus and six reentrant traces. Focused Clippy and formatting pass in `results/buffer-service/reentrant-*.log`. Parent task122 still owns callbacks during resize/reflow itself, which these tests do not establish.

Task99 separately owns actual parser/InputHandler/addon integration, host idle cleanup and observed native platform execution. Neither these component tests nor a configured CI workflow establish Linux/macOS or browser runtime evidence. Overall equivalence remains NOT PROVEN.
