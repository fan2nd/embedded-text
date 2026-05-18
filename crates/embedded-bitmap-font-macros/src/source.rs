use std::{fs, path::PathBuf};

use fontdue::{Font, FontSettings};
use syn::LitStr;

pub(crate) fn load_font(path: &LitStr) -> syn::Result<Font> {
    let font_path = resolve_font_path(&path.value()).map_err(|message| {
        syn::Error::new(
            path.span(),
            format!("failed to resolve font path {:?}: {message}", path.value()),
        )
    })?;

    let bytes = fs::read(&font_path).map_err(|err| {
        syn::Error::new(
            path.span(),
            format!("failed to read font path {}: {err}", font_path.display()),
        )
    })?;

    Font::from_bytes(bytes, FontSettings::default())
        .map_err(|err| syn::Error::new(path.span(), format!("failed to parse font: {err}")))
}

fn resolve_font_path(path: &str) -> Result<PathBuf, String> {
    let literal = PathBuf::from(path);
    if literal.is_absolute() && literal.exists() {
        return Ok(literal);
    }

    font_path_candidates(&literal)
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            "tried current directory, caller CARGO_MANIFEST_DIR, macro crate, and workspace root"
                .into()
        })
}

fn font_path_candidates(path: &PathBuf) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(path));
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest_dir).join(path));
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path));
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    );
    candidates
}
