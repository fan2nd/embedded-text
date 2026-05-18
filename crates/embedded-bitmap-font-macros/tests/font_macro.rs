//! Compile-fail tests for the public font macro API.

#[test]
fn font_data_supports_multiple_font_blocks_with_auto_y_offset() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/font_data_multi_font_pass.rs");
}

#[test]
fn font_data_rejects_duplicate_index_characters() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/font_data_duplicate_index_fail.rs");
}

#[test]
fn old_bitmap_font_macros_are_removed() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/old_bitmap_font_removed_fail.rs");
    t.compile_fail("tests/ui/old_bitmap_fonts_removed_fail.rs");
}

#[test]
fn bitmap_font_type_alias_is_removed() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/bitmap_font_type_alias_removed_fail.rs");
}
