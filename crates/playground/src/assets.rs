use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use thiserror::Error;

/// Embedded assets bundled with the Tesserae crate.
#[derive(RustEmbed)]
#[folder = "../../assets/"]
#[include = "fonts/**/*.ttf"]
#[include = "icons/**/*.svg"]
#[exclude = "*.DS_Store"]
pub struct Assets;

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
