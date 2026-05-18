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

fn style(ascii: Size, cjk: Size) -> TextStyle<BinaryColor> {
    TextStyle::new(BinaryColor::On).cells(ascii, cjk)
}

#[test]
fn finds_glyph_by_index_string() {
    let glyph = FONT.glyph('A').unwrap();
    assert_eq!(glyph.bitmap_offset, 0);
    assert_eq!(FONT.glyph('你').unwrap().width, 2);
    assert_eq!(FONT.glyph('Z'), None);
}

#[test]
fn measures_ascii_and_cjk_cells_differently() {
    let text = DrawableText::new(&FONT, "A你B", style(Size::new(4, 5), Size::new(6, 5)));

    let measured = text.measure();
    assert_eq!(measured, Size::new(14, 5));
}

#[test]
fn measures_multiline_horizontal_text_by_longest_line() {
    let text = DrawableText::new(&FONT, "A你\nBB", style(Size::new(4, 5), Size::new(6, 5)));

    assert_eq!(text.measure(), Size::new(10, 10));
}

#[test]
fn measures_vertical_text_by_columns() {
    let text = VerticalDrawableText::new(&FONT, "A你\nBB", style(Size::new(4, 5), Size::new(6, 5)));

    assert_eq!(text.measure(), Size::new(10, 10));
}

#[test]
fn draws_vertical_text_top_to_bottom_then_left_to_right() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = VerticalDrawableText::new(&FONT, "AB", style(Size::new(4, 5), Size::new(4, 5)));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[
        " # ", "# #", "###", "# #", "# #", "  #", "# #", " ##", "# #", " ##",
    ]);
}

#[test]
fn draws_vertical_multiline_text_in_columns() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = VerticalDrawableText::new(&FONT, "AB\nA", style(Size::new(4, 5), Size::new(4, 5)));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[
        " #   # ", "# # # #", "### ###", "# # # #", "# # # #", "  #    ", "# #    ", " ##    ",
        "# #    ", " ##    ",
    ]);
}

#[test]
fn draws_ascii_text_from_start_point() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = DrawableText::new(&FONT, "AB", style(Size::new(4, 5), Size::new(4, 5)));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[" #    #", "# # # #", "###  ##", "# # # #", "# #  ##"]);
}

#[test]
fn draws_glyphs_on_a_common_baseline() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = DrawableText::new(&FONT, "Ag", style(Size::new(4, 5), Size::new(4, 5)));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[" #   ", "# #  ", "###  ", "# #  ", "# # #"]);
}

#[test]
fn centers_font_design_square_inside_larger_cell() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = DrawableText::new(&FONT, "A", style(Size::new(7, 7), Size::new(7, 7)));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[
        "       ", "  #    ", " # #   ", " ###   ", " # #   ", " # #   ",
    ]);
}

#[test]
fn aligns_horizontal_text_design_square_to_top_left() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = DrawableText::new(
        &FONT,
        "A",
        style(Size::new(7, 7), Size::new(7, 7)).align(Alignment::TOP_LEFT),
    )
    .at(Point::new(0, 0));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[" #     ", "# #    ", "###    ", "# #    ", "# #    "]);
}

#[test]
fn aligns_horizontal_text_design_square_to_bottom_right() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = DrawableText::new(
        &FONT,
        "A",
        style(Size::new(7, 7), Size::new(7, 7)).align(Alignment::BOTTOM_RIGHT),
    )
    .at(Point::new(0, 0));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[
        "       ", "       ", "   #   ", "  # #  ", "  ###  ", "  # #  ", "  # #  ",
    ]);
}

#[test]
fn aligns_vertical_text_design_square_to_bottom_right() {
    let mut display = MockDisplay::<BinaryColor>::new();
    let text = VerticalDrawableText::new(
        &FONT,
        "A",
        style(Size::new(7, 7), Size::new(7, 7)).align(Alignment::BOTTOM_RIGHT),
    )
    .at(Point::new(0, 0));

    text.draw(&mut display).unwrap();

    display.assert_pattern(&[
        "       ", "       ", "   #   ", "  # #  ", "  ###  ", "  # #  ", "  # #  ",
    ]);
}

#[test]
fn positive_y_offset_moves_glyph_up() {
    const SHIFTED_FONT: FontData<'static> = FontData {
        index: "Ag",
        char_size: 5,
        bitmap: &BITMAP,
        glyphs: &[
            GLYPHS[0],
            Glyph {
                y_offset: GLYPHS[3].y_offset + 1,
                ..GLYPHS[3]
            },
        ],
    };

    let mut display = MockDisplay::<BinaryColor>::new();
    let text = DrawableText::new(&SHIFTED_FONT, "Ag", style(Size::new(4, 5), Size::new(4, 5)));
    text.draw(&mut display).unwrap();

    display.assert_pattern(&[" #   ", "# #  ", "###  ", "# # #", "# #  "]);
}
