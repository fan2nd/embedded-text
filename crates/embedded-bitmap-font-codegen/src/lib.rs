use std::{fmt::Write, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlyphBitmap {
    Bpp1(Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapGlyph {
    pub codepoint: char,
    pub width: u16,
    pub height: u16,
    pub x_offset: i16,
    pub y_offset: i16,
    pub x_advance: i16,
    pub bitmap: GlyphBitmap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenFont {
    pub ident: String,
    pub index: String,
    pub size: u16,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
    pub glyphs: Vec<BitmapGlyph>,
}

pub struct FontWriter {
    font: CodegenFont,
}

impl FontWriter {
    pub fn new(font: CodegenFont) -> Self {
        Self { font }
    }

    pub fn write_rust_source(&self) -> Result<String, std::fmt::Error> {
        let mut glyphs = self.font.glyphs.clone();
        glyphs.sort_by_key(|glyph| glyph.codepoint);

        let mut bitmap = Vec::new();
        let mut metrics = Vec::new();
        for glyph in &glyphs {
            let bitmap_offset = bitmap.len() as u32;
            match &glyph.bitmap {
                GlyphBitmap::Bpp1(pixels) => pack_bpp1(pixels, &mut bitmap),
            }
            metrics.push((glyph, bitmap_offset));
        }

        let mut source = String::new();
        writeln!(
            source,
            "const GLYPHS: [embedded_bitmap_font::Glyph; {}] = [",
            glyphs.len()
        )?;
        for (glyph, bitmap_offset) in metrics {
            writeln!(source, "    embedded_bitmap_font::Glyph {{")?;
            writeln!(source, "        bitmap_offset: {bitmap_offset},")?;
            writeln!(source, "        width: {},", glyph.width)?;
            writeln!(source, "        height: {},", glyph.height)?;
            writeln!(source, "        x_offset: {},", glyph.x_offset)?;
            writeln!(source, "        y_offset: {},", glyph.y_offset)?;
            writeln!(source, "        x_advance: {},", glyph.x_advance)?;
            writeln!(source, "    }},")?;
        }
        writeln!(source, "];\n")?;

        writeln!(source, "const BITMAP: [u8; {}] = [", bitmap.len())?;
        for byte in &bitmap {
            writeln!(source, "    0b{byte:08b},")?;
        }
        writeln!(source, "];\n")?;

        let index: String = glyphs.iter().map(|glyph| glyph.codepoint).collect();
        writeln!(
            source,
            "pub static {}: embedded_bitmap_font::FontData<'static> = embedded_bitmap_font::FontData {{",
            self.font.ident
        )?;
        writeln!(source, "    index: {:?},", index)?;
        writeln!(source, "    char_size: {},", self.font.size as usize)?;
        writeln!(source, "    bitmap: &BITMAP,")?;
        writeln!(source, "    glyphs: &GLYPHS,")?;
        writeln!(source, "}};")?;

        Ok(source)
    }

    pub fn write_rust_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let source = self
            .write_rust_source()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "format error"))?;
        std::fs::write(path, source)
    }
}

fn pack_bpp1(pixels: &[bool], output: &mut Vec<u8>) {
    let mut byte = 0u8;
    for (index, pixel) in pixels.iter().enumerate() {
        if *pixel {
            byte |= 1 << (7 - index % 8);
        }
        if index % 8 == 7 {
            output.push(byte);
            byte = 0;
        }
    }

    if !pixels.len().is_multiple_of(8) {
        output.push(byte);
    }
}
