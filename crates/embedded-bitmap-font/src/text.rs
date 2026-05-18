use embedded_graphics::{
    Drawable, Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
};

use crate::{
    FontData, Glyph, TextStyle,
    layout::{TextFlow, TextRun, glyph_origin},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawableText<'a, C: PixelColor> {
    pub font_data: &'a FontData<'a>,
    pub text: &'a str,
    pub start_point: Point,
    pub style: TextStyle<C>,
}

impl<'a, C: PixelColor> DrawableText<'a, C> {
    pub const fn new(font_data: &'a FontData<'a>, text: &'a str, style: TextStyle<C>) -> Self {
        Self {
            font_data,
            text,
            start_point: Point::zero(),
            style,
        }
    }

    pub const fn at(mut self, start_point: Point) -> Self {
        self.start_point = start_point;
        self
    }

    pub fn measure(&self) -> Size {
        self.run().measure()
    }

    pub(crate) fn run(&self) -> TextRun<'a> {
        TextRun::new(
            self.text,
            self.start_point,
            self.style.cells,
            self.style.alignment,
            TextFlow::Horizontal,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalDrawableText<'a, C: PixelColor> {
    pub font_data: &'a FontData<'a>,
    pub text: &'a str,
    pub start_point: Point,
    pub style: TextStyle<C>,
}

impl<'a, C: PixelColor> VerticalDrawableText<'a, C> {
    pub const fn new(font_data: &'a FontData<'a>, text: &'a str, style: TextStyle<C>) -> Self {
        Self {
            font_data,
            text,
            start_point: Point::zero(),
            style,
        }
    }

    pub const fn at(mut self, start_point: Point) -> Self {
        self.start_point = start_point;
        self
    }

    pub fn measure(&self) -> Size {
        self.run().measure()
    }

    pub(crate) fn run(&self) -> TextRun<'a> {
        TextRun::new(
            self.text,
            self.start_point,
            self.style.cells,
            self.style.alignment,
            TextFlow::Vertical,
        )
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
        draw_text_run(target, self.font_data, self.run(), self.style.color)
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
        draw_text_run(target, self.font_data, self.run(), self.style.color)
    }
}

fn draw_text_run<D, C>(
    target: &mut D,
    font: &FontData<'_>,
    run: TextRun<'_>,
    color: C,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    run.for_each_cell(|ch, cell_origin, cell| {
        if let Some(glyph) = font.glyph(ch) {
            let origin = glyph_origin(font, glyph, cell_origin, cell, run.alignment());
            draw_glyph(target, font, glyph, origin, color)?;
        }
        Ok(())
    })
}

fn draw_glyph<D, C>(
    target: &mut D,
    font: &FontData<'_>,
    glyph: &Glyph,
    origin: Point,
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
                    origin + Point::new(x as i32, y as i32),
                    color,
                )))?;
            }
        }
    }
    Ok(())
}
