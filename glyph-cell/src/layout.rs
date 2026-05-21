use embedded_graphics_core::geometry::{Point, Size};

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
        let measure = self.measure_horizontal();
        let mut pen_y = self.start.y;

        for line in self.text.split('\n') {
            let (line_width, line_height) = self.measure_horizontal_line(line);
            let mut pen = Point::new(
                self.start.x + self.alignment.horizontal.offset(measure.width, line_width),
                pen_y,
            );

            for ch in line.chars() {
                let cell = self.cells.for_char(ch);
                visit(ch, pen, cell)?;
                pen.x += cell.width as i32;
            }

            pen_y += line_height as i32;
        }

        Ok(())
    }

    fn for_each_vertical_cell<E>(
        &self,
        visit: &mut impl FnMut(char, Point, Size) -> Result<(), E>,
    ) -> Result<(), E> {
        let measure = self.measure_vertical();
        let mut pen_x = self.start.x;

        for column in self.text.split('\n') {
            let (column_width, column_height) = self.measure_vertical_column(column);
            let mut pen_y = self.start.y
                + self
                    .alignment
                    .vertical
                    .offset(measure.height, column_height);

            for ch in column.chars() {
                let cell = self.cells.for_char(ch);
                let cell_x = pen_x + self.alignment.horizontal.offset(column_width, cell.width);
                visit(ch, Point::new(cell_x, pen_y), cell)?;
                pen_y += cell.height as i32;
            }

            pen_x += column_width as i32;
        }

        Ok(())
    }

    fn measure_horizontal_line(&self, line: &str) -> (u32, u32) {
        let mut line_width = 0u32;
        let mut line_height = self.cells.ascii.height;

        for ch in line.chars() {
            let cell = self.cells.for_char(ch);
            line_width = line_width.saturating_add(cell.width);
            line_height = line_height.max(cell.height);
        }

        (line_width, line_height)
    }

    fn measure_vertical_column(&self, column: &str) -> (u32, u32) {
        let mut column_width = self.cells.ascii.width;
        let mut column_height = 0u32;

        for ch in column.chars() {
            let cell = self.cells.for_char(ch);
            column_width = column_width.max(cell.width);
            column_height = column_height.saturating_add(cell.height);
        }

        (column_width, column_height)
    }
}

pub(crate) fn design_box_bounds(
    font: &FontData<'_>,
    cell_origin: Point,
    cell: Size,
) -> (Point, Size) {
    let size = Size::new(font.char_size as u32, font.char_size as u32);
    let (x, y) = Alignment::CENTER.offset(cell, size);
    (cell_origin + Point::new(x, y), size)
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

pub(crate) fn glyph_origin(
    font: &FontData<'_>,
    glyph: &Glyph,
    cell_origin: Point,
    cell: Size,
) -> Point {
    let (design_origin, _) = design_box_bounds(font, cell_origin, cell);
    Point::new(
        design_origin.x + glyph.x_offset as i32,
        design_origin.y + font.char_size as i32 - glyph.y_offset as i32,
    )
}
