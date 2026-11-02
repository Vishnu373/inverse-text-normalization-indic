# Design

Inverse text normalization for Malayalam: spoken number words in a sentence become digits, and all other text is left as it is. Python/Pynini builds a weighted finite-state transducer (WFST) once; Rust embeds it and runs it without Python.

## Flow

```
numbers.tsv ──► cardinal.py ──► decimal.py ──► __init__.py ──► compile.py ──► numbers.fst
 (vocabulary)   (whole numbers)  (decimals)    (weights +       (fixtures,    (asset, committed)
                                                sentence        write)              │
                                                rewrite)                            ▼
                                       Rust: include_bytes! ──► runtime.rs ──► Normalizer::normalize
```

## Lookup

| Concept / component | File / dir | Purpose |
|---|---|---|
| Vocabulary | `wfst/ml/data/numbers.tsv` | Category, spoken form, digits; categories define each word's composition role |
| Whole numbers | `wfst/ml/rules/numbers/cardinal.py` | Composes digits up to crores with fixed-width padding; accepts spoken final `േ` on the complete number |
| Decimals | `wfst/ml/rules/numbers/decimal.py` | Whole number + `പോയിന്റ്`/`ദശാംശം` + a whole number or single digits |
| Sentence rewrite | `wfst/ml/rules/numbers/__init__.py` | Rewrite cost `1 - matched chars` favors complete phrases; Unicode word boundaries protect identifiers |
| Asset build | `tools/compile.py` | Builds the graph, checks fixtures, writes the asset; `--check` compares without writing |
| Compiled graph | `wfst/ml/assets/numbers.fst` | UTF-8 byte labels, tropical weights; label 0 is epsilon |
| Python manual run | `tools/normalize.py` | Runs one input against the saved asset |
| Public API | `src/lib.rs` | `Normalizer::new("ml")`, `normalize`, typed `AppError` codes; rejects NUL |
| Runtime | `src/runtime.rs` | Input acceptor → compose → epsilon removal → shortest path → UTF-8 output (`rustfst`) |
| Rust CLI | `src/bin/normalize.rs` | One sentence argument; exit 2 on usage errors, 1 on `AppError` |
| Shared fixtures | `wfst/ml/tests/numbers.tsv`, `decimals.tsv` | Spoken → expected; checked by Python, `compile.py` and Rust |
| Python tests | `wfst/ml/tests/` | Fresh grammar, saved asset, independent oracle, CLI tools |
| Rust tests | `tests/` | Fixtures, independent oracle, decimals, sentences and CLI |

## Key decisions

- **Build in Python, run in Rust.** Pynini is used only at build time, so Rust users need neither Python nor OpenFst.
- **One sentence-level graph.** Whole numbers and decimals are combined into one rewrite, so a sentence needs a single pass.
- **Spoken `േ` ending on complete numbers only.** It is applied after composition, so every number form accepts it; endings inside a scale multiplier (`നാലേ ലക്ഷം`) are not covered.
- **Independent test oracles.** Composition tests build expected output from their own vocabulary, not from the production grammar.
