# glyph-cell

`glyph-cell` is the public runtime crate. It is `no_std`, targets
`embedded-graphics-core` draw traits, and re-exports `font_data!` from
`glyph-cell-macros`.

## API

- `font_data!`: rasterize one or more font files into `FontData`.
- `FontData`: static glyph index, bitmap bytes, glyph metrics, and design size.
- `Glyph`: one bitmap glyph's metrics and bitmap offset.
- `TextStyle`: color, ASCII/CJK cell sizes, and 3x3 layout alignment.
- `DrawableText`: horizontal or vertical text drawing.

Enable the `debug` feature to draw cell, design, or glyph boxes around rendered
text.

## Example

```rust
use glyph_cell::{font_data, Alignment, DrawableText, FontData, TextStyle};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

const FONT: FontData<'static> = font_data! {
    size: 24,
    path: "assets/ascii.ttf",
    index: "Hello Rust!",
    path: "assets/cjk.otf",
    index: "你好世界",
};

let style = TextStyle::new(Rgb565::WHITE)
    .height(32)
    .ascii_width(16)
    .cjk_width(32)
    .align(Alignment::CENTER);

DrawableText::new(&FONT, "Hello\n你好", style)
    .at(Point::new(8, 8))
    .draw(&mut display)?;

DrawableText::new(&FONT, "Hello\n你好", style)
    .vertical()
    .at(Point::new(72, 8))
    .draw(&mut display)?;
```

With `features = ["debug"]`:

```rust
let text = DrawableText::new(&FONT, "A你", style).at(Point::new(0, 0));

text.draw_cell_boxes(&mut display)?;
text.draw_design_boxes(&mut display)?;
text.draw_glyph_boxes(&mut display)?;
```
