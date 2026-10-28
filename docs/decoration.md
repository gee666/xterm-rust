# Decoration service and logical-line cache

`crates/xterm-core/src/decoration.rs`, `decoration_service.rs` and `decoration_cache.rs` port the common DecorationService, Decoration and DecorationLineCache classes. `docs/decoration-acceptance.md` lists the full branch contract. OSC link registration is separate in `osc_link_service`.

## Integration

Construct `DecorationService::with_buffers(&BufferSet)` to attach to the active buffer and follow normal/alternate activation. The service owns a weak activation subscription and releases it on disposal or native owner drop. `new()` plus `attach_to_buffer_lines` supports explicit hosts and focused tests.

Registration takes `Rc<RefCell<DecorationOptions>>`. Options remain shared and mutable. A disposed marker returns `Ok(None)`. `Decoration::clone` preserves object identity. The service uses the shared deferred SortedList for source ordering and snapshots. Cell iterators defer bucket lookup until first next, retain the service while active and stay exhausted after completion. They retain a live bucket so removing a visited entry shifts later entries exactly as JavaScript array traversal does. `decorations_on_line` returns a read-only shared bucket handle whose length/content reflect later mutations. Marker disposal removes a decoration from its original indexed line before the public removal event. No RefCell borrow crosses a user callback.

Logical line coordinates use the shared signed Marker representation. Heights are unsigned integers; x and width preserve floating-point comparisons. Browser bindings must validate integer geometry instead of silently truncating arbitrary JavaScript values. Opaque host element handles use `Rc<dyn Any>`, keeping browser DOM types out of the core. A renderer sets the element with `set_element`, then fires `fire_render` when the upstream render contract calls for it. Lazy foreground/background colors reuse the shared CSS parser and accept a browser canvas fallback. Their cached Rc/RefCell handles preserve IColor identity and external mutation across calls. Parse failures propagate and do not populate the cache.

Insert/delete line events enqueue cache work. Subscribe to `line_cache().on_sync_requested()` and schedule `flush_pending()` through the browser's real queueMicrotask. Do not flush synchronously inside the line event because buffer marker listeners may not have updated yet. Native hosts flush explicitly at the corresponding microtask boundary. Flush returns a Result so invalid signed conversions are not successful no-ops. Clear cancels queued work; a stale host callback finds no pending actions. Browser renderer/microtask assembly belongs to task32 and remains separately required.

## Verification

```sh
node scripts/decoration-oracle.mjs
cargo test -p xterm-core --test decoration_upstream --test decoration_differential --test decoration_edges
cargo test -p xterm-core --release --test decoration_upstream --test decoration_differential --test decoration_edges
cargo clippy -p xterm-core --test decoration_upstream --test decoration_differential --test decoration_edges -- -D warnings
```

The original ten tests execute under actual Mocha with all source assertions unchanged. The Rust upstream file preserves all ten cases, including real Buffer line insertion and the deferred cache flush. Seven additional tests cover disposed markers, shared options/defaults, layers, event order, lazy colors, live versus snapshot traversal, attachment replacement, reset cancellation, real BufferSet activation and opaque render identity. One differential test replays1024 operations against original-JavaScript expected states from seed0xdec0a710. Every step compares all retained marker/decoration disposal states, sorted decoration order and240 ordered cell/layer lookups.

Observed native Windows results are in `results/decoration/oracle.json`, `oracle.log`, `tests-stable.log` and `release-current.log`. JavaScript10/10 and the initial Rust18/18 pass. The combined release log also independently reruns32 cell tests. Independent review found eager generator lookup and copied bucket views; those are fixed. `results/color-zone/decoration.log` records20 passing Rust tests after the generator fix, including independent cold-iterator and fused-exhaustion regressions. Cached colors now also retain shared identity. After the reviewed disk cleanup, the latest generator, live-bucket and cached-color identity fixes pass all21 scoped Rust tests in debug and release. Independent reviewer Fergus reran the unchanged source baseline,1024-step fixture verification, native tests and focused Clippy. His findings and raw commands are in `results/reviews/fergus-decoration-review.json`. Early compilation snapshots preserve concurrent API migration failures. `clippy-current.log` retains unrelated unfinished screen_shared and headless event helpers. Those owners completed their wiring, and `clippy-clean.log` records a fresh successful focused cell/decoration Clippy run with warnings denied. This is not a whole-workspace clean claim. Focused source files are formatted and remain below350 lines. No unsafe operations or new dependencies were added.

Task64 is independently accepted after the recorded fixes. Cell task73 has a separate completed independent review in `results/cell/isaac-review-run2.json`. Browser DOM/microtask rendering, three-browser E2E and native Linux/macOS evidence are not established by these tests. Overall equivalence is NOT PROVEN.
