# fita-packs

Country packs and issue detectors for [Fita](https://github.com/Fita-data/fita).

A pack teaches Fita how identifiers, company names, addresses and phones look in one country. It normalizes values, validates them and adds the derived attributes the matching rules use. Packs live outside the engine so a new country can ship without a new Fita release.

## Layout

```
packs/
  es/                 Spain (first pack)
    pack.yaml         manifest
    dictionaries/     CSV reference tables
    src/              optional functions (Rust compiled to WebAssembly)
    tests/            golden input and expected output
  pt/ fr/ de/         planned
detectors/            issue detectors that are not country specific
crates/
  fita-pack           loader and validator for pack.yaml, shared by Fita and CI
docs/
  PACK_FORMAT.md      how a pack is built
  ROADMAP.md          planned work
```

## Using a pack

In a Fita project:

```yaml
# fita.yaml
packs:
  - es@0.1
```

## Writing a pack

Read [docs/PACK_FORMAT.md](docs/PACK_FORMAT.md). Every function needs golden tests, and CI runs them before a pack is published.

## License

Apache License 2.0. Dictionaries built from public sources keep a note of their source and license in `dictionaries/SOURCES.md`.
