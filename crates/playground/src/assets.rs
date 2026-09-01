use std::borrow::Cow;

use gpui::{App, AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use thiserror::Error;

/// Embedded assets bundled with the tesserae crate.
#[derive(RustEmbed)]
#[folder = "../../assets/"]
#[include = "fonts/**/*.ttf"]
#[include = "icons/**/*.svg"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl Assets {
    pub fn init(cx: &mut App) -> gpui::Result<()> {
        let font_paths = cx.asset_source().list("fonts")?;
        let mut embedded_fonts = Vec::new();

        for font_path in font_paths {
            if !font_path.ends_with(".ttf") {
                continue;
            }

            let Some(font_bytes) = cx.asset_source().load(&font_path)? else {
                continue;
            };

            embedded_fonts.push(font_bytes);
        }

        cx.text_system().add_fonts(embedded_fonts)
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let asset = <Self as RustEmbed>::get(path).map(|f| f.data);

        if asset.is_some() {
            return Ok(asset);
        }

        Err(AssetLoadError::InvalidPath(path.to_string()).into())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("could not find asset at path \"{0}\"")]
    InvalidPath(String),
}
