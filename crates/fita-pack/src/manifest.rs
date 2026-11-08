//! Typed view of `pack.yaml`.
//!
//! Unknown keys are rejected so a typo in a manifest fails loudly instead of
//! being ignored.

use std::path::Path;
use std::path::PathBuf;

use semver::{Version, VersionReq};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Pack name. Must match the folder name.
    pub pack: String,
    pub version: Version,
    /// ISO 3166-1 alpha-2 country code, uppercase.
    pub country: String,
    /// Engine versions this pack works with.
    pub fita: VersionReq,
    #[serde(default)]
    pub extends: Extends,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extends {
    pub tax_id: Option<TaxId>,
    pub company_name: Option<CompanyName>,
    pub address: Option<Address>,
    pub phone: Option<Phone>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaxId {
    /// Name of a built in validator or a function in the pack's `src/`.
    pub validator: String,
    #[serde(default)]
    pub strip_prefix: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanyName {
    pub legal_forms: Option<PathBuf>,
    pub stopwords: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    pub street_types: Option<PathBuf>,
    pub postcode: Option<Postcode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Postcode {
    pub pattern: String,
    pub province_from_prefix: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phone {
    pub default_region: String,
}

impl Manifest {
    /// Every dictionary path the manifest references, relative to the pack root.
    pub fn dictionaries(&self) -> Vec<&Path> {
        let ext = &self.extends;
        let company = ext.company_name.as_ref();
        let address = ext.address.as_ref();
        let province = address
            .and_then(|a| a.postcode.as_ref())
            .and_then(|p| p.province_from_prefix.as_deref());

        [
            company.and_then(|c| c.legal_forms.as_deref()),
            company.and_then(|c| c.stopwords.as_deref()),
            address.and_then(|a| a.street_types.as_deref()),
            province,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
