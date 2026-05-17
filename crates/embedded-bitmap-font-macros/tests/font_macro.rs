//! Compile-fail tests for the public font macro API.

#[test]
fn font_macro_generates_bitmap_font_static() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/font_macro_pass.rs");
}

#[test]
fn font_macro_supports_ranges_and_deduplicates_glyphs() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/font_macro_ranges_pass.rs");
}

#[test]
fn font_macro_generates_multiple_sizes() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/font_macro_multi_size_pass.rs");
}

#[test]
fn font_data_supports_multiple_font_blocks_with_y_offset() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/font_data_multi_font_pass.rs");
}

#[test]
fn font_data_rejects_duplicate_index_characters() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/font_data_duplicate_index_fail.rs");
}
