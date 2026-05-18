use embedded_graphics::{
    Drawable, Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
};

use crate::{FontData, Glyph};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellSizes {
    pub ascii: Size,
    pub cjk: Size,
}

impl CellSizes {
    pub const fn new(ascii: Size, cjk: Size) -> Self {
        assert!(
            ascii.height == cjk.height,
            "CellSizes ascii and cjk heights must match"
        );
        Self { ascii, cjk }
    }

    pub const fn for_char(&self, ch: char) -> Size {
        if ch.is_ascii() { self.ascii } else { self.cjk }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawableText<'a, C: PixelColor> {
    pub font_data: &'a FontData<'a>,
    pub text: &'a str,
    pub start_point: Point,
    pub cell_sizes: CellSizes,
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
        Self::with_cell_sizes(
            font_data,
            text,
            start_point,
            CellSizes::new(ascii_cell_size, cjk_cell_size),
            color,
        )
    }

    pub const fn with_cell_sizes(
        font_data: &'a FontData<'a>,
        text: &'a str,
        start_point: Point,
        cell_sizes: CellSizes,
        color: C,
    ) -> Self {
        Self {
            font_data,
            text,
            start_point,
            cell_sizes,
            color,
        }
    }

    pub fn measure(&self) -> Size {
        self.run().measure()
    }

    pub(crate) fn run(&self) -> TextRun<'a> {
        TextRun::new(
            self.text,
            self.start_point,
            self.cell_sizes,
            TextFlow::Horizontal,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalDrawableText<'a, C: PixelColor> {
    pub font_data: &'a FontData<'a>,
    pub text: &'a str,
    pub start_point: Point,
    pub cell_sizes: CellSizes,
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
        Self::with_cell_sizes(
            font_data,
            text,
            start_point,
            CellSizes::new(ascii_cell_size, cjk_cell_size),
            color,
        )
    }

    pub const fn with_cell_sizes(
        font_data: &'a FontData<'a>,
        text: &'a str,
        start_point: Point,
        cell_sizes: CellSizes,
        color: C,
    ) -> Self {
        Self {
            font_data,
            text,
            start_point,
            cell_sizes,
            color,
        }
    }

    pub fn measure(&self) -> Size {
        self.run().measure()
    }

    pub(crate) fn run(&self) -> TextRun<'a> {
        TextRun::new(
            self.text,
            self.start_point,
            self.cell_sizes,
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
        draw_text_run(target, self.font_data, self.run(), self.color)
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
        draw_text_run(target, self.font_data, self.run(), self.color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextFlow {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextRun<'a> {
    text: &'a str,
    start_point: Point,
    cell_sizes: CellSizes,
    flow: TextFlow,
}

impl<'a> TextRun<'a> {
    const fn new(text: &'a str, start_point: Point, cell_sizes: CellSizes, flow: TextFlow) -> Self {
        Self {
            text,
            start_point,
            cell_sizes,
            flow,
        }
    }

    pub(crate) fn measure(&self) -> Size {
        match self.flow {
            TextFlow::Horizontal => self.measure_horizontal(),
            TextFlow::Vertical => self.measure_vertical(),
        }
    }

    pub(crate) fn for_each_cell<E>(
        &self,
        mut visit: impl FnMut(char, Point, Size) -> Result<(), E>,
    ) -> Result<(), E> {
        match self.flow {
            TextFlow::Horizontal => self.for_each_horizontal_cell(&mut visit),
            TextFlow::Vertical => self.for_each_vertical_cell(&mut visit),
        }
    }

    fn measure_horizontal(&self) -> Size {
        let mut max_width = 0u32;
        let mut total_height = 0u32;
        let mut line_width = 0u32;
        let mut line_height = 0u32;

        for ch in self.text.chars() {
            if ch == '\n' {
                max_width = max_width.max(line_width);
                total_height = total_height.saturating_add(line_height);
                line_width = 0;
                line_height = 0;
                continue;
            }

            let cell = self.cell_sizes.for_char(ch);
            line_width = line_width.saturating_add(cell.width);
            line_height = line_height.max(cell.height);
        }

        max_width = max_width.max(line_width);
        total_height = total_height.saturating_add(line_height);
        Size::new(max_width, total_height)
    }

    fn measure_vertical(&self) -> Size {
        let mut total_width = 0u32;
        let mut max_height = 0u32;
        let mut column_width = 0u32;
        let mut column_height = 0u32;

        for ch in self.text.chars() {
            if ch == '\n' {
                total_width = total_width.saturating_add(column_width);
                max_height = max_height.max(column_height);
                column_width = 0;
                column_height = 0;
                continue;
            }

            let cell = self.cell_sizes.for_char(ch);
            column_width = column_width.max(cell.width);
            column_height = column_height.saturating_add(cell.height);
        }

        total_width = total_width.saturating_add(column_width);
        max_height = max_height.max(column_height);
        Size::new(total_width, max_height)
    }

    fn for_each_horizontal_cell<E>(
        &self,
        visit: &mut impl FnMut(char, Point, Size) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut pen_x = self.start_point.x;
        let mut pen_y = self.start_point.y;
        let mut line_height = self.cell_sizes.ascii.height;

        for ch in self.text.chars() {
            if ch == '\n' {
                pen_x = self.start_point.x;
                pen_y += line_height as i32;
                line_height = self.cell_sizes.ascii.height;
                continue;
            }

            let cell = self.cell_sizes.for_char(ch);
            line_height = line_height.max(cell.height);
            visit(ch, Point::new(pen_x, pen_y), cell)?;
            pen_x += cell.width as i32;
        }

        Ok(())
    }

    fn for_each_vertical_cell<E>(
        &self,
        visit: &mut impl FnMut(char, Point, Size) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut pen_x = self.start_point.x;
        let mut pen_y = self.start_point.y;
        let mut column_width = self.cell_sizes.ascii.width;

        for ch in self.text.chars() {
            if ch == '\n' {
                pen_x += column_width as i32;
                pen_y = self.start_point.y;
                column_width = self.cell_sizes.ascii.width;
                continue;
            }

            let cell = self.cell_sizes.for_char(ch);
            column_width = column_width.max(cell.width);
            visit(ch, Point::new(pen_x, pen_y), cell)?;
            pen_y += cell.height as i32;
        }

        Ok(())
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
            draw_glyph_in_cell(target, font, glyph, cell_origin, cell, color)?;
        }
        Ok(())
    })
}

pub(crate) fn design_box_bounds(
    font: &FontData<'_>,
    cell_origin: Point,
    cell: Size,
) -> (Point, Size) {
    let size = Size::new(font.char_size as u32, font.char_size as u32);
    let origin = Point::new(
        cell_origin.x + (cell.width as i32 - font.char_size as i32) / 2,
        cell_origin.y + (cell.height as i32 - font.char_size as i32) / 2,
    );
    (origin, size)
}

#[cfg(feature = "debug")]
pub(crate) fn glyph_box_bounds(
    font: &FontData<'_>,
    glyph: &Glyph,
    cell_origin: Point,
    cell: Size,
) -> (Point, Size) {
    (
        glyph_origin(font, glyph, cell_origin, cell),
        Size::new(glyph.width as u32, glyph.height as u32),
    )
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
    let draw_pos = glyph_origin(font, glyph, cell_origin, cell);
    draw_glyph(target, font, glyph, draw_pos, color)
}

fn glyph_origin(font: &FontData<'_>, glyph: &Glyph, cell_origin: Point, cell: Size) -> Point {
    let (design_origin, _) = design_box_bounds(font, cell_origin, cell);
    Point::new(
        design_origin.x + glyph.x_offset as i32,
        design_origin.y + font.char_size as i32 - glyph.y_offset as i32,
    )
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
