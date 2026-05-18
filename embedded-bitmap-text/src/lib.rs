#![no_std]

mod font;
mod layout;
mod style;
mod text;

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
pub use debug::DebugBoxKind;
pub use embedded_bitmap_text_macros::font_data;
pub use font::{BitsPerPixel, FontData, Glyph};
pub use style::{Alignment, CellSizes, HorizontalAlignment, TextStyle, VerticalAlignment};
pub use text::{DrawableText, VerticalDrawableText};
