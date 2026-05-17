//! Procedural macros for embedding bitmap fonts.

use embedded_bitmap_font_codegen::{BitmapGlyph, CodegenFont, FontWriter, GlyphBitmap};
use fontdue::{Font, FontSettings};
use proc_macro::TokenStream;
use quote::quote;
use std::{fs, path::PathBuf};
use syn::{
    Ident, LitInt, LitStr, Result, Token, Type, Visibility, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct FontMacroInput {
    vis: Visibility,
    ident: Ident,
    ty: Type,
    path: LitStr,
    size: LitInt,
    glyphs: LitStr,
}

impl Parse for FontMacroInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let vis = input.parse()?;
        input.parse::<Token![static]>()?;
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;

        let content;
        braced!(content in input);

        let mut path = None;
        let mut size = None;
        let mut glyphs = None;

        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "path" => path = Some(content.parse()?),
                "size" => size = Some(content.parse()?),
                "glyphs" => glyphs = Some(content.parse()?),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected path, size, or glyphs",
                    ));
                }
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        input.parse::<Token![;]>()?;

        Ok(Self {
            vis,
            ident,
            ty,
            path: path.ok_or_else(|| input.error("missing path"))?,
            size: size.ok_or_else(|| input.error("missing size"))?,
            glyphs: glyphs.ok_or_else(|| input.error("missing glyphs"))?,
        })
    }
}

#[proc_macro]
pub fn bitmap_font(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FontMacroInput);
    expand_bitmap_font(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand_bitmap_font(input: FontMacroInput) -> syn::Result<proc_macro2::TokenStream> {
    let font_path = resolve_font_path(&input.path.value()).map_err(|message| {
        syn::Error::new(
            input.path.span(),
            format!(
                "failed to resolve font path {:?}: {message}",
                input.path.value()
            ),
        )
    })?;
    let bytes = fs::read(&font_path).map_err(|err| {
        syn::Error::new(
            input.path.span(),
            format!("failed to read font path {}: {err}", font_path.display()),
        )
    })?;
    let font = Font::from_bytes(bytes, FontSettings::default()).map_err(|err| {
        syn::Error::new(input.path.span(), format!("failed to parse font: {err}"))
    })?;
    let size = input.size.base10_parse::<u16>()?;
    let px = size as f32;
    let glyphs = input
        .glyphs
        .value()
        .chars()
        .map(|codepoint| rasterize_glyph(&font, codepoint, px))
        .collect();

    let ident = input.ident;
    let vis = input.vis;
    let ty = input.ty;
    let source = FontWriter::new(CodegenFont {
        ident: ident.to_string(),
        size,
        ascent: (size as i16) - 3,
        descent: -3,
        line_gap: 2,
        glyphs,
    })
    .write_rust_source()
    .map_err(|_| syn::Error::new(ident.span(), "failed to format generated font"))?;
    let generated: proc_macro2::TokenStream = source
        .replace("use embedded_bitmap_font::{\n    BitmapFont, BitsPerPixel, CMapEntry, GlyphMetrics,\n};\n\n", "")
        .parse()
        .map_err(|err| {
            syn::Error::new(
                ident.span(),
                format!("generated invalid Rust source: {err}"),
            )
        })?;

    Ok(quote! {
        const _: fn() = || {
            fn _assert_type(_: &#ty) {}
            let _ = _assert_type;
        };
        use embedded_bitmap_font::{BitsPerPixel, CMapEntry, GlyphMetrics};
        #vis #generated
    })
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

fn rasterize_glyph(font: &Font, codepoint: char, px: f32) -> BitmapGlyph {
    let (metrics, bitmap) = font.rasterize(codepoint, px);
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
        x_offset: metrics.xmin as i16,
        y_offset: (metrics.height as i32 + metrics.ymin) as i16,
        x_advance: metrics.advance_width.ceil().max(width as f32) as i16,
        bitmap: GlyphBitmap::Bpp1(pixels),
    }
}
