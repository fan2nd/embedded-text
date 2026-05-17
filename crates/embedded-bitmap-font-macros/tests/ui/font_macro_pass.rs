use embedded_bitmap_font::BitmapFont;
use embedded_bitmap_font_macros::bitmap_font;

bitmap_font! {
    pub static DEMO_FONT: BitmapFont<'static> = {
        path: "../../.ref/Cubic_11.ttf",
        size: 12,
        glyphs: "AR你"
    };
}

fn main() {
    assert_eq!(DEMO_FONT.size, 12);
    assert!(DEMO_FONT.glyph('A').is_some());
    assert!(DEMO_FONT.glyph('R').is_some());
    assert!(DEMO_FONT.glyph('你').is_some());
}
