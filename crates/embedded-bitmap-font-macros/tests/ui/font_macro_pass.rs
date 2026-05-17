use embedded_bitmap_font_macros::bitmap_font;

bitmap_font! {
    pub static DEMO_FONT: embedded_bitmap_font::FontData<'static> = {
        path: "../../.ref/Cubic_11.ttf",
        size: 12,
        glyphs: "AR你"
    };
}

fn main() {
    assert_eq!(DEMO_FONT.char_size, 12);
    assert_eq!(DEMO_FONT.index, "AR你");
    assert!(DEMO_FONT.glyph('A').is_some());
    assert!(DEMO_FONT.glyph('R').is_some());
    assert!(DEMO_FONT.glyph('你').is_some());
}
