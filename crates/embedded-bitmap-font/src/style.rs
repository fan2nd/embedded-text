use embedded_graphics::{geometry::Size, pixelcolor::PixelColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

impl Alignment {
    pub const TOP_LEFT: Self = Self::new(HorizontalAlignment::Left, VerticalAlignment::Top);
    pub const TOP_CENTER: Self = Self::new(HorizontalAlignment::Center, VerticalAlignment::Top);
    pub const TOP_RIGHT: Self = Self::new(HorizontalAlignment::Right, VerticalAlignment::Top);
    pub const MIDDLE_LEFT: Self = Self::new(HorizontalAlignment::Left, VerticalAlignment::Middle);
    pub const CENTER: Self = Self::new(HorizontalAlignment::Center, VerticalAlignment::Middle);
    pub const MIDDLE_RIGHT: Self = Self::new(HorizontalAlignment::Right, VerticalAlignment::Middle);
    pub const BOTTOM_LEFT: Self = Self::new(HorizontalAlignment::Left, VerticalAlignment::Bottom);
    pub const BOTTOM_CENTER: Self =
        Self::new(HorizontalAlignment::Center, VerticalAlignment::Bottom);
    pub const BOTTOM_RIGHT: Self = Self::new(HorizontalAlignment::Right, VerticalAlignment::Bottom);

    pub const fn new(horizontal: HorizontalAlignment, vertical: VerticalAlignment) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub(crate) fn offset(self, outer: Size, inner: Size) -> (i32, i32) {
        (
            horizontal_offset(outer.width, inner.width, self.horizontal),
            vertical_offset(outer.height, inner.height, self.vertical),
        )
    }
}

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

    pub const fn for_char(self, ch: char) -> Size {
        if ch.is_ascii() { self.ascii } else { self.cjk }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyle<C: PixelColor> {
    pub color: C,
    pub cells: CellSizes,
    pub alignment: Alignment,
}

impl<C: PixelColor> TextStyle<C> {
    pub const fn new(color: C) -> Self {
        Self {
            color,
            cells: CellSizes::new(Size::new(0, 0), Size::new(0, 0)),
            alignment: Alignment::CENTER,
        }
    }

    pub const fn cells(mut self, ascii: Size, cjk: Size) -> Self {
        self.cells = CellSizes::new(ascii, cjk);
        self
    }

    pub const fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

fn horizontal_offset(cell_width: u32, design_width: u32, alignment: HorizontalAlignment) -> i32 {
    match alignment {
        HorizontalAlignment::Left => 0,
        HorizontalAlignment::Center => (cell_width as i32 - design_width as i32) / 2,
        HorizontalAlignment::Right => cell_width as i32 - design_width as i32,
    }
}

fn vertical_offset(cell_height: u32, design_height: u32, alignment: VerticalAlignment) -> i32 {
    match alignment {
        VerticalAlignment::Top => 0,
        VerticalAlignment::Middle => (cell_height as i32 - design_height as i32) / 2,
        VerticalAlignment::Bottom => cell_height as i32 - design_height as i32,
    }
}
