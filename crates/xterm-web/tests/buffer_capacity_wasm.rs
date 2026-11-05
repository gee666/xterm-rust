#![cfg(target_arch = "wasm32")]
#[path = "../../xterm-core/tests/fixtures/buffer_capacity.rs"]
mod cases;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn sparse_capacity_preserves_the_u32_source_limit() {
    cases::verify_capacity_boundaries();
}
