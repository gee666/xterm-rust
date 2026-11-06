#[path = "support/font_cff_fdselect_assertions.rs"]
mod assertions;

#[test]
fn unchanged_full_source_fdselect_constructors() {
    assertions::assert_corpus();
}
// Browser suites run their own corpus directly; this recursive predecessor
// chain uses native-only support and must not enter the WASM test build.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn unchanged_predecessor_constructor_assertions() {
    assertions::assert_previous();
}
