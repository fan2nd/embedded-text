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
