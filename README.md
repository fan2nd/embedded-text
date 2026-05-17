# font-conv

Embedded Rust bitmap font experiment.

## Current MVP

This repository contains a small Rust workspace for a no-std bitmap font runtime
plus a build-time code generator:

```text
crates/embedded-bitmap-font/         no_std runtime + embedded-graphics Drawable
crates/embedded-bitmap-font-codegen/ host-side Rust source generator
crates/embedded-bitmap-font-macros/  proc-macro front-end for compile-time font embedding
.ref/lv_font_conv/                   reference implementation notes/code
```

The first version intentionally starts with hand-supplied 1bpp glyph bitmaps and
Rust source generation. FreeType extraction will be wired into the codegen crate
next, reusing the same generated data model.

## Runtime data model

`embedded-bitmap-font` defines:

- `BitmapFont`: static font metadata, glyph table, bitmap blob, ASCII map, sparse
  non-ASCII cmap.
- `GlyphMetrics`: LVGL-style bitmap metrics: width/height, bitmap offset,
  x/y offset, and x advance.
- `BitmapText`: text layout object that implements `embedded_graphics::Drawable`.
- `TextStyle`: cell policy, writing mode, direction, and alignment options.

The current drawing path supports:

- `#![no_std]` runtime crate.
- 1bpp bitmap decoding.
- ASCII fast lookup and sparse non-ASCII lookup.
- Horizontal left-to-right drawing.
- Basic vertical / reverse-direction layout hooks.
- `embedded-graphics` `DrawTarget` integration.

## Example runtime use

```rust
use embedded_bitmap_font::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};

let text = BitmapText::new(
    "Hello",
    &FONT_16,
    Rectangle::new(Point::new(0, 0), Size::new(128, 32)),
    TextStyle::new(BinaryColor::On),
);
text.draw(&mut display)?;
```

## Example codegen use

`embedded-bitmap-font-codegen` can already generate Rust const data from an
in-memory glyph list:

```rust
use embedded_bitmap_font_codegen::{BitmapGlyph, CodegenFont, FontWriter, GlyphBitmap};

let font = CodegenFont {
    ident: "FONT_5".into(),
    size: 5,
    ascent: 5,
    descent: 0,
    line_gap: 0,
    glyphs: vec![BitmapGlyph {
        codepoint: 'A',
        width: 3,
        height: 5,
        x_offset: 0,
        y_offset: 5,
        x_advance: 4,
        bitmap: GlyphBitmap::Bpp1(vec![
            false, true, false,
            true, false, true,
            true, true, true,
            true, false, true,
            true, false, true,
        ]),
    }],
};

FontWriter::new(font).write_rust_file("$OUT_DIR/font.rs")?;
```

Generated source can be included from a target crate with:

```rust
include!(concat!(env!("OUT_DIR"), "/font.rs"));
```

## Example procedural macro use

`embedded-bitmap-font-macros` starts the final API direction: font assets can be
rasterized at compile time directly from a proc macro. The current first slice supports `path`, pixel `size`, and an explicit glyph string.
It also supports ASCII-style inclusive ranges and multi-size generation:

```rust
use embedded_bitmap_font::BitmapFont;
use embedded_bitmap_font_macros::{bitmap_font, bitmap_fonts};

bitmap_font! {
    pub static FONT_12: BitmapFont<'static> = {
        path: "assets/Cubic_11.ttf",
        size: 12,
        glyphs: "Hello Rust 你好",
        ranges: ['0'..='9']
    };
}

bitmap_fonts! {
    path: "assets/Cubic_11.ttf",
    glyphs: "Hello Rust 你好",
    pub static {
        FONT_12: BitmapFont<'static> = 12,
        FONT_18: BitmapFont<'static> = 18,
    }
}
```

There is also a small runtime demo for the new `FontData` / `DrawableText` API:

```bash
cargo run -p embedded-bitmap-font --example drawable_text_demo
```

## Planned next steps

1. Add better macro diagnostics and compile-fail coverage for missing fields / invalid ranges.
2. Add FreeType-backed extraction in `embedded-bitmap-font-codegen`.
3. Add fixed ASCII/non-ASCII cell sizing tests.
4. Add robust vertical layout tests.
5. Add 4bpp drawing semantics and optional blending strategy.

## Checks

```bash
cargo fmt --all --check
cargo test --workspace
cargo check --workspace
```
