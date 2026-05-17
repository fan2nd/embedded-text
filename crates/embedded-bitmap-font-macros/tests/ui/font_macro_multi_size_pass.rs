use embedded_bitmap_font_macros::bitmap_fonts;

bitmap_fonts! {
    path: "../../.ref/Cubic_11.ttf",
    glyphs: "AB你",
    pub static {
        FONT_12: BitmapFont<'static> = 12,
        FONT_18: BitmapFont<'static> = 18,
    }
}

fn main() {
    assert_eq!(FONT_12.size, 12);
    assert_eq!(FONT_18.size, 18);
    assert!(FONT_12.glyph('A').is_some());
    assert!(FONT_18.glyph('你').is_some());
}
