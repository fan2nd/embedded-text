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
        let mut width = 0u32;
        let mut height = 0u32;
        for ch in self.text.chars() {
            let cell = self.cell_size_for(ch);
            width = width.saturating_add(cell.width);
            height = height.max(cell.height);
        }
        Size::new(width, height)
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
        let pen_y = self.start_point.y;

        for ch in self.text.chars() {
            let cell = self.cell_size_for(ch);
            let Some(glyph) = self.font_data.glyph(ch) else {
                pen_x += cell.width as i32;
                continue;
            };

            let draw_pos = Point::new(
                pen_x + glyph.x_offset as i32,
                pen_y + cell.height as i32 - glyph.y_offset as i32,
            );
            draw_glyph(target, self.font_data, glyph, draw_pos, self.color)?;
            pen_x += cell.width as i32;
        }

        Ok(())
    }
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
