#![no_std]

use embedded_graphics::{
    Drawable, Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitsPerPixel {
    Bpp1,
    Bpp4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub bitmap_offset: u32,
    pub width: u16,
    pub height: u16,
    pub x_offset: i16,
    pub y_offset: i16,
    pub x_advance: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontData<'a> {
    pub index: &'a str,
    pub char_size: usize,
    pub bitmap: &'a [u8],
    pub glyphs: &'a [Glyph],
}

impl<'a> FontData<'a> {
    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        self.index
            .chars()
            .position(|candidate| candidate == ch)
            .and_then(|index| self.glyphs.get(index))
    }

    pub fn glyph_pixel(&self, glyph: &Glyph, x: u16, y: u16) -> bool {
        if x >= glyph.width || y >= glyph.height {
            return false;
        }

        let pixel_index = y as usize * glyph.width as usize + x as usize;
        let bit_index = glyph.bitmap_offset as usize * 8 + pixel_index;
        let byte = self.bitmap.get(bit_index / 8).copied().unwrap_or(0);
        let bit = 7 - (bit_index % 8);
        byte & (1 << bit) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawableText<'a, C: PixelColor> {
    pub font_data: &'a FontData<'a>,
    pub text: &'a str,
    pub ascii_cell_size: Size,
    pub cjk_cell_size: Size,
    pub start_point: Point,
    pub color: C,
}

impl<'a, C: PixelColor> DrawableText<'a, C> {
    pub const fn new(
        font_data: &'a FontData<'a>,
        text: &'a str,
        start_point: Point,
        ascii_cell_size: Size,
        cjk_cell_size: Size,
        color: C,
    ) -> Self {
        assert!(
            ascii_cell_size.height == cjk_cell_size.height,
            "DrawableText ascii_cell_size and cjk_cell_size heights must match"
        );
        Self {
            font_data,
            text,
            ascii_cell_size,
            cjk_cell_size,
            start_point,
            color,
        }
    }

    pub fn measure(&self) -> Size {
        measure_horizontal(self.text, |ch| self.cell_size_for(ch))
    }

    fn cell_size_for(&self, ch: char) -> Size {
        if ch.is_ascii() {
            self.ascii_cell_size
        } else {
            self.cjk_cell_size
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalDrawableText<'a, C: PixelColor> {
    pub font_data: &'a FontData<'a>,
    pub text: &'a str,
    pub ascii_cell_size: Size,
    pub cjk_cell_size: Size,
    pub start_point: Point,
    pub color: C,
}

impl<'a, C: PixelColor> VerticalDrawableText<'a, C> {
    pub const fn new(
        font_data: &'a FontData<'a>,
        text: &'a str,
        start_point: Point,
        ascii_cell_size: Size,
        cjk_cell_size: Size,
        color: C,
    ) -> Self {
        assert!(
            ascii_cell_size.height == cjk_cell_size.height,
            "VerticalDrawableText ascii_cell_size and cjk_cell_size heights must match"
        );
        Self {
            font_data,
            text,
            ascii_cell_size,
            cjk_cell_size,
            start_point,
            color,
        }
    }

    pub fn measure(&self) -> Size {
        measure_vertical(self.text, |ch| self.cell_size_for(ch))
    }

    fn cell_size_for(&self, ch: char) -> Size {
        if ch.is_ascii() {
            self.ascii_cell_size
        } else {
            self.cjk_cell_size
        }
    }
}

impl<'a, C> Drawable for DrawableText<'a, C>
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let mut pen_x = self.start_point.x;
        let mut pen_y = self.start_point.y;
        let mut line_height = self.ascii_cell_size.height;

        for ch in self.text.chars() {
            if ch == '\n' {
                pen_x = self.start_point.x;
                pen_y += line_height as i32;
                line_height = self.ascii_cell_size.height;
                continue;
            }

            let cell = self.cell_size_for(ch);
            line_height = line_height.max(cell.height);
            let Some(glyph) = self.font_data.glyph(ch) else {
                pen_x += cell.width as i32;
                continue;
            };

            draw_glyph_in_cell(
                target,
                self.font_data,
                glyph,
                Point::new(pen_x, pen_y),
                cell,
                self.color,
            )?;
            pen_x += cell.width as i32;
        }

        Ok(())
    }
}

impl<'a, C> Drawable for VerticalDrawableText<'a, C>
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let mut pen_x = self.start_point.x;
        let mut pen_y = self.start_point.y;
        let mut column_width = self.ascii_cell_size.width;

        for ch in self.text.chars() {
            if ch == '\n' {
                pen_x += column_width as i32;
                pen_y = self.start_point.y;
                column_width = self.ascii_cell_size.width;
                continue;
            }

            let cell = self.cell_size_for(ch);
            column_width = column_width.max(cell.width);
            let Some(glyph) = self.font_data.glyph(ch) else {
                pen_y += cell.height as i32;
                continue;
            };

            draw_glyph_in_cell(
                target,
                self.font_data,
                glyph,
                Point::new(pen_x, pen_y),
                cell,
                self.color,
            )?;
            pen_y += cell.height as i32;
        }

        Ok(())
    }
}

fn measure_horizontal(text: &str, mut cell_size_for: impl FnMut(char) -> Size) -> Size {
    let mut max_width = 0u32;
    let mut total_height = 0u32;
    let mut line_width = 0u32;
    let mut line_height = 0u32;

    for ch in text.chars() {
        if ch == '\n' {
            max_width = max_width.max(line_width);
            total_height = total_height.saturating_add(line_height);
            line_width = 0;
            line_height = 0;
            continue;
        }

        let cell = cell_size_for(ch);
        line_width = line_width.saturating_add(cell.width);
        line_height = line_height.max(cell.height);
    }

    max_width = max_width.max(line_width);
    total_height = total_height.saturating_add(line_height);
    Size::new(max_width, total_height)
}

fn measure_vertical(text: &str, mut cell_size_for: impl FnMut(char) -> Size) -> Size {
    let mut total_width = 0u32;
    let mut max_height = 0u32;
    let mut column_width = 0u32;
    let mut column_height = 0u32;

    for ch in text.chars() {
        if ch == '\n' {
            total_width = total_width.saturating_add(column_width);
            max_height = max_height.max(column_height);
            column_width = 0;
            column_height = 0;
            continue;
        }

        let cell = cell_size_for(ch);
        column_width = column_width.max(cell.width);
        column_height = column_height.saturating_add(cell.height);
    }

    total_width = total_width.saturating_add(column_width);
    max_height = max_height.max(column_height);
    Size::new(total_width, max_height)
}

fn draw_glyph_in_cell<D, C>(
    target: &mut D,
    font: &FontData<'_>,
    glyph: &Glyph,
    cell_origin: Point,
    cell: Size,
    color: C,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    let source_square_size = font.char_size as i32;
    // Treat every generated glyph as living in the font's original
    // `char_size x char_size` design square. Layout cells only decide
    // where that square is placed: first center the source square in
    // the target ASCII/CJK cell, then apply glyph offsets inside the
    // square to get the actual bitmap origin. `Glyph::y_offset` is a
    // screen-space top offset here: larger values move the glyph down.
    let square_x = cell_origin.x + (cell.width as i32 - source_square_size) / 2;
    let square_y = cell_origin.y + (cell.height as i32 - source_square_size) / 2;
    let draw_pos = Point::new(
        square_x + glyph.x_offset as i32,
        square_y + glyph.y_offset as i32,
    );
    draw_glyph(target, font, glyph, draw_pos, color)
}

fn draw_glyph<D, C>(
    target: &mut D,
    font: &FontData<'_>,
    glyph: &Glyph,
    pos: Point,
    color: C,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    for y in 0..glyph.height {
        for x in 0..glyph.width {
            if font.glyph_pixel(glyph, x, y) {
                target.draw_iter(core::iter::once(Pixel(
                    pos + Point::new(x as i32, y as i32),
                    color,
                )))?;
            }
        }
    }
    Ok(())
}
