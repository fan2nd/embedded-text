use embedded_bitmap_font_macros::bitmap_font;

bitmap_font! {
    pub static RANGE_FONT: embedded_bitmap_font::FontData<'static> = {
        path: "../../.ref/Cubic_11.ttf",
        size: 12,
        glyphs: "A00你",
        ranges: ['0'..='2']
    };
}

fn main() {
    assert_eq!(RANGE_FONT.char_size, 12);
    assert_eq!(RANGE_FONT.index, "012A你");
    assert!(RANGE_FONT.glyph('A').is_some());
    assert!(RANGE_FONT.glyph('0').is_some());
    assert!(RANGE_FONT.glyph('1').is_some());
    assert!(RANGE_FONT.glyph('2').is_some());
    assert!(RANGE_FONT.glyph('你').is_some());
    assert_eq!(RANGE_FONT.glyphs.len(), 5);
}
