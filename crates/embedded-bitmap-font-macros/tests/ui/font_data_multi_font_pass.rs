use embedded_bitmap_font::FontData;
use embedded_bitmap_font_macros::font_data;

const FONT18: FontData<'static> = font_data! {
    size: 18,
    path: "../../.ref/Cubic_11.ttf",
    index: "AB",
    path: "../../.ref/Cubic_11.ttf",
    index: "你",
};

fn main() {
    assert_eq!(FONT18.char_size, 18);
    assert_eq!(FONT18.index, "AB你");
    assert_eq!(FONT18.glyphs.len(), 3);

    let ascii_only_font: FontData<'static> = font_data! {
        size: 18,
        path: "../../.ref/Cubic_11.ttf",
        index: "A",
    };

    let mixed_a = FONT18.glyph('A').expect("mixed A exists");
    let ascii_a = ascii_only_font.glyph('A').expect("ASCII-only A exists");
    assert_eq!(mixed_a.y_offset, ascii_a.y_offset);
}
