use embedded_graphics::geometry::{Point, Size};

use crate::{Alignment, CellSizes, FontData, Glyph};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextFlow {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextRun<'a> {
    text: &'a str,
    start: Point,
    cells: CellSizes,
    alignment: Alignment,
    flow: TextFlow,
}

impl<'a> TextRun<'a> {
    pub(crate) const fn new(
        text: &'a str,
        start: Point,
        cells: CellSizes,
        alignment: Alignment,
        flow: TextFlow,
    ) -> Self {
        Self {
            text,
            start,
            cells,
            alignment,
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

    pub(crate) const fn alignment(&self) -> Alignment {
        self.alignment
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
            } else {
                let cell = self.cells.for_char(ch);
                line_width = line_width.saturating_add(cell.width);
                line_height = line_height.max(cell.height);
            }
        }

        Size::new(
            max_width.max(line_width),
            total_height.saturating_add(line_height),
        )
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
            } else {
                let cell = self.cells.for_char(ch);
                column_width = column_width.max(cell.width);
                column_height = column_height.saturating_add(cell.height);
            }
        }

        Size::new(
            total_width.saturating_add(column_width),
            max_height.max(column_height),
        )
    }

    fn for_each_horizontal_cell<E>(
        &self,
        visit: &mut impl FnMut(char, Point, Size) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut pen = self.start;
        let mut line_height = self.cells.ascii.height;

        for ch in self.text.chars() {
            if ch == '\n' {
                pen.x = self.start.x;
                pen.y += line_height as i32;
                line_height = self.cells.ascii.height;
            } else {
                let cell = self.cells.for_char(ch);
                line_height = line_height.max(cell.height);
                visit(ch, pen, cell)?;
                pen.x += cell.width as i32;
            }
        }

        Ok(())
    }

    fn for_each_vertical_cell<E>(
        &self,
        visit: &mut impl FnMut(char, Point, Size) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut pen = self.start;
        let mut column_width = self.cells.ascii.width;

        for ch in self.text.chars() {
            if ch == '\n' {
                pen.x += column_width as i32;
                pen.y = self.start.y;
                column_width = self.cells.ascii.width;
            } else {
                let cell = self.cells.for_char(ch);
                column_width = column_width.max(cell.width);
                visit(ch, pen, cell)?;
                pen.y += cell.height as i32;
            }
        }

        Ok(())
    }
}

pub(crate) fn design_box_bounds(
    font: &FontData<'_>,
    cell_origin: Point,
    cell: Size,
    alignment: Alignment,
) -> (Point, Size) {
    let size = Size::new(font.char_size as u32, font.char_size as u32);
    let (x, y) = alignment.offset(cell, size);
    (cell_origin + Point::new(x, y), size)
}

#[cfg(feature = "debug")]
pub(crate) fn glyph_box_bounds(
    font: &FontData<'_>,
    glyph: &Glyph,
    cell_origin: Point,
    cell: Size,
    alignment: Alignment,
) -> (Point, Size) {
    (
        glyph_origin(font, glyph, cell_origin, cell, alignment),
        Size::new(glyph.width as u32, glyph.height as u32),
    )
}

pub(crate) fn glyph_origin(
    font: &FontData<'_>,
    glyph: &Glyph,
    cell_origin: Point,
    cell: Size,
    alignment: Alignment,
) -> Point {
    let (design_origin, _) = design_box_bounds(font, cell_origin, cell, alignment);
    Point::new(
        design_origin.x + glyph.x_offset as i32,
        design_origin.y + font.char_size as i32 - glyph.y_offset as i32,
    )
}
