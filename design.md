# Design

Inverse text normalization for Malayalam: spoken number words in a sentence become digits, and all other text is left as it is. Python/Pynini builds a weighted finite-state transducer (WFST) once; Rust embeds it and runs it without Python.

## Flow

```
numbers.tsv --> cardinal.py --+--> whole numbers --+
 (vocabulary)                +--> decimal.py -----+--> __init__.py --> compile.py --> numbers.fst
                             +--> fraction.py ----+     (rewrite)      (fixtures,        |
                                                                        write)          v
                                            Rust: include_bytes! --> runtime.rs --> reorder_fractions --> Normalizer::normalize
```

## Lookup

| Concept / component | File / dir | Purpose |
|---|---|---|
| Vocabulary | `wfst/ml/data/numbers.tsv` | Category, spoken form, digits; categories define each word's composition role |
| Whole numbers | `wfst/ml/rules/numbers/cardinal.py` | Composes digits up to crores with fixed-width padding; accepts spoken final `േ` on the complete number |
| Decimals | `wfst/ml/rules/numbers/decimal.py` | Whole number + `പോയിന്റ്`/`ദശാംശം` + a whole number or single digits |
| Fractions | `wfst/ml/rules/numbers/fraction.py` | Mixed, bar, denominator-first, and fixed-suffix lakh/crore forms; no runtime arithmetic |
| Sentence rewrite | `wfst/ml/rules/numbers/__init__.py` | Complete-phrase weights, Unicode boundaries, and protection for unjoined number/fraction-word sequences |
| Asset build | `tools/compile.py` | Builds the graph, checks fixtures, writes the asset; `--check` compares without writing |
| Compiled graph | `wfst/ml/assets/numbers.fst` | UTF-8 byte labels, tropical weights; label 0 is epsilon |
| Fraction output decoding | `wfst/ml/postprocess.py`, `src/lib.rs` | Validate internal markers and swap denominator-first digit strings after graph execution |
| Python manual run | `tools/normalize.py` | Runs one input against the saved asset |
| Public API | `src/lib.rs` | `Normalizer::new("ml")`, `normalize`, typed `AppError` codes; rejects NUL and reserved fraction markers |
| Runtime | `src/runtime.rs` | Input acceptor → compose → epsilon removal → shortest path → UTF-8 output (`rustfst`) |
| Rust CLI | `src/bin/normalize.rs` | One sentence argument; exit 2 on usage errors, 1 on `AppError` |
| Shared fixtures | `wfst/ml/tests/numbers.tsv`, `decimals.tsv`, `fractions.tsv` | Spoken → expected; checked by Python, `compile.py` and Rust |
| Python tests | `wfst/ml/tests/` | Fresh grammar, saved asset, independent oracle, CLI tools |
| Rust tests | `tests/` | Fixtures, independent oracles, decimals, fractions, sentences and CLI |

## Key decisions

- **Build in Python, run in Rust.** Pynini is used only at build time, so Rust users need neither Python nor OpenFst.
- **One sentence-level graph.** Whole numbers, decimals, and fractions are combined into one WFST rewrite, followed by a linear fraction-marker decoding step. Decimal and fraction builders receive only the whole-number graph, not each other's categories.
- **Exact fractions without runtime arithmetic.** Bar operands use the full whole-number range with a positive denominator. Denominator-first forms keep denominators 1-100 and full-range spaced numerators, emitting `\uE000denominator\uE001numerator\uE002`. Fused `-ിലൊന്ന്` forms reuse the bounded denominator rule and emit numerator 1, so `മൂന്നിലൊന്ന്` becomes `1/3`; other numerators still require whitespace. Python and Rust swap these digit strings into `numerator/denominator`, avoiding a full numerator-grammar copy per denominator. Preserve `2/4`, `5/3`, and `0/7` as spoken.
- **Reserved fraction markers.** Reject U+E000-U+E002 in user input; private-use characters can occur in real text. Malformed graph markers or empty/non-ASCII-digit operands fail loudly (`ValueError` in Python, `GRAPH_EXECUTION_FAILED` in Rust). Public normalizers never return markers. Other private-use characters remain unchanged, and shared fixtures cover both implementations.
- **Explicit mixed joins only.** Reuse whole numbers for fused halves and fused/spaced long-e quarter joins. A sentence-level protection consumes plain spoken/written integer + fraction-word sequences while preserving the word and spacing, excluding valid mixed inputs. Its score excludes separating whitespace so long gaps cannot displace complete bar/decimal parses. This also preserves normalized-output idempotence; standalone fractions elsewhere still convert.
- **Fixed fractional scale endings.** Fractional lakh/crore amounts reuse those mixed forms with whole counts 1-99, or a bare common fraction. The scale mapping emits the fixed half/quarter/three-quarter suffix after the whole prefix, without runtime arithmetic. Vocabulary stays in `numbers.tsv`; no remainder addition, fractional lakh inside crore composition, or promotion to decimal/bar operands.
- **Distinct Malayalam spellings.** `കാൽ` is the quarter mapping; `കാല്` is not a fraction alias and remains prose.
- **Spoken `േ` ending on complete numbers only.** It is applied after composition, so every number form accepts it; endings inside a scale multiplier (`നാലേ ലക്ഷം`) are not covered.
- **Independent test oracles.** Composition tests build expected output from their own vocabulary, not from the production grammar.
