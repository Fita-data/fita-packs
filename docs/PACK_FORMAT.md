# Pack format

A pack is a folder under `packs/<country code>/`. It must be self contained and deterministic. The same input gives the same output on every machine and every run.

## pack.yaml

```yaml
pack: es
version: 0.1.0
country: ES
fita: ">=0.1, <0.3"          # compatible engine versions
extends:
  tax_id:
    validator: es_tax_id      # built in or from src/
    strip_prefix: [ES]
  company_name:
    legal_forms: dictionaries/legal_forms.csv
    stopwords: dictionaries/stopwords.csv
  address:
    street_types: dictionaries/street_types.csv
    postcode: { pattern: "^[0-9]{5}$", province_from_prefix: dictionaries/provinces.csv }
  phone:
    default_region: ES
```

## Dictionaries

Plain CSV, UTF-8, comma separated, with a header row. One mapping per row, variant first and canonical value second. Keep them sorted so diffs stay readable.

```csv
variant,canonical
C/,CALLE
CARRER,CALLE
AVDA.,AVENIDA
AVINGUDA,AVENIDA
```

Record where each dictionary comes from in `dictionaries/SOURCES.md`. Only use public data with a license that allows redistribution.

## Functions

Core types already live in the engine. A pack only adds functions when a dictionary is not enough. Pack functions are written in Rust (or any language that compiles to WebAssembly) and run in wasmtime with no network, no file system and limits on memory and time.

The interface is one function per normalizer or validator. It receives a string and returns a normalized string plus a list of warnings. No global state.

## Tests

Every pack has `tests/<type>.csv` with three columns: input, expected normalized value and expected warnings (empty if none). CI fails if any row does not match.

Write cases for the tricky inputs, not only the happy path. Lowercase, accents, extra spaces, missing parts, regional language variants, prefixes, wrong check digits.

All test values are invented. Tax ids in tests must be valid by algorithm but must not belong to a real company or person.

## Spain pack reference

- **NIF.** 8 digits and a letter. The letter is `TRWAGMYFPDXBNJZSQVHLCKE[number % 23]`.
- **NIE.** Starts with X, Y or Z. Replace with 0, 1 or 2 and apply the NIF rule.
- **CIF.** Entity letter, 7 digits and a control character. Sum the digits in even positions, add the digit sum of double each odd position digit, take `(10 - total % 10) % 10`. Depending on the entity letter the control is that digit, the letter `JABCDEFGHI[digit]`, or either.
- **Company names.** Uppercase, strip accents and punctuation, extract the legal form (S.L., S.L.U., S.A., S.A.U., S.C.C.L., S.L.L., A.I.E., C.B., S.C., cooperativa) into `legal_form`, drop stopwords, keep a phonetic key.
- **Addresses.** Street types in Spanish, Catalan, Galician and Basque. Five digit postcode. Province from the first two digits.
- **Phones.** E.164 with ES as the default region, mark mobile or landline.
