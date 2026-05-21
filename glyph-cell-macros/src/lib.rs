mod emit;
mod input;
mod raster;
mod source;

use std::collections::BTreeMap;

use input::{FontBlock, FontDataInput};
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn font_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FontDataInput);
    expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: FontDataInput) -> syn::Result<proc_macro2::TokenStream> {
    let size = input.size.base10_parse::<u16>()?;
    let blocks = indexed_blocks(input.blocks)?;
    let mut glyphs = Vec::new();

    for (block, chars) in blocks {
        let font = source::load_font(&block.path)?;
        glyphs.extend(raster::rasterize_block(&font, size, chars));
    }

    let generated = emit::font_expression(size, glyphs)?;
    Ok(quote! {{ #generated }})
}

fn indexed_blocks(blocks: Vec<FontBlock>) -> syn::Result<Vec<(FontBlock, Vec<char>)>> {
    let mut seen = BTreeMap::<char, LitStr>::new();
    blocks
        .into_iter()
        .map(|block| {
            let chars = unique_block_chars(&block.index, &mut seen)?;
            Ok((block, chars))
        })
        .collect()
}

fn unique_block_chars(index: &LitStr, seen: &mut BTreeMap<char, LitStr>) -> syn::Result<Vec<char>> {
    let mut chars = Vec::new();

    for ch in index.value().chars() {
        if chars.contains(&ch) {
            continue;
        }

        if let Some(first_index) = seen.get(&ch) {
            return Err(syn::Error::new(
                index.span(),
                format!(
                    "duplicate index character {ch:?}; first seen in index {:?}",
                    first_index.value()
                ),
            ));
        }

        seen.insert(ch, index.clone());
        chars.push(ch);
    }

    Ok(chars)
}
