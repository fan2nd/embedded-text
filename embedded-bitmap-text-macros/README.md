# embedded-bitmap-text-macros

`embedded-bitmap-text-macros` implements the `font_data!` procedural macro used
by `embedded-bitmap-text`.

Most users should import the macro from the public runtime crate instead:

```rust
use embedded_bitmap_text::{font_data, FontData};
```

## Macro Shape

```rust
const FONT: FontData<'static> = font_data! {
    size: 24,
    path: "assets/ascii.ttf",
    index: "Hello Rust!",
    path: "assets/cjk.otf",
    index: "你好世界",
};
```

Each invocation creates one pixel size. Multiple `path` + `index` blocks may be
mixed in the same invocation, for example one ASCII block and one CJK block. The
macro deduplicates repeated characters inside one block, rejects duplicate
characters across blocks, and derives a shared vertical offset per font block.
