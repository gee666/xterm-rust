# Raw image palette values

Task528 repairs `sixelPaletteLimit` on the browser ImageAddon. Full image-family parity remains open in311/324. `pixelLimit` is still a typed projection, and the private option view is read-only rather than the source's complete writable record.

The source copies options twice without converting either palette value. The first copy is current state. Reset and XTSMGRAPHICS SET_DEFAULT restore the second copy before storage disposal, reply conversion or handler reset. READ and SET_DEFAULT interpolate the raw value with the string primitive hint. Replies preserve raw UTF-16, including lone surrogates. SET validates the parser's numeric parameter against the protocol's4096-color maximum and replaces the raw value with that number.

SIXEL captures the current palette value at handler construction. After asynchronous decoder loading, it first validates `value > 4096`, then performs a separate numeric conversion for the decoder's WASM i32 argument. Objects can return different values on those two calls. The native decoder receives the converted and unsigned/clamped limit, not a second validation of that numeric result. Error identity and the initialization promise rejection are preserved.

Each ready hook commits packet size zero and clears aborted before looking up the background or converting the live palette argument. `undefined` uses the decoder constructor's original option as the default argument. Pending hooks skip background and conversion. No native handler or option borrow crosses host callbacks.

`Decoder.release` is also observable. It frees canvas/band storage before converting the original constructor option and resetting decoder state. A reset then zeroes all4096 colors before converting its live init argument and restoring the default256 colors. Release conversion failures skip the later operations. Reentry continues with the same live decoder, without repeating the earlier prologue or clearing storage a second time.

Portable native APIs split hook, release, reset and abort into explicit stages. Numeric convenience methods retain their existing signatures and compose those stages. Only the browser raw-palette decoder defers the release inside a rejected memory check. It returns that release request to the host before warning formatting. A throwing release becomes the caught decoder error, then the warning runs before aborted is set and another release is attempted. Snapshot and large-image-copy releases likewise evaluate the raw argument at their actual call sites. Decoding, pixels, placement, storage and control dispatch remain Rust-owned. The JavaScript expressions only implement language conversions, template interpolation and exact exceptions. No dependency or unsafe code is added.

## Verification status

Source commitf40366b1 is based directly on main11ad71a8. It has no private prerequisites and does not replay515. All61 inherited addon test functions are kept. Ten new functions compare raw constructor/default identity in source/direct/shipped bindings, disabled controls, conversion hints, pending/ready behavior, primitive forms, undefined defaults, exact exceptions, synchronous reset reentry, asynchronous initialization rejections, memory/abort warnings and real small/large SIXEL pixels.

Native initial execution found one incorrect assumption in a new test: numeric abort release already restores the constructor palette and fill. The retained initial failure log and corrected test check those states before and after the hook prologue. Existing assertions were not weakened. Initial WASM lint also caught an eight-argument diagnostic constructor and one unused import. The constructor was removed in favor of explicit struct fields, and the import was removed.

Current focused native results are14 SIXEL tests and209 addon tests, with no failures or ignored tests. Strict native addon and WASM library/two-image-target Clippy pass. The five package formatting checks pass. Exact aggregate formatting fails Windows OS206 and is retained as a failure, not a waiver. The first browser matrix completed70/71 in each engine. All61 inherited functions passed. The new lone-surrogate reply assertion exposed a lossy UTF-16 conversion in the shared Pair comparison. The fixture now creates a JsString from raw code units, keeping the same comparison and adding no replacement characters. Firefox also reported prevented unhandled-rejection events to its debugging protocol. The new initialization test now observes the exact created promise chains before rejection, restoring the Promise constructor and then method in finally. It still compares every rejection reason and conversion trace; no browser errors are filtered or ignored.

After these test-fixture corrections, all71 functions pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4, with zero page errors, ignored or filtered functions. The exact eleven-production-file control restores main11ad71a8 and completes61 inherited passes with exactly the ten new palette failures in Chromium. It restores all eleven fixed files byte-for-byte in finally, followed by another71/71 pass in each engine. The unchanged typed SIXEL producer suite passes15/15 per engine. Final strict Clippy and package formatting checks were repeated after the final test edits.

The read-only identity audit passes435 repeated source pins across initial/control/final/producer phases,508 comparator input pins, both776-file reference manifests, twelve immutable compiler/browser artifact pins and the retained native executable hash. Each of the four WASM targets was actually recompiled with fresh=false in Cargo's artifact record. The runner overwrites its generated comparator entry between addon and producer targets, so exact entry reconstructions are retained and required to match the historical SHA before the audit uses them. No compiled comparator source is edited. No complete workspace/platform result is claimed.

Re-run commands with an owned O: worktree, private target/TEMP/TMP and current resource grants:

```sh
node results/image-palette-values/run.mjs native
node results/image-palette-values/run.mjs format
node results/image-palette-values/run.mjs initial
node results/image-palette-values/negative-control.mjs
node results/image-palette-values/run.mjs final
node results/image-palette-values/run.mjs producer
```

Set `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE` and matching0.2.100 `WASM_BINDGEN` to the existing reference dependencies. The original and owned physical reference copy each passed776 manifest hashes before runtime. Complete original639 image suites, img01..48, private writable/prototype state, GC and native Windows/Linux/macOS final matrices remain separate obligations. Overall equivalence is NOT PROVEN.
