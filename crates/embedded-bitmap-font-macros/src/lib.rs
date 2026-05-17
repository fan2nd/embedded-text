use embedded_bitmap_font_codegen::{BitmapGlyph, CodegenFont, FontWriter, GlyphBitmap};
use fontdue::{Font, FontSettings};
use proc_macro::TokenStream;
use quote::quote;
use std::{fs, path::PathBuf};
use syn::{
    Ident, LitChar, LitInt, LitStr, Result, Token, Type, Visibility, braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct FontMacroInput {
    vis: Visibility,
    ident: Ident,
    ty: Type,
    path: LitStr,
    size: LitInt,
    glyphs: LitStr,
    ranges: Vec<GlyphRange>,
}

struct GlyphRange {
    start: LitChar,
    end: LitChar,
}

impl Parse for GlyphRange {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let start = input.parse()?;
        input.parse::<Token![..=]>()?;
        let end = input.parse()?;
        Ok(Self { start, end })
    }
}

fn parse_ranges(input: ParseStream<'_>) -> Result<Vec<GlyphRange>> {
    let content;
    bracketed!(content in input);
    let mut ranges = Vec::new();
    while !content.is_empty() {
        ranges.push(content.parse()?);
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }
    Ok(ranges)
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
        let mut ranges = Vec::new();

        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "path" => path = Some(content.parse()?),
                "size" => size = Some(content.parse()?),
                "glyphs" => glyphs = Some(content.parse()?),
                "ranges" => ranges.extend(parse_ranges(&content)?),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected path, size, glyphs, or ranges",
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
            ranges,
        })
    }
}

struct MultiFontInput {
    path: LitStr,
    glyphs: LitStr,
    vis: Visibility,
    fonts: Punctuated<MultiFontSpec, Token![,]>,
}

struct MultiFontSpec {
    ident: Ident,
    ty: Type,
    size: LitInt,
}

impl Parse for MultiFontSpec {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;
        let size = input.parse()?;
        Ok(Self { ident, ty, size })
    }
}

impl Parse for MultiFontInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let path = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let glyphs = input.parse()?;
        input.parse::<Token![,]>()?;

        let vis = input.parse()?;
        input.parse::<Token![static]>()?;
        let content;
        braced!(content in input);
        let fonts = content.parse_terminated(MultiFontSpec::parse, Token![,])?;
        Ok(Self {
            path,
            glyphs,
            vis,
            fonts,
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

#[proc_macro]
pub fn bitmap_fonts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MultiFontInput);
    expand_bitmap_fonts(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand_bitmap_font(input: FontMacroInput) -> syn::Result<proc_macro2::TokenStream> {
    let bytes = read_font_bytes(&input.path)?;
    let font = parse_font(&input.path, bytes)?;
    let size = input.size.base10_parse::<u16>()?;
    let glyphs = rasterize_codepoints(&font, size, input.glyphs.value().chars(), &input.ranges)?;
    emit_font(
        input.vis,
        input.ident,
        input.ty,
        size,
        input.glyphs.value(),
        glyphs,
    )
}

fn expand_bitmap_fonts(input: MultiFontInput) -> syn::Result<proc_macro2::TokenStream> {
    let bytes = read_font_bytes(&input.path)?;
    let font = parse_font(&input.path, bytes)?;
    let mut output = proc_macro2::TokenStream::new();

    for spec in input.fonts {
        let size = spec.size.base10_parse::<u16>()?;
        let glyphs = rasterize_codepoints(&font, size, input.glyphs.value().chars(), &[])?;
        let ident = spec.ident;
        let module_ident = Ident::new(
            &format!(
                "__embedded_bitmap_font_{}",
                ident.to_string().to_lowercase()
            ),
            ident.span(),
        );
        let font_tokens = emit_font(
            input.vis.clone(),
            ident.clone(),
            spec.ty,
            size,
            input.glyphs.value(),
            glyphs,
        )?;
        output.extend(quote! {
            mod #module_ident {
                #font_tokens
            }
            pub use #module_ident::#ident;
        });
    }

    Ok(output)
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

fn rasterize_codepoints(
    font: &Font,
    size: u16,
    chars: impl Iterator<Item = char>,
    ranges: &[GlyphRange],
) -> syn::Result<Vec<BitmapGlyph>> {
    let mut codepoints: Vec<char> = chars.collect();
    for range in ranges {
        let start = range.start.value() as u32;
        let end = range.end.value() as u32;
        if start > end {
            return Err(syn::Error::new(
                range.start.span(),
                "range start must be <= range end",
            ));
        }
        for codepoint in start..=end {
            let Some(ch) = char::from_u32(codepoint) else {
                continue;
            };
            codepoints.push(ch);
        }
    }
    codepoints.sort_unstable();
    codepoints.dedup();
    Ok(codepoints
        .into_iter()
        .map(|codepoint| rasterize_glyph(font, codepoint, size as f32))
        .collect())
}

fn emit_font(
    vis: Visibility,
    ident: Ident,
    ty: Type,
    size: u16,
    index: String,
    glyphs: Vec<BitmapGlyph>,
) -> syn::Result<proc_macro2::TokenStream> {
    let source = FontWriter::new(CodegenFont {
        ident: ident.to_string(),
        index,
        size,
        ascent: (size as i16) - 3,
        descent: -3,
        line_gap: 2,
        glyphs,
    })
    .write_rust_source()
    .map_err(|_| syn::Error::new(ident.span(), "failed to format generated font"))?;
    let generated: proc_macro2::TokenStream = source.parse().map_err(|err| {
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
