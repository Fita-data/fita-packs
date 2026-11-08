//! Loader and validator for Fita country packs.
//!
//! A pack is a folder with a `pack.yaml` manifest and a set of CSV
//! dictionaries. See `docs/PACK_FORMAT.md` for the format.

mod dictionary;
mod manifest;
mod pack;

use std::path::PathBuf;

pub use dictionary::{Dictionary, normalize};
pub use manifest::{Address, CompanyName, Extends, Manifest, Phone, Postcode, TaxId};
pub use pack::{MANIFEST_FILE, Pack};

/// Errors raised while loading or validating a pack.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{}: {source}", path.display())]
    Yaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },
    #[error("{}: {source}", path.display())]
    Csv { path: PathBuf, source: csv::Error },
    #[error("{}: {message}", path.display())]
    Invalid { path: PathBuf, message: String },
}

impl Error {
    pub(crate) fn invalid(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Error::Invalid {
            path: path.into(),
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
