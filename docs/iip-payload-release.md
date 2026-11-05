# IIP payload release boundary

The IIP handler now retains decoded BASE64 state when it emits a `DisplayRequest`. Request bytes are owned, but emitting the request is not successful host preparation. The original `IIPHandler.ts` retains its payload if QOI decoding, ImageData construction or Blob construction throws synchronously.

A live browser coordinator must call `IipImageDecoder::prepare_transfer` with the actual `RefCell<IipHandler>`. The adapter holds no protocol borrow across browser calls. It releases the live payload after QOI/ImageData preparation or Blob construction, before creating the unscaled canvas or calling `createImageBitmap`. A synchronous exception before release keeps the current payload. An exception after release does not restore it. If an ImageData callback starts a multipart transfer, successful outer preparation releases that new live payload too, matching the source. This is deliberately not a generation-guarded release.

`IipImageDecoder::prepare` remains a standalone operation for owned requests without an associated protocol controller. It cannot update protocol state. The full ImageAddon coordinator must use `prepare_transfer`, not the standalone method. Non-browser hosts can call `IipHandler::release_payload` at their corresponding successful preparation boundary. The recording IIP oracle does so when its successful Blob host consumes a display request.

## Verification

The new browser regression compares unchanged original `IIPHandler` execution against native IIP parsing and the real Rust browser decoder. It observes loaded BASE64 bytes at ImageData, Blob, canvas context and bitmap calls; checks exact thrown-value identity; and compares payload state after normal and nested multipart transfers. Eight directed cases run in each browser. Existing pixel, retained QOI memory, resize, bitmap and storage tests remain intact.

The native `iip_payload` tests check retained state after dropping a failed host request and release of a nested live payload. Existing metadata tests still require zero loaded bytes after the host release, and now also assert retained bytes before that release. The original protocol oracle still compares all existing state snapshots after its successful recording host.

Commands and observed results will be retained under `results/iip-payload-release/`. Full ImageAddon activation and original image suite/demo acceptance remain task311 and task324. This focused repair does not prove full addon equivalence, raw JS option/getter ordering, decoder warning callback ordering, or native Linux/macOS behavior.
