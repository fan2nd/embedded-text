use embedded_bitmap_font_codegen::{BitmapGlyph, CodegenFont, FontWriter, GlyphBitmap};

#[test]
fn writes_static_bitmap_font_rust_source() {
    let font = CodegenFont {
        ident: "TEST_FONT".into(),
        size: 5,
        ascent: 5,
        descent: 0,
        line_gap: 0,
        glyphs: vec![BitmapGlyph {
            codepoint: 'A',
            width: 3,
            height: 5,
            x_offset: 0,
            y_offset: 5,
            x_advance: 4,
            bitmap: GlyphBitmap::Bpp1(vec![
                false, true, false, true, false, true, true, true, true, true, false, true, true,
                false, true,
            ]),
        }],
    };

    let source = FontWriter::new(font).write_rust_source().unwrap();

    assert!(source.contains("pub static TEST_FONT: BitmapFont<'static>"));
    assert!(source.contains("codepoint: 'A'"));
    assert!(source.contains("bitmap_offset: 0"));
    assert!(source.contains("map[b'A' as usize] = 0"));
    assert!(source.contains("0b01010111"));
}
