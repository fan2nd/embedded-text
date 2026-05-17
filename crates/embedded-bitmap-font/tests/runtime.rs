use embedded_bitmap_font::*;
use embedded_graphics::{
    mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};

const ASCII_MAP: [u16; 128] = {
    let mut map = [u16::MAX; 128];
    map[b'A' as usize] = 0;
    map[b'B' as usize] = 1;
    map
};

const GLYPHS: [GlyphMetrics; 2] = [
    GlyphMetrics {
        codepoint: 'A',
        bitmap_offset: 0,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 5,
        x_advance: 4,
    },
    GlyphMetrics {
        codepoint: 'B',
        bitmap_offset: 2,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 5,
        x_advance: 4,
    },
];

// A: .#./#.#/###/#.#/#.#
// B: ##./#.#/##./#.#/##.
const BITMAP: [u8; 4] = [0b01010111, 0b11011010, 0b00110101, 0b11010111];

const FONT: BitmapFont<'static> = BitmapFont {
    size: 5,
    ascent: 5,
    descent: 0,
    line_gap: 0,
    bpp: BitsPerPixel::Bpp1,
    glyphs: &GLYPHS,
    bitmap: &BITMAP,
    ascii_map: Some(&ASCII_MAP),
    cmap: &[],
};

#[test]
fn finds_ascii_glyph_with_fast_map() {
    let glyph = FONT.glyph('A').unwrap();
    assert_eq!(glyph.codepoint, 'A');
    assert_eq!(glyph.width, 3);
    assert_eq!(FONT.glyph('Z'), None);
}

#[test]
fn decodes_1bpp_glyph_pixels() {
    let glyph = FONT.glyph('A').unwrap();
    assert!(!FONT.glyph_pixel(glyph, 0, 0));
    assert!(FONT.glyph_pixel(glyph, 1, 0));
    assert!(FONT.glyph_pixel(glyph, 0, 1));
    assert!(FONT.glyph_pixel(glyph, 2, 2));
    assert!(!FONT.glyph_pixel(glyph, 3, 0));
}

#[test]
fn draws_horizontal_ltr_text_to_embedded_graphics_target() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = BitmapText::new(
        "AB",
        &FONT,
        Rectangle::new(Point::new(0, 0), Size::new(8, 5)),
        TextStyle::new(BinaryColor::On),
    );

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[" #    # ", "# # # # ", "###  ## ", "# # # # ", "# #  ## "]);
}
