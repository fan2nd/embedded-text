#[test]
fn drawable_text_rejects_mismatched_ascii_and_cjk_heights() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/drawable_text_mismatched_cell_heights_fail.rs");
}
