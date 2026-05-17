use embedded_bitmap_font::{DrawableText, FontData};
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
    Point::new(0, 0),
    Size::new(4, 8),
    Size::new(8, 9),
    BinaryColor::On,
);

fn main() {}
