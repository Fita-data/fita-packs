//! Loading a pack folder and checking it is consistent.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::{Dictionary, Error, Manifest, Result};

pub const MANIFEST_FILE: &str = "pack.yaml";

/// A loaded and validated pack.
#[derive(Debug, Clone)]
pub struct Pack {
    pub root: PathBuf,
    pub manifest: Manifest,
    /// Dictionaries keyed by their path relative to the pack root, as written
    /// in the manifest.
    pub dictionaries: BTreeMap<PathBuf, Dictionary>,
}

impl Pack {
    /// Loads `<root>/pack.yaml` and every dictionary it references.
    ///
    /// Fails if the manifest does not parse, its name does not match the
    /// folder, or a dictionary is missing, outside the pack or malformed.
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let manifest_path = root.join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&manifest_path).map_err(|source| Error::Io {
            path: manifest_path.clone(),
            source,
        })?;
        let manifest: Manifest = serde_yaml::from_str(&text).map_err(|source| Error::Yaml {
            path: manifest_path.clone(),
            source,
        })?;

        validate_manifest(&manifest, &root, &manifest_path)?;

        let mut dictionaries = BTreeMap::new();
        for rel in manifest.dictionaries() {
            if !is_inside_pack(rel) {
                return Err(Error::invalid(
                    &manifest_path,
                    format!(
                        "dictionary `{}` must be a relative path inside the pack",
                        rel.display()
                    ),
                ));
            }
            let dict = Dictionary::from_path(&root.join(rel))?;
            dictionaries.insert(rel.to_path_buf(), dict);
        }

        Ok(Pack {
            root,
            manifest,
            dictionaries,
        })
    }

    /// Finds a dictionary by file stem, for example `street_types`.
    pub fn dictionary(&self, name: &str) -> Option<&Dictionary> {
        self.dictionaries
            .iter()
            .find(|(path, _)| path.file_stem().is_some_and(|s| s == name))
            .map(|(_, dict)| dict)
    }
}

fn validate_manifest(manifest: &Manifest, root: &Path, manifest_path: &Path) -> Result<()> {
    let folder = root.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if manifest.pack != folder {
        return Err(Error::invalid(
            manifest_path,
            format!(
                "pack name `{}` does not match folder `{folder}`",
                manifest.pack
            ),
        ));
    }

    let country = &manifest.country;
    if country.len() != 2 || !country.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(Error::invalid(
            manifest_path,
            format!("country `{country}` must be two uppercase letters"),
        ));
    }
    Ok(())
}

/// Packs are self contained, so dictionary paths may not escape the pack.
fn is_inside_pack(path: &Path) -> bool {
    path.components().all(|c| matches!(c, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const MANIFEST: &str = r#"
pack: xx
version: 0.1.0
country: XX
fita: ">=0.1, <0.3"
extends:
  address:
    street_types: dictionaries/street_types.csv
"#;

    fn write_pack(manifest: &str, dict: Option<&str>) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("xx");
        fs::create_dir_all(root.join("dictionaries")).unwrap();
        fs::write(root.join(MANIFEST_FILE), manifest).unwrap();
        if let Some(dict) = dict {
            fs::write(root.join("dictionaries/street_types.csv"), dict).unwrap();
        }
        tmp
    }

    #[test]
    fn loads_valid_pack() {
        let tmp = write_pack(MANIFEST, Some("variant,canonical\nC/,CALLE\n"));
        let pack = Pack::load(tmp.path().join("xx")).unwrap();
        assert_eq!(pack.manifest.pack, "xx");
        assert_eq!(pack.dictionary("street_types").unwrap().len(), 1);
    }

    #[test]
    fn fails_on_missing_dictionary() {
        let tmp = write_pack(MANIFEST, None);
        let err = Pack::load(tmp.path().join("xx")).unwrap_err();
        assert!(matches!(err, Error::Io { .. }), "{err}");
    }

    #[test]
    fn fails_on_bad_dictionary_header() {
        let tmp = write_pack(MANIFEST, Some("from,to\nC/,CALLE\n"));
        let err = Pack::load(tmp.path().join("xx")).unwrap_err();
        assert!(err.to_string().contains("header must be"), "{err}");
    }

    #[test]
    fn fails_on_unknown_key() {
        let manifest = MANIFEST.replace("country: XX", "country: XX\ncolour: blue");
        let tmp = write_pack(&manifest, Some("variant,canonical\n"));
        let err = Pack::load(tmp.path().join("xx")).unwrap_err();
        assert!(matches!(err, Error::Yaml { .. }), "{err}");
    }

    #[test]
    fn fails_when_name_does_not_match_folder() {
        let manifest = MANIFEST.replace("pack: xx", "pack: yy");
        let tmp = write_pack(&manifest, Some("variant,canonical\n"));
        let err = Pack::load(tmp.path().join("xx")).unwrap_err();
        assert!(err.to_string().contains("does not match folder"), "{err}");
    }

    #[test]
    fn fails_on_dictionary_outside_pack() {
        let manifest = MANIFEST.replace("dictionaries/street_types.csv", "../street_types.csv");
        let tmp = write_pack(&manifest, None);
        let err = Pack::load(tmp.path().join("xx")).unwrap_err();
        assert!(err.to_string().contains("inside the pack"), "{err}");
    }
}
