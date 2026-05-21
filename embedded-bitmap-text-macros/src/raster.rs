use fontdue::Font;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BitmapGlyph {
    pub codepoint: char,
    pub width: u16,
    pub height: u16,
    pub x_offset: i16,
    pub y_offset: i16,
    pub bitmap: Vec<bool>,
}

pub(crate) fn rasterize_block(
    font: &Font,
    size: u16,
    chars: impl IntoIterator<Item = char>,
) -> Vec<BitmapGlyph> {
    let mut glyphs: Vec<_> = chars
        .into_iter()
        .map(|codepoint| rasterize_glyph(font, codepoint, size))
        .collect();
    apply_auto_y_offset(size, &mut glyphs);
    glyphs
}

fn rasterize_glyph(font: &Font, codepoint: char, size: u16) -> BitmapGlyph {
    let (metrics, bitmap) = font.rasterize(codepoint, size as f32);
    let width = metrics.width.max(1) as u16;
    let height = metrics.height.max(1) as u16;
    let pixels = if metrics.width == 0 || metrics.height == 0 {
        vec![false; width as usize * height as usize]
    } else {
        bitmap.into_iter().map(|alpha| alpha >= 96).collect()
    };

    BitmapGlyph {
        codepoint,
        width,
        height,
        x_offset: centered_offset(size, width),
        y_offset: (metrics.height as i32 + metrics.ymin) as i16,
        bitmap: pixels,
    }
}

fn centered_offset(design: u16, glyph: u16) -> i16 {
    ((design as i32 - glyph as i32) / 2) as i16
}

fn apply_auto_y_offset(design_size: u16, glyphs: &mut [BitmapGlyph]) {
    let delta = y_offset_delta(design_size, glyphs);
    for glyph in glyphs {
        glyph.y_offset = (glyph.y_offset as i32 + delta) as i16;
    }
}

fn y_offset_delta(design_size: u16, glyphs: &[BitmapGlyph]) -> i32 {
    let Some(first) = glyphs.first() else {
        return 0;
    };

    let design_size = design_size as i32;
    let mut min_top = glyph_top(design_size, first);
    let mut max_bottom = glyph_bottom(design_size, first);

    for glyph in &glyphs[1..] {
        min_top = min_top.min(glyph_top(design_size, glyph));
        max_bottom = max_bottom.max(glyph_bottom(design_size, glyph));
    }

    let min_delta = max_bottom - design_size;
    let max_delta = min_top;

    if min_delta <= max_delta {
        0.clamp(min_delta, max_delta)
    } else {
        (min_top + max_bottom - design_size) / 2
    }
}

fn glyph_top(design_size: i32, glyph: &BitmapGlyph) -> i32 {
    design_size - glyph.y_offset as i32
}

fn glyph_bottom(design_size: i32, glyph: &BitmapGlyph) -> i32 {
    glyph_top(design_size, glyph) + glyph.height as i32
}

#[cfg(test)]
mod tests {
    use super::{BitmapGlyph, centered_offset, y_offset_delta};

    #[test]
    fn centers_glyph_bitmap_in_design_box() {
        assert_eq!(centered_offset(24, 10), 7);
        assert_eq!(centered_offset(24, 24), 0);
        assert_eq!(centered_offset(24, 28), -2);
    }

    #[test]
    fn keeps_glyphs_that_already_fit_in_place() {
        assert_eq!(y_offset_delta(12, &[glyph(5, 10), glyph(4, 8)]), 0);
    }

    #[test]
    fn moves_font_block_down_until_top_fits() {
        assert_eq!(y_offset_delta(12, &[glyph(5, 14), glyph(4, 12)]), -2);
    }

    #[test]
    fn moves_font_block_up_until_bottom_fits() {
        assert_eq!(y_offset_delta(12, &[glyph(5, 3), glyph(4, 4)]), 2);
    }

    fn glyph(height: u16, y_offset: i16) -> BitmapGlyph {
        BitmapGlyph {
            codepoint: 'A',
            width: 1,
            height,
            x_offset: 0,
            y_offset,
            bitmap: vec![true; height as usize],
        }
    }
}
