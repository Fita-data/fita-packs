//! Loads every pack under `packs/` and runs its dictionary golden tests.
//!
//! A file `packs/<pack>/tests/<name>.csv` is a dictionary test when the pack
//! has a dictionary whose file stem is `<name>`. Other test files belong to
//! pack functions and are not run here.

use std::path::{Path, PathBuf};

use fita_pack::Pack;

const TEST_HEADER: [&str; 3] = ["input", "expected", "warnings"];

fn packs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs")
}

fn pack_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(packs_dir())
        .expect("packs/ directory is readable")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join(fita_pack::MANIFEST_FILE).is_file())
        .collect();
    roots.sort();
    roots
}

fn test_files(pack: &Pack) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(pack.root.join("tests")) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "csv"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_pack_loads() {
    let roots = pack_roots();
    assert!(!roots.is_empty(), "no packs found in {:?}", packs_dir());
    for root in roots {
        if let Err(err) = Pack::load(&root) {
            panic!("{err}");
        }
    }
}

#[test]
fn dictionary_golden_tests() {
    let mut failures = Vec::new();
    let mut cases = 0;

    for root in pack_roots() {
        let pack = Pack::load(&root).unwrap_or_else(|err| panic!("{err}"));
        for file in test_files(&pack) {
            let name = file.file_stem().unwrap().to_string_lossy();
            let Some(dict) = pack.dictionary(&name) else {
                continue;
            };

            let mut csv = csv::Reader::from_path(&file).unwrap();
            let header = csv.headers().unwrap();
            assert!(
                header.iter().eq(TEST_HEADER),
                "{}: header must be `{}`",
                file.display(),
                TEST_HEADER.join(",")
            );

            for record in csv.records() {
                let record = record.unwrap_or_else(|err| panic!("{}: {err}", file.display()));
                let line = record.position().map_or(0, |p| p.line());
                let (input, expected, warnings) = (&record[0], &record[1], &record[2]);
                cases += 1;

                let actual = dict.apply(input);
                if actual != expected {
                    failures.push(format!(
                        "{}:{line}: `{input}` gave `{actual}`, expected `{expected}`",
                        file.display()
                    ));
                }
                // Dictionaries never produce warnings.
                if !warnings.is_empty() {
                    failures.push(format!(
                        "{}:{line}: dictionary tests cannot expect warnings",
                        file.display()
                    ));
                }
            }
        }
    }

    assert!(cases > 0, "no dictionary test cases found");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
