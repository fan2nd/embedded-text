use embedded_bitmap_font_macros::bitmap_fonts;

bitmap_fonts! {
    path: "../../.ref/Cubic_11.ttf",
    glyphs: "AB你",
    pub static {
        FONT_12: embedded_bitmap_font::FontData<'static> = 12,
        FONT_18: embedded_bitmap_font::FontData<'static> = 18,
    }
}

fn main() {
    assert_eq!(FONT_12.char_size, 12);
    assert_eq!(FONT_18.char_size, 18);
    assert_eq!(FONT_12.index, "AB你");
    assert!(FONT_12.glyph('A').is_some());
    assert!(FONT_18.glyph('你').is_some());
}
