# Dictionary sources

Every dictionary in this folder, where it comes from and under which terms it is redistributed. All files follow the format in [docs/PACK_FORMAT.md](../../../docs/PACK_FORMAT.md).

| File | Content | Source | License |
|------|---------|--------|---------|
| `legal_forms.csv` | Legal form abbreviations and full names in Spanish and Catalan, mapped to a short code (`SL`, `SA`, `SCCL`...) | Written by hand for this pack from the names of the legal forms in Spanish and Catalan company and cooperative law. No text is copied. | Apache-2.0, same as this repository |
| `stopwords.csv` | Articles, prepositions and conjunctions in Spanish, Catalan, Galician and Basque that are dropped from company names. The canonical column is empty, which removes the word. | Written by hand for this pack. | Apache-2.0, same as this repository |
| `street_types.csv` | Street types and their usual abbreviations in Spanish, Catalan, Galician and Basque, mapped to the Spanish name | Written by hand for this pack from common postal usage. | Apache-2.0, same as this repository |
| `provinces.csv` | The 52 provinces and autonomous cities keyed by the first two digits of the postcode | Province codes published by the Instituto Nacional de Estadística (INE). The first two digits of a Spanish postcode are the INE province code. Names in uppercase without accents on vowels, using the official bilingual name where there is one. | Public administrative data, reusable under Ley 37/2007 on the reuse of public sector information |

## Notes

- Basque puts the street type after the name (`Nagusia kalea`). The dictionary replaces the word in place and does not move it.
- `PG.` maps to `PASEO` (Catalan `passeig`). Industrial estates use `POL.` instead.
