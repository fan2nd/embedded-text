use syn::{
    Ident, LitInt, LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct FontDataInput {
    pub size: LitInt,
    pub blocks: Vec<FontBlock>,
}

pub(crate) struct FontBlock {
    pub path: LitStr,
    pub index: LitStr,
}

impl Parse for FontDataInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut size = None;
        let mut blocks = Vec::new();
        let mut current = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "size" => size = Some(input.parse()?),
                "path" => {
                    finish_block(&mut blocks, current.take());
                    current = Some(FontBlock {
                        path: input.parse()?,
                        index: LitStr::new("", key.span()),
                    });
                }
                "index" => parse_index(&mut current, key, input)?,
                _ => return Err(syn::Error::new(key.span(), "expected size, path, or index")),
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        finish_block(&mut blocks, current.take());
        validate(size, blocks, input)
    }
}

fn parse_index(current: &mut Option<FontBlock>, key: Ident, input: ParseStream<'_>) -> Result<()> {
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
    Ok(())
}

fn finish_block(blocks: &mut Vec<FontBlock>, block: Option<FontBlock>) {
    if let Some(block) = block {
        blocks.push(block);
    }
}

fn validate(
    size: Option<LitInt>,
    blocks: Vec<FontBlock>,
    input: ParseStream<'_>,
) -> Result<FontDataInput> {
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

    Ok(FontDataInput {
        size: size.ok_or_else(|| input.error("missing size"))?,
        blocks,
    })
}
