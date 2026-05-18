#![cfg(feature = "debug")]

use embedded_bitmap_text::*;
use embedded_graphics::{mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*};

const FONT: FontData<'static> = FontData {
    index: "AB你g",
    char_size: 5,
    bitmap: &BITMAP,
    glyphs: &GLYPHS,
};

const GLYPHS: [Glyph; 4] = [
    Glyph {
        bitmap_offset: 0,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 5,
        x_advance: 4,
    },
    Glyph {
        bitmap_offset: 2,
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 5,
        x_advance: 4,
    },
    Glyph {
        bitmap_offset: 4,
        width: 2,
        height: 5,
        x_offset: 0,
        y_offset: 5,
        x_advance: 2,
    },
    Glyph {
        bitmap_offset: 6,
        width: 1,
        height: 1,
        x_offset: 0,
        y_offset: 1,
        x_advance: 1,
    },
];

// A: .#./#.#/###/#.#/#.#
// B: ##./#.#/##./#.#/##.
// 你: ##/##/##/##/## (placeholder bitmap for API test)
// g: #
const BITMAP: [u8; 7] = [
    0b01010111, 0b11011010, 0b00110101, 0b11010111, 0b11000011, 0b11000011, 0b10000000,
];

fn style() -> TextStyle<BinaryColor> {
    TextStyle::new(BinaryColor::On).cells(Size::new(4, 5), Size::new(6, 5))
}

fn sample_text() -> DrawableText<'static, BinaryColor> {
    DrawableText::new(&FONT, "A你\nB", style()).at(Point::new(1, 1))
}

fn sample_vertical_text() -> VerticalDrawableText<'static, BinaryColor> {
    VerticalDrawableText::new(&FONT, "A你\nB", style()).at(Point::new(1, 1))
}

#[test]
fn draws_design_debug_boxes_for_each_horizontal_character() {
    let mut display = MockDisplay::<BinaryColor>::new();
    display.set_allow_overdraw(true);

    sample_text().draw_design_boxes(&mut display).unwrap();

    display.assert_pattern(&[
        "              ",
        " #########    ",
        " #   #   #    ",
        " #   #   #    ",
        " #   #   #    ",
        " #########    ",
        " #####        ",
        " #   #        ",
        " #   #        ",
        " #   #        ",
        " #####        ",
    ]);
}

#[test]
fn draws_cell_debug_boxes_for_each_horizontal_character() {
    let mut display = MockDisplay::<BinaryColor>::new();
    display.set_allow_overdraw(true);

    sample_text().draw_cell_boxes(&mut display).unwrap();

    display.assert_pattern(&[
        "             ",
        " ##########  ",
        " #  ##    #  ",
        " #  ##    #  ",
        " #  ##    #  ",
        " ##########  ",
        " ####        ",
        " #  #        ",
        " #  #        ",
        " #  #        ",
        " ####        ",
    ]);
}

#[test]
fn draws_glyph_debug_boxes_for_each_horizontal_character() {
    let mut display = MockDisplay::<BinaryColor>::new();
    display.set_allow_overdraw(true);

    sample_text().draw_glyph_boxes(&mut display).unwrap();

    display.assert_pattern(&[
        "       ", " ### ##", " # # ##", " # # ##", " # # ##", " ### ##", " ###   ", " # #   ",
        " # #   ", " # #   ", " ###   ",
    ]);
}

#[test]
fn draws_design_debug_boxes_for_each_vertical_character() {
    let mut display = MockDisplay::<BinaryColor>::new();
    display.set_allow_overdraw(true);

    sample_vertical_text()
        .draw_design_boxes(&mut display)
        .unwrap();

    display.assert_pattern(&[
        "             ",
        " ##### ##### ",
        " #   # #   # ",
        " #   # #   # ",
        " #   # #   # ",
        " ##### ##### ",
        " #####       ",
        " #   #       ",
        " #   #       ",
        " #   #       ",
        " #####       ",
    ]);
}

#[test]
fn draws_cell_debug_boxes_for_each_vertical_character() {
    let mut display = MockDisplay::<BinaryColor>::new();
    display.set_allow_overdraw(true);

    sample_vertical_text()
        .draw_cell_boxes(&mut display)
        .unwrap();

    display.assert_pattern(&[
        "            ",
        " ####  #### ",
        " #  #  #  # ",
        " #  #  #  # ",
        " #  #  #  # ",
        " ####  #### ",
        " ######     ",
        " #    #     ",
        " #    #     ",
        " #    #     ",
        " ######     ",
    ]);
}

#[test]
fn draws_glyph_debug_boxes_for_each_vertical_character() {
    let mut display = MockDisplay::<BinaryColor>::new();
    display.set_allow_overdraw(true);

    sample_vertical_text()
        .draw_glyph_boxes(&mut display)
        .unwrap();

    display.assert_pattern(&[
        "          ",
        " ###   ###",
        " # #   # #",
        " # #   # #",
        " # #   # #",
        " ###   ###",
        " ##       ",
        " ##       ",
        " ##       ",
        " ##       ",
        " ##       ",
    ]);
}
