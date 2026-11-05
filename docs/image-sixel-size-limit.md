# Raw SIXEL packet size limit

Task451 repairs `sixelSizeLimit` constructor values and the streaming SIXEL packet check. Full image-addon parity remains open in311/324.

The source copies constructor options twice without coercing this field. It retains the first copy for decoding. Activation with SIXEL disabled, pending decoder input and aborted-sequence input do not convert it. Each ready packet first increments `_size`, then compares that committed number with the raw option using JavaScript `>`. The comparison requests the number primitive hint, supports BigInt comparison and can throw an arbitrary value. Failed coercion does not undo the size increment or mark the handler aborted.

The Rust browser addon now retains the raw value and exposes the same identity through its existing read-only `_opts` inspection. Native `ImageOptions.sixel_size_limit` remains the typed API for native consumers. Browser constructor projection does not convert either constructor copy of this field. Other unresolved raw numeric options remain outside this repair.

The portable `SixelHandler` splits packet handling into `begin_put` and `decode_put`. Native `put` composes those steps with its existing numeric comparison. The browser releases the handler borrow between steps, compares through a small `size > limit` binding expression, then uses the live decoder. This expression is needed to preserve JavaScript relational semantics and exception identity, not to implement terminal decoding in JavaScript. SIXEL decoding, storage, pixels and placement remain Rust-owned. No new dependency or unsafe code is added.

Coercion may synchronously reset or hook the live handler. Continuation does not repeat the earlier abort guard or size increment. Limit warnings still run before marking aborted and releasing the decoder. A throwing warning leaves those later operations unexecuted.

The addon still exposes a read-only private diagnostic record, not source's complete writable `_opts` or `_core._inputHandler` ABI. This repair does not close full original639 image-family suites, img01..48, other raw options, native OS matrices or workspace equivalence.

## Verification

Pending focused native and real three-engine browser execution under the coordinator's resource grant. Commands and raw receipts will be retained in `results/image-sixel-size-limit/`. The five new browser functions compare unchanged source constructors and real source/Rust terminal parsing, including raw values, readiness/abort guards, exact exceptions, reset reentry, warnings and actual image pixels.

This branch starts from the committed448 assembly at `fe9dcd9e`, which already contains the accepted431/439 raw placeholder/storage union. Only451 changes should be picked for later integration. No main ref operation is owned by451.
