use embedded_bitmap_font::{DrawableText, FontData, Glyph};
use embedded_graphics::{mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*};

const BASE_FONT: FontData<'static> = FontData {
    index: "AB你",
    char_size: 5,
    bitmap: &BITMAP,
    glyphs: &BASE_GLYPHS,
};

const RAISED_FONT: FontData<'static> = FontData {
    index: "AB你",
    char_size: 5,
    bitmap: &BITMAP,
    glyphs: &RAISED_GLYPHS,
};

const BASE_GLYPHS: [Glyph; 3] = [
    Glyph {
        bitmap_offset: 0,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 0,
        x_advance: 4,
    },
    Glyph {
        bitmap_offset: 2,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 0,
        x_advance: 4,
    },
    Glyph {
        bitmap_offset: 4,
        width: 2,
        height: 5,
        x_offset: 0,
        y_offset: 0,
        x_advance: 2,
    },
];

const RAISED_GLYPHS: [Glyph; 3] = [
    Glyph {
        bitmap_offset: 0,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: -1,
        x_advance: 4,
    },
    Glyph {
        bitmap_offset: 2,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: -1,
        x_advance: 4,
    },
    Glyph {
        bitmap_offset: 4,
        width: 2,
        height: 5,
        x_offset: 0,
        y_offset: -1,
        x_advance: 2,
    },
];

// A: .#./#.#/###/#.#/#.#
// B: ##./#.#/##./#.#/##.
// 你: ##/##/##/##/## (placeholder bitmap for API demonstration)
const BITMAP: [u8; 6] = [
    0b01010111, 0b11011010, 0b00110101, 0b11010111, 0b11000011, 0b11000011,
];

fn main() {
    let mut display = MockDisplay::<BinaryColor>::new();

    let base = DrawableText::new(
        &BASE_FONT,
        "A你B",
        Point::new(0, 0),
        Size::new(4, 5),
        Size::new(6, 5),
        BinaryColor::On,
    );
    let raised = DrawableText::new(
        &RAISED_FONT,
        "A你B",
        Point::new(0, 8),
        Size::new(4, 5),
        Size::new(6, 5),
        BinaryColor::On,
    );

    base.draw(&mut display).unwrap();
    raised.draw(&mut display).unwrap();

    println!("base measure: {:?}", base.measure());
    println!("raised measure: {:?}", raised.measure());
    println!("{display:#?}");
}
