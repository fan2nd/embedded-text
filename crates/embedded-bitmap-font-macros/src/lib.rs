use std::fmt::Write;

use fontdue::{Font, FontSettings};
use proc_macro::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, fs, path::PathBuf};
use syn::{
    Expr, Ident, Lit, LitInt, LitStr, Result, Token, UnOp,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum GlyphBitmap {
    Bpp1(Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BitmapGlyph {
    codepoint: char,
    width: u16,
    height: u16,
    x_offset: i16,
    y_offset: i16,
    x_advance: i16,
    bitmap: GlyphBitmap,
}

struct FontDataInput {
    size: LitInt,
    blocks: Vec<FontDataBlock>,
}

struct FontDataBlock {
    path: LitStr,
    index: LitStr,
    y_offset: Option<Expr>,
}

impl Parse for FontDataInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut size = None;
        let mut blocks = Vec::new();
        let mut current: Option<FontDataBlock> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "size" => size = Some(input.parse()?),
                "path" => {
                    if let Some(block) = current.take() {
                        blocks.push(block);
                    }
                    current = Some(FontDataBlock {
                        path: input.parse()?,
                        index: LitStr::new("", key.span()),
                        y_offset: None,
                    });
                }
                "index" => {
                    let Some(block) = current.as_mut() else {
                        return Err(syn::Error::new(key.span(), "index must follow path"));
                    };
                    if !block.index.value().is_empty() {
                        return Err(syn::Error::new(
                            key.span(),
                            "duplicate index for font block",
                        ));
                    }
                    block.index = input.parse()?;
                }
                "y_offset" => {
                    let Some(block) = current.as_mut() else {
                        return Err(syn::Error::new(key.span(), "y_offset must follow path"));
                    };
                    if block.y_offset.is_some() {
                        return Err(syn::Error::new(
                            key.span(),
                            "duplicate y_offset for font block",
                        ));
                    }
                    block.y_offset = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected size, path, index, or y_offset",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if let Some(block) = current.take() {
            blocks.push(block);
        }
        if blocks.is_empty() {
            return Err(input.error("missing font block"));
        }
        for block in &blocks {
            if block.index.value().is_empty() {
                return Err(syn::Error::new(
                    block.path.span(),
                    "missing index for font block",
                ));
            }
        }

        Ok(Self {
            size: size.ok_or_else(|| input.error("missing size"))?,
            blocks,
        })
    }
}

#[proc_macro]
pub fn font_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FontDataInput);
    expand_font_data(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand_font_data(input: FontDataInput) -> syn::Result<proc_macro2::TokenStream> {
    let size = input.size.base10_parse::<u16>()?;
    let mut seen = BTreeMap::<char, LitStr>::new();
    let mut glyphs = Vec::new();
    let mut index = String::new();

    for block in input.blocks {
        let y_offset = match block.y_offset {
            Some(value) => parse_y_offset(&value)?,
            None => 0,
        };
        let bytes = read_font_bytes(&block.path)?;
        let font = parse_font(&block.path, bytes)?;
        let mut block_chars = Vec::new();

        for ch in block.index.value().chars() {
            if block_chars.contains(&ch) {
                continue;
            }
            if let Some(first_index) = seen.get(&ch) {
                return Err(syn::Error::new(
                    block.index.span(),
                    format!(
                        "duplicate index character {ch:?}; first seen in index {:?}",
                        first_index.value()
                    ),
                ));
            }
            seen.insert(ch, block.index.clone());
            index.push(ch);
            block_chars.push(ch);
        }

        glyphs.extend(rasterize_codepoints_with_y_offset(
            &font,
            size,
            block_chars.into_iter(),
            y_offset,
        ));
    }

    emit_font_expression(size, glyphs)
}

fn parse_y_offset(expr: &Expr) -> syn::Result<i16> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(value) => value.base10_parse::<i16>(),
            _ => Err(syn::Error::new(
                expr_lit.lit.span(),
                "y_offset must be an integer literal",
            )),
        },
        Expr::Unary(expr_unary) if matches!(expr_unary.op, UnOp::Neg(_)) => {
            match &*expr_unary.expr {
                Expr::Lit(expr_lit) => match &expr_lit.lit {
                    Lit::Int(value) => {
                        let magnitude = value.base10_parse::<i16>()?;
                        magnitude.checked_neg().ok_or_else(|| {
                            syn::Error::new(
                                value.span(),
                                "y_offset is outside the supported i16 range",
                            )
                        })
                    }
                    _ => Err(syn::Error::new(
                        expr_lit.lit.span(),
                        "y_offset must be an integer literal",
                    )),
                },
                _ => Err(syn::Error::new(
                    expr_unary.expr.span(),
                    "y_offset must be an integer literal",
                )),
            }
        }
        _ => Err(syn::Error::new(
            expr.span(),
            "y_offset must be an integer literal",
        )),
    }
}

fn read_font_bytes(path: &LitStr) -> syn::Result<Vec<u8>> {
    let font_path = resolve_font_path(&path.value()).map_err(|message| {
        syn::Error::new(
            path.span(),
            format!("failed to resolve font path {:?}: {message}", path.value()),
        )
    })?;
    fs::read(&font_path).map_err(|err| {
        syn::Error::new(
            path.span(),
            format!("failed to read font path {}: {err}", font_path.display()),
        )
    })
}

fn parse_font(path: &LitStr, bytes: Vec<u8>) -> syn::Result<Font> {
    Font::from_bytes(bytes, FontSettings::default())
        .map_err(|err| syn::Error::new(path.span(), format!("failed to parse font: {err}")))
}

fn rasterize_codepoints_with_y_offset(
    font: &Font,
    size: u16,
    chars: impl Iterator<Item = char>,
    y_offset: i16,
) -> Vec<BitmapGlyph> {
    let mut codepoints: Vec<char> = chars.collect();
    codepoints.sort_unstable();
    codepoints.dedup();
    codepoints
        .into_iter()
        .map(|codepoint| {
            let mut glyph = rasterize_glyph(font, codepoint, size);
            glyph.y_offset += y_offset;
            glyph
        })
        .collect()
}

fn emit_font_expression(
    size: u16,
    glyphs: Vec<BitmapGlyph>,
) -> syn::Result<proc_macro2::TokenStream> {
    let source = write_font_expression_source(size, glyphs)
        .map_err(|_| syn::Error::new(input_span(), "failed to format generated font"))?;
    let generated: proc_macro2::TokenStream = source.parse().map_err(|err| {
        syn::Error::new(
            input_span(),
            format!("generated invalid Rust source: {err}"),
        )
    })?;

    Ok(quote! {{
        #generated
    }})
}

fn write_font_expression_source(
    size: u16,
    mut glyphs: Vec<BitmapGlyph>,
) -> std::result::Result<String, std::fmt::Error> {
    glyphs.sort_by_key(|glyph| glyph.codepoint);

    let mut bitmap = Vec::new();
    let mut metrics = Vec::new();
    for glyph in &glyphs {
        let bitmap_offset = bitmap.len() as u32;
        match &glyph.bitmap {
            GlyphBitmap::Bpp1(pixels) => pack_bpp1(pixels, &mut bitmap),
        }
        metrics.push((glyph, bitmap_offset));
    }

    let mut source = String::new();
    writeln!(
        source,
        "const GLYPHS: [embedded_bitmap_font::Glyph; {}] = [",
        glyphs.len()
    )?;
    for (glyph, bitmap_offset) in metrics {
        writeln!(source, "    embedded_bitmap_font::Glyph {{")?;
        writeln!(source, "        bitmap_offset: {bitmap_offset},")?;
        writeln!(source, "        width: {},", glyph.width)?;
        writeln!(source, "        height: {},", glyph.height)?;
        writeln!(source, "        x_offset: {},", glyph.x_offset)?;
        writeln!(source, "        y_offset: {},", glyph.y_offset)?;
        writeln!(source, "        x_advance: {},", glyph.x_advance)?;
        writeln!(source, "    }},")?;
    }
    writeln!(source, "];\n")?;

    writeln!(source, "const BITMAP: [u8; {}] = [", bitmap.len())?;
    for byte in &bitmap {
        writeln!(source, "    0b{byte:08b},")?;
    }
    writeln!(source, "];\n")?;

    let index: String = glyphs.iter().map(|glyph| glyph.codepoint).collect();
    writeln!(source, "embedded_bitmap_font::FontData {{")?;
    writeln!(source, "    index: {:?},", index)?;
    writeln!(source, "    char_size: {},", size as usize)?;
    writeln!(source, "    bitmap: &BITMAP,")?;
    writeln!(source, "    glyphs: &GLYPHS,")?;
    writeln!(source, "}}")?;

    Ok(source)
}

fn pack_bpp1(pixels: &[bool], output: &mut Vec<u8>) {
    let mut byte = 0u8;
    for (index, pixel) in pixels.iter().enumerate() {
        if *pixel {
            byte |= 1 << (7 - index % 8);
        }
        if index % 8 == 7 {
            output.push(byte);
            byte = 0;
        }
    }

    if !pixels.len().is_multiple_of(8) {
        output.push(byte);
    }
}

fn input_span() -> proc_macro2::Span {
    proc_macro2::Span::call_site()
}

fn resolve_font_path(path: &str) -> std::result::Result<PathBuf, String> {
    let literal = PathBuf::from(path);
    if literal.is_absolute() && literal.exists() {
        return Ok(literal);
    }

    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(&literal));
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest_dir).join(&literal));
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&literal));
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&literal),
    );

    candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            "tried current directory, caller CARGO_MANIFEST_DIR, macro crate, and workspace root"
                .into()
        })
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
        x_offset: centered_x_offset(size, width),
        y_offset: (metrics.height as i32 + metrics.ymin) as i16,
        x_advance: metrics.advance_width.ceil().max(width as f32) as i16,
        bitmap: GlyphBitmap::Bpp1(pixels),
    }
}

fn centered_x_offset(design_size: u16, glyph_width: u16) -> i16 {
    ((design_size as i32 - glyph_width as i32) / 2) as i16
}

#[cfg(test)]
mod tests {
    use super::centered_x_offset;

    #[test]
    fn centers_glyph_bitmap_in_design_box() {
        assert_eq!(centered_x_offset(24, 10), 7);
        assert_eq!(centered_x_offset(24, 24), 0);
        assert_eq!(centered_x_offset(24, 28), -2);
    }
}
