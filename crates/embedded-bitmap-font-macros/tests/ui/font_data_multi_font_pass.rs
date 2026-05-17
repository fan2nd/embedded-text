use embedded_bitmap_font::FontData;
use embedded_bitmap_font_macros::font_data;

const FONT18: FontData<'static> = font_data! {
    size: 18,
    path: "../../.ref/Cubic_11.ttf",
    index: "AB",
    y_offset: 9,
    path: "../../.ref/Cubic_11.ttf",
    index: "你",
};

fn main() {
    assert_eq!(FONT18.char_size, 18);
    assert_eq!(FONT18.index, "AB你");
    assert_eq!(FONT18.glyphs.len(), 3);

    let font_without_offset: FontData<'static> = font_data! {
        size: 18,
        path: "../../.ref/Cubic_11.ttf",
        index: "A",
    };

    let shifted = FONT18.glyph('A').expect("shifted A exists");
    let original = font_without_offset.glyph('A').expect("original A exists");
    assert_eq!(shifted.y_offset, original.y_offset + 9);

    let negatively_shifted: FontData<'static> = font_data! {
        size: 18,
        path: "../../.ref/Cubic_11.ttf",
        index: "A",
        y_offset: -4,
    };

    assert_eq!(
        negatively_shifted
            .glyph('A')
            .expect("negative shifted A exists")
            .y_offset,
        original.y_offset - 4
    );
}
