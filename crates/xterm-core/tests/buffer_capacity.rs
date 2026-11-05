#[path = "fixtures/buffer_capacity.rs"]
mod cases;

#[test]
fn sparse_capacity_preserves_the_u32_source_limit() {
    cases::verify_capacity_boundaries();
}
