#[path = "support/font_cff2_fonts_assertions.rs"]
mod assertions;

#[test]
fn complete_source_cff2_font_and_private_constructors() {
    assertions::assert_corpus();
}
// Browser suites run their own corpus directly; this recursive predecessor
// chain uses native-only support and must not enter the WASM test build.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn predecessor_cff_constructor_assertions_remain_intact() {
    assertions::assert_previous();
}
