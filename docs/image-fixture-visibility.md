# Paired image fixture visibility

Task455 fixes the Rust-owned differential fixture, not terminal geometry or image production code. Full equivalence remains NOT PROVEN.

The combined431/439 image suite discovered36 functions. The initial unchanged assertions produced35 passes and one failure in each browser. `alternate_resize_public_reset_and_disposal_match_source` reported Rust storage usage0.001392 versus source0.000672. The retained Chromium diagnostic records the cause. At a720px viewport, the Rust terminal occupied551..755 and remained partially visible, while the source terminal occupied755..959 and its RenderService was paused. Added test output had moved only the second stacked terminal offscreen.

Both original images were14x12 with7.7x17 cells. Rust rendered after resizing to24 columns, rescaling to15x12 at7.708333333333333x17. The paused source correctly retained the14x12 original. The extra180 pixels explain the usage difference. Source DomRenderer uses the same rounded canvas/column formula as Rust. Changing geometry or suppressing the usage assertion would hide the invalid comparison.

The owned Pair helper now positions both parents at the viewport top, with Rust at left0 and source at left50%. Font, dimensions, DPR, options, parser input, lifecycle and every existing assertion remain unchanged. A new regression inserts4096px of output, checks that source rendering is unpaused and both terminals are within the viewport, then performs the same SIXEL/write/resize/storage comparison. Its temporary element is removed by RAII even when the test fails. No original source fixture or assertion is edited.

`results/integration455/` retains initial failures, diagnostic geometry, final browser receipts, source pins, warning-denied focused WASM Clippy and member formatting. Final formatted inputs pass37 functions in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4, with zero failed, ignored or filtered functions. These are directed real Rust/source image tests, not the complete original image or demo matrix.

Reproduce from the repository root using installed prerequisites from README. Set `SOURCE_REPO` to the unchanged reference, `REFERENCE_NODE_MODULES` to its installed dependencies, `PLAYWRIGHT_MODULE` to its Playwright index.mjs, `WASM_BINDGEN` to the matching0.2.100 executable, `WASM_TEST=image_addon_browser`, `CARGO_BUILD_JOBS=2`, and owned temporary/target paths. Run `node scripts/image-canvas-check.mjs`. No browser timeout or retry setting changes.
