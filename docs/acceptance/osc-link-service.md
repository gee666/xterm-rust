# OSC link service acceptance

Scope is `common/services/OscLinkService.ts` and all three original OscLinkService.test.ts cases. This is the link-id/marker store, not the OSC8 parser or browser URL opener. Those integrations need their own evidence.

## Registration and lookup

LNK01. Every service starts link ids at one. Absent id allocates a new link and marker every call, even for an equal URI. Present id, including empty id, keys by exact UTF-16 `id + ';;' + uri`. Equal keys reuse a link id and add at most one marker per current line. Different ids/URIs differ unless the source delimiter encoding collides. Explicitly test the collision `a;;b,c` versus `a,b;;c`, which upstream treats as equal.

LNK02. Preserve exact UTF-16 ids/URIs, including empty strings, separators, astral characters and lone surrogates. The original data object remains live: lookup returns the same identity, not a defensive copy. Mutation is visible to callers. Keys are captured when created and do not recompute on later data mutation. Rust maps this to Rc<RefCell<OscLinkData>> and documents recoverable borrow conflicts rather than panicking.

LNK03. Numeric ids are service-local, sequential and remain positive. Preserve the JavaScript counter's 2^53 increment saturation. At saturation, map overwrite behavior must follow source, not wrap to zero. No global id registry is needed for links. Marker ids remain the separate Marker.ts identity.

LNK04. Unknown ids return None. Reading a known id has no cursor/buffer mutation and does not allocate markers. Failed registration due to a typed buffer/data borrow error must not silently return a successful id. Document the typed u64 id and i64 line domains versus JavaScript number/accessor objects.

## Marker lifetime and mutation

LNK05. Register markers at the live active buffer's ybase+y, not at a stale normal buffer or viewport-relative row. Re-registering a named link on a different line adds a marker, but re-registering on an already-live line does not. Equal lines in different active buffers are still deduplicated because the source compares line only.

LNK06. `addLineToLink` is a no-op only for unknown ids or a line already represented by a current marker. A moved marker's new line participates in the equality check. New markers belong to the active buffer at call time. Negative i64 line values retain source behavior rather than being clamped silently.

LNK07. A marker's disposal removes that marker by true object identity. One surviving line keeps the link data/id/key alive. Disposal of the last line removes numeric lookup and, if current data.id is present, removes the original captured string key. Verify re-registration then allocates a new id.

LNK08. Data mutation affects removal exactly as source. If a named data object's id becomes absent before last disposal, the captured keyed entry remains while numeric lookup is removed. Registering the old key then returns the stale id, and addLine does nothing because numeric lookup is missing. If an unnamed data object gains an id, cleanup still removes numeric lookup without inventing a key. Do not 'fix' these source behaviors.

LNK09. No RefCell borrow crosses a callback or buffer mutation. Marker subscriptions capture weak service/entry references so the Rust port creates no strong self-cycle. Dropping the service must release its stores without disposing unrelated buffer-owned markers or invoking callbacks unexpectedly. Later marker disposal after service drop must be safe. Marker disposal errors propagate, never disappear into a void adapter.

## Real integration and evidence

LNK10. Faithfully port all three original tests and five assertions: repeated ids, alt-buffer zero-scrollback trim expiring a link id, and exact stored data. Execute the original test bodies against original BufferService dependencies. Record any unrun baseline honestly rather than treating a mocked service as that test suite.

LNK11. Compare deterministic JS/Rust traces over registrations, explicit added lines, cursor movement, active-buffer switches, direct marker moves/disposal, unknown ids and live-data mutation. Compare returned ids/data, marker counts/lines and lookup expiration. Include seeded traces and retain exact failing inputs, not just a digest.

LNK12. Rust integration uses the actual BufferService and Buffer markers, not a fake marker provider. Cover scroll trim, alt buffer, multiple lines, last-marker disposal, named-key collision and source data-mutation quirks. OSC8 InputHandler integration must set/clear URL ids on real cells and the browser provider must resolve them through this same service; unit-service success alone does not prove that path.

LNK13. Source mapping, exact command/runtime/seed/counts, source and fixture hashes, original suite result, Rust assertions, fmt, warning-free Clippy and independent review are required. Source attribution is retained. Native Linux/macOS and real browser checks remain unrun until observed. Full terminal equivalence is NOT PROVEN by these service checks.
