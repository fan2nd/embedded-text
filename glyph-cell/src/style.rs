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
    pub height: u32,
    pub ascii_width: u32,
    pub cjk_width: u32,
}

impl CellSizes {
    pub const fn new(height: u32, ascii_width: u32, cjk_width: u32) -> Self {
        Self {
            height,
            ascii_width,
            cjk_width,
        }
    }

    pub const fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub const fn with_ascii_width(mut self, width: u32) -> Self {
        self.ascii_width = width;
        self
    }

    pub const fn with_cjk_width(mut self, width: u32) -> Self {
        self.cjk_width = width;
        self
    }

    pub const fn for_char(self, ch: char) -> Size {
        let width = if ch.is_ascii() {
            self.ascii_width
        } else {
            self.cjk_width
        };
        Size::new(width, self.height)
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
            cells: CellSizes::new(0, 0, 0),
            alignment: Alignment::CENTER,
        }
    }

    pub const fn height(mut self, height: u32) -> Self {
        self.cells = self.cells.with_height(height);
        self
    }

    pub const fn ascii_width(mut self, width: u32) -> Self {
        self.cells = self.cells.with_ascii_width(width);
        self
    }

    pub const fn cjk_width(mut self, width: u32) -> Self {
        self.cells = self.cells.with_cjk_width(width);
        self
    }

    pub const fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}
