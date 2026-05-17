#![no_std]

use embedded_graphics::{
    Drawable, Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
    primitives::Rectangle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitsPerPixel {
    Bpp1,
    Bpp4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphMetrics {
    pub codepoint: char,
    pub bitmap_offset: u32,
    pub width: u16,
    pub height: u16,
    pub x_offset: i16,
    pub y_offset: i16,
    pub x_advance: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CMapEntry {
    pub codepoint: char,
    pub glyph_index: u16,
}

pub struct BitmapFont<'a> {
    pub size: u16,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
    pub bpp: BitsPerPixel,
    pub glyphs: &'a [GlyphMetrics],
    pub bitmap: &'a [u8],
    pub ascii_map: Option<&'a [u16; 128]>,
    pub cmap: &'a [CMapEntry],
}

impl<'a> BitmapFont<'a> {
    pub fn glyph(&self, ch: char) -> Option<&GlyphMetrics> {
        let code = ch as u32;

        if code < 128 {
            if let Some(map) = self.ascii_map {
                let glyph_index = map[code as usize];
                if glyph_index != u16::MAX {
                    return self.glyphs.get(glyph_index as usize);
                }
            }
        }

        self.cmap
            .binary_search_by_key(&ch, |entry| entry.codepoint)
            .ok()
            .and_then(|index| self.glyphs.get(self.cmap[index].glyph_index as usize))
    }

    pub fn glyph_pixel(&self, glyph: &GlyphMetrics, x: u16, y: u16) -> bool {
        if x >= glyph.width || y >= glyph.height {
            return false;
        }

        match self.bpp {
            BitsPerPixel::Bpp1 => {
                let pixel_index = y as usize * glyph.width as usize + x as usize;
                let bit_index = glyph.bitmap_offset as usize * 8 + pixel_index;
                let byte = self.bitmap.get(bit_index / 8).copied().unwrap_or(0);
                let bit = 7 - (bit_index % 8);
                byte & (1 << bit) != 0
            }
            BitsPerPixel::Bpp4 => self.glyph_alpha4(glyph, x, y) != 0,
        }
    }

    pub fn glyph_alpha4(&self, glyph: &GlyphMetrics, x: u16, y: u16) -> u8 {
        if x >= glyph.width || y >= glyph.height {
            return 0;
        }

        let pixel_index = y as usize * glyph.width as usize + x as usize;
        let nibble_index = glyph.bitmap_offset as usize * 2 + pixel_index;
        let byte = self.bitmap.get(nibble_index / 2).copied().unwrap_or(0);
        if nibble_index % 2 == 0 {
            byte >> 4
        } else {
            byte & 0x0f
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellSize {
    GlyphAdvance,
    Fixed { width: u16, height: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPolicy {
    pub ascii: CellSize,
    pub non_ascii: CellSize,
}

impl Default for CellPolicy {
    fn default() -> Self {
        Self {
            ascii: CellSize::GlyphAdvance,
            non_ascii: CellSize::GlyphAdvance,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritingMode {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyle<C: PixelColor> {
    pub text_color: C,
    pub background_color: Option<C>,
    pub cell_policy: CellPolicy,
    pub writing_mode: WritingMode,
    pub direction: TextDirection,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub line_spacing: i16,
    pub char_spacing: i16,
}

impl<C: PixelColor> TextStyle<C> {
    pub const fn new(text_color: C) -> Self {
        Self {
            text_color,
            background_color: None,
            cell_policy: CellPolicy {
                ascii: CellSize::GlyphAdvance,
                non_ascii: CellSize::GlyphAdvance,
            },
            writing_mode: WritingMode::Horizontal,
            direction: TextDirection::LeftToRight,
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Top,
            line_spacing: 0,
            char_spacing: 0,
        }
    }
}

pub struct BitmapText<'a, C: PixelColor> {
    pub text: &'a str,
    pub font: &'a BitmapFont<'a>,
    pub area: Rectangle,
    pub style: TextStyle<C>,
}

impl<'a, C: PixelColor> BitmapText<'a, C> {
    pub const fn new(
        text: &'a str,
        font: &'a BitmapFont<'a>,
        area: Rectangle,
        style: TextStyle<C>,
    ) -> Self {
        Self {
            text,
            font,
            area,
            style,
        }
    }

    pub fn measure(&self) -> Size {
        match self.style.writing_mode {
            WritingMode::Horizontal => {
                let mut width = 0u32;
                let mut height = self.font.size as u32;
                for ch in self.text.chars() {
                    let Some(glyph) = self.font.glyph(ch) else {
                        continue;
                    };
                    let cell = cell_size_for(self.style.cell_policy, ch, glyph, self.font.size);
                    width = width
                        .saturating_add(cell.width as u32 + self.style.char_spacing.max(0) as u32);
                    height = height.max(cell.height as u32);
                }
                Size::new(width, height)
            }
            WritingMode::Vertical => {
                let mut width = self.font.size as u32;
                let mut height = 0u32;
                for ch in self.text.chars() {
                    let Some(glyph) = self.font.glyph(ch) else {
                        continue;
                    };
                    let cell = cell_size_for(self.style.cell_policy, ch, glyph, self.font.size);
                    width = width.max(cell.width as u32);
                    height = height
                        .saturating_add(cell.height as u32 + self.style.char_spacing.max(0) as u32);
                }
                Size::new(width, height)
            }
        }
    }
}

impl<'a, C> Drawable for BitmapText<'a, C>
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let measured = self.measure();
        let start = aligned_start(
            self.area,
            measured,
            self.style.horizontal_align,
            self.style.vertical_align,
        );

        match self.style.writing_mode {
            WritingMode::Horizontal => self.draw_horizontal(target, start),
            WritingMode::Vertical => self.draw_vertical(target, start),
        }
    }
}

impl<'a, C> BitmapText<'a, C>
where
    C: PixelColor,
{
    fn draw_horizontal<D>(&self, target: &mut D, start: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let mut pen_x = start.x;
        let baseline_y = start.y + self.font.ascent as i32;

        let chars = self.text.chars();
        if self.style.direction == TextDirection::RightToLeft {
            let mut buf = [char::REPLACEMENT_CHARACTER; 64];
            let mut len = 0usize;
            for ch in chars.take(buf.len()) {
                buf[len] = ch;
                len += 1;
            }
            for ch in buf[..len].iter().rev().copied() {
                pen_x = self.draw_one_horizontal(target, ch, pen_x, baseline_y)?;
            }
        } else {
            for ch in chars {
                pen_x = self.draw_one_horizontal(target, ch, pen_x, baseline_y)?;
            }
        }

        Ok(())
    }

    fn draw_one_horizontal<D>(
        &self,
        target: &mut D,
        ch: char,
        pen_x: i32,
        baseline_y: i32,
    ) -> Result<i32, D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let Some(glyph) = self.font.glyph(ch) else {
            return Ok(pen_x);
        };
        let draw_pos = Point::new(
            pen_x + glyph.x_offset as i32,
            baseline_y - glyph.y_offset as i32,
        );
        draw_glyph(target, self.font, glyph, draw_pos, self.style.text_color)?;
        let cell = cell_size_for(self.style.cell_policy, ch, glyph, self.font.size);
        Ok(pen_x + cell.width as i32 + self.style.char_spacing as i32)
    }

    fn draw_vertical<D>(&self, target: &mut D, start: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let mut pen_y = start.y;
        let chars = self.text.chars();
        if self.style.direction == TextDirection::BottomToTop {
            let mut buf = [char::REPLACEMENT_CHARACTER; 64];
            let mut len = 0usize;
            for ch in chars.take(buf.len()) {
                buf[len] = ch;
                len += 1;
            }
            for ch in buf[..len].iter().rev().copied() {
                pen_y = self.draw_one_vertical(target, ch, start.x, pen_y)?;
            }
        } else {
            for ch in chars {
                pen_y = self.draw_one_vertical(target, ch, start.x, pen_y)?;
            }
        }
        Ok(())
    }

    fn draw_one_vertical<D>(
        &self,
        target: &mut D,
        ch: char,
        pen_x: i32,
        pen_y: i32,
    ) -> Result<i32, D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let Some(glyph) = self.font.glyph(ch) else {
            return Ok(pen_y);
        };
        let cell = cell_size_for(self.style.cell_policy, ch, glyph, self.font.size);
        let draw_pos = Point::new(
            pen_x + ((cell.width as i32 - glyph.width as i32) / 2) + glyph.x_offset as i32,
            pen_y + ((cell.height as i32 - glyph.height as i32) / 2),
        );
        draw_glyph(target, self.font, glyph, draw_pos, self.style.text_color)?;
        Ok(pen_y + cell.height as i32 + self.style.char_spacing as i32)
    }
}

fn draw_glyph<D, C>(
    target: &mut D,
    font: &BitmapFont<'_>,
    glyph: &GlyphMetrics,
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

fn cell_size_for(policy: CellPolicy, ch: char, glyph: &GlyphMetrics, font_size: u16) -> Size {
    let class_policy = if ch.is_ascii() {
        policy.ascii
    } else {
        policy.non_ascii
    };

    match class_policy {
        CellSize::GlyphAdvance => Size::new(
            glyph.x_advance.max(glyph.width as i16) as u32,
            font_size as u32,
        ),
        CellSize::Fixed { width, height } => Size::new(width as u32, height as u32),
    }
}

fn aligned_start(
    area: Rectangle,
    measured: Size,
    horizontal_align: HorizontalAlign,
    vertical_align: VerticalAlign,
) -> Point {
    let dx = match horizontal_align {
        HorizontalAlign::Left => 0,
        HorizontalAlign::Center => (area.size.width as i32 - measured.width as i32) / 2,
        HorizontalAlign::Right => area.size.width as i32 - measured.width as i32,
    };
    let dy = match vertical_align {
        VerticalAlign::Top => 0,
        VerticalAlign::Middle => (area.size.height as i32 - measured.height as i32) / 2,
        VerticalAlign::Bottom => area.size.height as i32 - measured.height as i32,
    };

    area.top_left + Point::new(dx, dy)
}
