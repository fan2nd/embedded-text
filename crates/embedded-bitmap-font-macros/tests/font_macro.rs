//! Compile-fail tests for the public font macro API.

#[test]
fn font_macro_generates_bitmap_font_static() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/font_macro_pass.rs");
}
