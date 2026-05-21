use embedded_graphics_core::{geometry::Size, pixelcolor::PixelColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

impl HorizontalAlignment {
    pub(crate) fn offset(self, outer_width: u32, inner_width: u32) -> i32 {
        match self {
            Self::Left => 0,
            Self::Center => (outer_width as i32 - inner_width as i32) / 2,
            Self::Right => outer_width as i32 - inner_width as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

impl VerticalAlignment {
    pub(crate) fn offset(self, outer_height: u32, inner_height: u32) -> i32 {
        match self {
            Self::Top => 0,
            Self::Middle => (outer_height as i32 - inner_height as i32) / 2,
            Self::Bottom => outer_height as i32 - inner_height as i32,
        }
    }
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
            self.horizontal.offset(outer.width, inner.width),
            self.vertical.offset(outer.height, inner.height),
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

    pub const fn with_ascii(mut self, ascii: Size) -> Self {
        assert!(
            self.cjk.height == 0 || ascii.height == self.cjk.height,
            "CellSizes ascii and cjk heights must match"
        );
        self.ascii = ascii;
        self
    }

    pub const fn with_cjk(mut self, cjk: Size) -> Self {
        assert!(
            self.ascii.height == 0 || self.ascii.height == cjk.height,
            "CellSizes ascii and cjk heights must match"
        );
        self.cjk = cjk;
        self
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

    pub const fn ascii_cell(mut self, size: Size) -> Self {
        self.cells = self.cells.with_ascii(size);
        self
    }

    pub const fn cjk_cell(mut self, size: Size) -> Self {
        self.cells = self.cells.with_cjk(size);
        self
    }

    pub const fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}
