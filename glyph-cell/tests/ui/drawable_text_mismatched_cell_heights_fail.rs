use glyph_cell::{DrawableText, FontData, TextStyle};
use embedded_graphics::{geometry::Size, pixelcolor::BinaryColor, prelude::Point};

const FONT: FontData<'static> = FontData {
    index: "",
    char_size: 8,
    bitmap: &[],
    glyphs: &[],
};

const _TEXT: DrawableText<'static, BinaryColor> = DrawableText::new(
    &FONT,
    "",
    TextStyle::new(BinaryColor::On).cells(Size::new(4, 8), Size::new(8, 9)),
)
.at(Point::new(0, 0));

fn main() {}
