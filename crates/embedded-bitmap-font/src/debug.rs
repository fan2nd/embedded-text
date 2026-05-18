use embedded_graphics::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
};

use crate::{
    DrawableText, FontData, VerticalDrawableText,
    layout::{TextRun, design_box_bounds, glyph_box_bounds},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugBoxKind {
    Design,
    Cell,
    Glyph,
}

impl<'a, C: PixelColor> DrawableText<'a, C> {
    pub fn draw_debug_boxes<D>(&self, target: &mut D, kind: DebugBoxKind) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_text_boxes(target, self.font_data, self.run(), self.style.color, kind)
    }

    pub fn draw_design_boxes<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.draw_debug_boxes(target, DebugBoxKind::Design)
    }

    pub fn draw_cell_boxes<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.draw_debug_boxes(target, DebugBoxKind::Cell)
    }

    pub fn draw_glyph_boxes<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.draw_debug_boxes(target, DebugBoxKind::Glyph)
    }
}

impl<'a, C: PixelColor> VerticalDrawableText<'a, C> {
    pub fn draw_debug_boxes<D>(&self, target: &mut D, kind: DebugBoxKind) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_text_boxes(target, self.font_data, self.run(), self.style.color, kind)
    }

    pub fn draw_design_boxes<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.draw_debug_boxes(target, DebugBoxKind::Design)
    }

    pub fn draw_cell_boxes<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.draw_debug_boxes(target, DebugBoxKind::Cell)
    }

    pub fn draw_glyph_boxes<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.draw_debug_boxes(target, DebugBoxKind::Glyph)
    }
}

fn draw_text_boxes<D, C>(
    target: &mut D,
    font: &FontData<'_>,
    run: TextRun<'_>,
    color: C,
    kind: DebugBoxKind,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    run.for_each_cell(|ch, cell_origin, cell| {
        if let Some((origin, size)) = box_bounds(font, ch, cell_origin, cell, run.alignment(), kind)
        {
            draw_outline(target, origin, size, color)?;
        }
        Ok(())
    })
}

fn box_bounds(
    font: &FontData<'_>,
    ch: char,
    cell_origin: Point,
    cell: Size,
    alignment: crate::Alignment,
    kind: DebugBoxKind,
) -> Option<(Point, Size)> {
    match kind {
        DebugBoxKind::Design => Some(design_box_bounds(font, cell_origin, cell, alignment)),
        DebugBoxKind::Cell => Some((cell_origin, cell)),
        DebugBoxKind::Glyph => font
            .glyph(ch)
            .map(|glyph| glyph_box_bounds(font, glyph, cell_origin, cell, alignment)),
    }
}

fn draw_outline<D, C>(target: &mut D, origin: Point, size: Size, color: C) -> Result<(), D::Error>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    if size.width == 0 || size.height == 0 {
        return Ok(());
    }

    let right = origin.x + size.width as i32 - 1;
    let bottom = origin.y + size.height as i32 - 1;

    for x in origin.x..=right {
        target.draw_iter(core::iter::once(Pixel(Point::new(x, origin.y), color)))?;
        if bottom != origin.y {
            target.draw_iter(core::iter::once(Pixel(Point::new(x, bottom), color)))?;
        }
    }

    for y in (origin.y + 1)..bottom {
        target.draw_iter(core::iter::once(Pixel(Point::new(origin.x, y), color)))?;
        if right != origin.x {
            target.draw_iter(core::iter::once(Pixel(Point::new(right, y), color)))?;
        }
    }

    Ok(())
}
