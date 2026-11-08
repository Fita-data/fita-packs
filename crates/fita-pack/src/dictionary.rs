//! CSV dictionaries that map variants to a canonical value.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::{Error, Result};

/// Header every dictionary must start with.
pub const HEADER: [&str; 2] = ["variant", "canonical"];

/// A variant to canonical mapping loaded from a dictionary CSV.
///
/// Variants are matched on whole words after [`normalize`]. A variant can span
/// several words (`SOCIEDAD LIMITADA`); the longest match wins. An empty
/// canonical value removes the matched words, which is how stopword lists work.
#[derive(Debug, Clone, Default)]
pub struct Dictionary {
    entries: HashMap<String, String>,
    /// Number of words in the longest variant.
    max_words: usize,
}

impl Dictionary {
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_reader(file, path)
    }

    /// Parses a dictionary. `path` is only used in error messages.
    pub fn from_reader(reader: impl Read, path: &Path) -> Result<Self> {
        let csv_err = |source| Error::Csv {
            path: path.to_path_buf(),
            source,
        };
        let mut csv = csv::ReaderBuilder::new().from_reader(reader);

        let header = csv.headers().map_err(csv_err)?;
        if header.iter().ne(HEADER) {
            return Err(Error::invalid(
                path,
                format!(
                    "header must be `{}`, found `{}`",
                    HEADER.join(","),
                    header.iter().collect::<Vec<_>>().join(",")
                ),
            ));
        }

        let mut dict = Dictionary::default();
        for record in csv.records() {
            let record = record.map_err(csv_err)?;
            let line = record.position().map_or(0, |p| p.line());
            let variant = normalize(&record[0]);
            if variant.is_empty() {
                return Err(Error::invalid(path, format!("line {line}: empty variant")));
            }
            let words = variant.split(' ').count();
            if dict
                .entries
                .insert(variant.clone(), record[1].trim().to_string())
                .is_some()
            {
                return Err(Error::invalid(
                    path,
                    format!("line {line}: duplicate variant `{variant}`"),
                ));
            }
            dict.max_words = dict.max_words.max(words);
        }
        Ok(dict)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Canonical value for a single variant, if there is one.
    pub fn get(&self, variant: &str) -> Option<&str> {
        self.entries.get(&normalize(variant)).map(String::as_str)
    }

    /// Normalizes `value` and replaces every variant found in it with its
    /// canonical value. Words that match nothing are kept, normalized.
    pub fn apply(&self, value: &str) -> String {
        let normalized = normalize(value);
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut out: Vec<&str> = Vec::with_capacity(words.len());

        let mut i = 0;
        'words: while i < words.len() {
            let longest = self.max_words.min(words.len() - i);
            for n in (1..=longest).rev() {
                if let Some(canonical) = self.entries.get(&words[i..i + n].join(" ")) {
                    if !canonical.is_empty() {
                        out.push(canonical);
                    }
                    i += n;
                    continue 'words;
                }
            }
            out.push(words[i]);
            i += 1;
        }
        out.join(" ")
    }
}

/// Uppercases, removes accents from vowels and collapses whitespace.
///
/// `Ñ` and `Ç` are letters of their own in Spanish and Catalan and are kept.
pub fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for word in value.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.extend(word.chars().flat_map(char::to_uppercase).map(fold_accent));
    }
    out
}

fn fold_accent(c: char) -> char {
    match c {
        'Á' | 'À' | 'Â' | 'Ä' => 'A',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'Ó' | 'Ò' | 'Ô' | 'Ö' => 'O',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict(csv: &str) -> Result<Dictionary> {
        Dictionary::from_reader(csv.as_bytes(), Path::new("test.csv"))
    }

    #[test]
    fn normalize_uppercases_and_folds_accents() {
        assert_eq!(normalize("  plaça   de l'Àngel "), "PLAÇA DE L'ANGEL");
        assert_eq!(normalize("Rúa Peña"), "RUA PEÑA");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn apply_prefers_longest_variant() {
        let d = dict("variant,canonical\nS.L.,SL\nSOCIEDAD LIMITADA,SL\nSOCIEDAD,SOC\n").unwrap();
        assert_eq!(
            d.apply("Talleres Inventados Sociedad Limitada"),
            "TALLERES INVENTADOS SL"
        );
        assert_eq!(d.apply("sociedad nueva"), "SOC NUEVA");
    }

    #[test]
    fn apply_drops_words_with_empty_canonical() {
        let d = dict("variant,canonical\nDE,\nLA,\n").unwrap();
        assert_eq!(d.apply("Casa de la Prueba"), "CASA PRUEBA");
    }

    #[test]
    fn rejects_wrong_header() {
        let err = dict("from,to\nA,B\n").unwrap_err();
        assert!(err.to_string().contains("header must be"), "{err}");
    }

    #[test]
    fn rejects_duplicate_variant_after_normalizing() {
        let err = dict("variant,canonical\nRúa,CALLE\nRUA,CALLE\n").unwrap_err();
        assert!(err.to_string().contains("duplicate variant `RUA`"), "{err}");
    }

    #[test]
    fn rejects_rows_with_wrong_column_count() {
        assert!(dict("variant,canonical\nA,B,C\n").is_err());
    }
}
