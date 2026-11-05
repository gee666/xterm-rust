# Raw SIXEL packet size limit

Task451 repairs `sixelSizeLimit` constructor values and the streaming SIXEL packet check. Full image-addon parity remains open in311/324.

The source copies constructor options twice without coercing this field. It retains the first copy for decoding. Activation with SIXEL disabled, pending decoder input and aborted-sequence input do not convert it. Each ready packet first increments `_size`, then compares that committed number with the raw option using JavaScript `>`. The comparison requests the number primitive hint, supports BigInt comparison and can throw an arbitrary value. Failed coercion does not undo the size increment or mark the handler aborted.

The Rust browser addon now retains the raw value and exposes the same identity through its existing read-only `_opts` inspection. Native `ImageOptions.sixel_size_limit` remains the typed API for native consumers. Browser constructor projection does not convert either constructor copy of this field. Other unresolved raw numeric options remain outside this repair.

The portable `SixelHandler` splits packet handling into `begin_put` and `decode_put`. Native `put` composes those steps with its existing numeric comparison. The browser releases the handler borrow between steps, compares through a small `size > limit` binding expression, then uses the live decoder. This expression is needed to preserve JavaScript relational semantics and exception identity, not to implement terminal decoding in JavaScript. SIXEL decoding, storage, pixels and placement remain Rust-owned. No new dependency or unsafe code is added.

Coercion may synchronously reset or hook the live handler. Continuation does not repeat the earlier abort guard or size increment. Limit warnings still run before marking aborted and releasing the decoder. A throwing warning leaves those later operations unexecuted.

The addon still exposes a read-only private diagnostic record, not source's complete writable `_opts` or `_core._inputHandler` ABI. This repair does not close full original639 image-family suites, img01..48, other raw options, native OS matrices or workspace equivalence.

## Verification

Initial Windows execution under the coordinator's bounded resource grant passed11 native SIXEL tests, including the new staged-packet regression. Native and WASM focused Clippy passed with warnings denied after removing one newly introduced owned-value comparison in a test. All five workspace packages passed separate formatting checks. The exact aggregate formatting command still fails Windows OS206. These scoped checks are not a full workspace gate.

The initial41-function browser suite passed40 and failed1 in each engine. All five new raw-limit functions passed. The sole failure was the already observed resize storage-usage mismatch in448. Read-only investigation confirmed that source and Rust use the same geometry formula. Task455's diagnostic then proved the source terminal was offscreen and IntersectionObserver-paused while its paired Rust terminal was visible.455 changes only the owned test fixture to keep both terminals visible and adds a long-output regression. No application geometry or existing assertion was changed.

The first old-code control restored seven production files exactly to `fe9dcd9e`. Its expected constructor failures exposed a problem in the new test setup: the common Pair constructor had already opened a terminal before the old constructor threw. An asynchronous panic escaped the failing tests and prevented final runner counts. This incomplete control is retained as `initial-negative-*`, not claimed as a successful negative test. The new tests now validate constructor identity before allocating a terminal, so expected old-code failures cannot leak that terminal. The seven production files were restored byte-for-byte in finally.

With accepted455's fixture correction, the complete42-function addon target passes42/42 in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The old seven-production-file control then passes37 and fails exactly the five new tests in Chromium. It completes every discovered function, restores all seven fixed files byte-for-byte in finally and is followed by another42/42 pass in each engine. No functions are ignored or filtered. The control preserves455's fixture fix and all source assertions.

Commands and raw receipts are in `results/image-sixel-size-limit/`. The five new browser functions compare unchanged source constructors and real source/Rust terminal parsing, including20 raw constructor values,19 active primitive forms, first-versus-second copies, readiness/abort guards, exact exceptions, committed-size continuation after a throw, reset reentry, warnings and actual image pixels. The comparison after a throwing conversion confirms that the first increment remains committed: a subsequent byte exceeds the new1-byte limit in both implementations.

The final identity audit validates104 Rust/test/script pins,128 comparator dependency pins,121 of those against the original reference, all776 original manifest entries and seven restored control files. The retained native executable is also hashed for independent replay. The other seven comparator dependencies reside in the borrowed original dependency tree. Historical O: paths identify the actual build, not permanent installations.

```sh
cargo test --locked -j2 -p xterm-addons --lib image::sixel
cargo clippy --locked -j2 -p xterm-addons --lib --tests -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown \
  --lib --test image_addon_browser --test image_sixel_terminal_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node results/image-sixel-size-limit/negative-control.mjs
cargo fmt --all -- --check
```

Use the tool/dependency environment in `docs/image-canvas.md`. The exact workspace formatting failure is retained as `fmt-all.log`; the five successful member checks have separate logs. Full workspace, native OS and complete original suite verification remain separate project work, not inferred from these checks.

This branch starts from the committed448 assembly at `fe9dcd9e`, which already contains the accepted431/439 raw placeholder/storage union. It privately adopts455 source `73fa8f06` as `997771c1`, keeping both455 and451 test input pins. Only451 changes should be picked for later integration, not that private455 copy or any448 prerequisites. No main ref operation is owned by451.
