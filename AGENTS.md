# Agent Instructions

## Working Boundaries
- Design/performance questions are discussion, not permission to implement. Preserve the concise README structure and simple pip/venv and Cargo commands; do not introduce uv or unrequested CLI flags.
- Scope is Malayalam whole numbers `0` through `999999999` and unsigned decimals. Signs, fractions, dates, currency formatting, and other languages are not implemented. This is sentence rewriting, not whole-input validation: unsupported phrases may partially normalize.
- `AGENTS.md` is deliberately git-ignored. Do not remove the ignore or force-add this file unless asked.

## Code Map
- Read `design.md` first: build flow, component lookup table, and key decisions. Add to it when you have to search for something that belongs there.
- `src/`: `lib.rs` (public `Normalizer`, `AppError`), `runtime.rs` (WFST execution), `bin/normalize.rs` (Rust CLI).
- `wfst/ml/`: Malayalam grammar package: `data/numbers.tsv` vocabulary, `rules/numbers/` grammar, `assets/numbers.fst` generated graph, `tests/` Python tests and shared fixture TSVs.
- `tests/`: Rust integration tests: `numbers.rs` and `decimals.rs` (fixtures/category checks), `number_composition.rs` (independent oracle), `sentences.rs` (sentences and CLI).
- `tools/`: `compile.py` builds/checks the asset; `normalize.py` runs one input against the saved asset.
- Root: `Cargo.toml`/`Cargo.lock` (Rust deps), `requirements.txt` (pinned Python deps), `pyproject.toml` (strict mypy config only).

## Execution And Assets
- `src/lib.rs` exposes `Normalizer::new("ml")` and embeds `wfst/ml/assets/numbers.fst`. Reuse the normalizer; only exact lowercase `ml` is accepted. Empty input works; NUL is rejected because label zero is epsilon.
- `src/runtime.rs` orchestrates UTF-8 byte acceptor -> compose -> epsilon removal -> shortest path -> checked UTF-8 output. `rustfst` implements the algorithms; runtime consumers need neither Python nor native OpenFst.
- Preserve the library error codes: `UNSUPPORTED_LANGUAGE`, `INVALID_INPUT`, `GRAPH_LOAD_FAILED`, `GRAPH_EXECUTION_FAILED`. These are not HTTP response envelopes.
- Python imports are `wfst.ml`, not `ml`. In `wfst/ml/rules/numbers/`, `cardinal.py` builds numbers, `decimal.py` reuses them, and `__init__.py` adds shared sentence boundaries and weights. The single `numbers.fst` contains both categories.
- `tools/compile.py` builds the graph, checks all shared fixtures, then writes the asset; it does not run the unit suite. `--check` rebuilds in memory and compares without writing. Cargo has no automatic grammar-compilation step.
- `tools/normalize.py` reads the saved asset only: no rebuilding, writing, or Rust invocation. Rust CLI: `cargo run -- "your sentence"`; Python CLI: `python tools/normalize.py "your sentence"`. Pass complete phrases/sentences as one quoted argument, not word by word.

## Build And Verify
- Python authoring is verified on Linux/WSL x86-64 with Python 3.12. Follow README setup, activate `.venv`, and install from `requirements.txt`; do not move dependencies into `pyproject.toml`.
- `mypy` is external, not in `requirements.txt`. Its configuration resolves Pynini through `.venv/bin/python`; a different active environment alone does not change that.
- For behavior changes, add a failing regression first. After grammar/data edits, run `python tools/compile.py` BEFORE saved-asset tests or Rust builds. Keep the generated asset with source changes, then rebuild Rust to refresh the embedded graph.
- Full checks after rebuilding when needed:

```sh
python tools/compile.py --check
python -m unittest discover -s wfst/ml/tests
mypy --strict tools wfst
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

- Python shared fixtures, including decimals: `python -m unittest wfst.ml.tests.test_numbers.NumbersTests.test_shared_fixtures`.
- Python decimals: `python -m unittest wfst.ml.tests.test_decimals`.
- Rust single fixture test: `cargo test --test numbers approved_numbers_and_preservation_fixtures`.
- Rust sentence/API/CLI coverage: `cargo test --test sentences`; decimal-specific checks: `cargo test --test decimals`.

## Tests And Grammar Traps
- `wfst/ml/data/numbers.tsv` is the only mapping TSV: three nonempty headerless columns, category/spoken/output. Categories encode different composition roles; do not flatten or split the vocabulary by rule.
- Shared fixtures are `wfst/ml/tests/numbers.tsv` and `decimals.tsv`: two nonempty headerless columns, spoken/expected. Python discovers fixture TSVs; Rust explicitly embeds each in its category test. Adding a TSV does not automatically add Rust coverage.
- Keep `numbers.tsv` fixtures to novel whole-number combinations, not copied mapping/plan examples or sentences. Boundary/preservation cases belong in test code, not entity-only fixture TSVs.
- `tests/sentences.rs` owns the combined sentence list, library checks, and CLI checks, including feeding normalized output back through the CLI. `tests/decimals.rs` is decimal-specific. These are regression examples, not a collected ASR corpus.
- Keep Python/Rust composition oracles independent of production vocabulary/grammar. Fresh-graph tests and saved-asset tests cover different failure modes; neither replaces the other. Compiler failure tests use temporary project copies, never corrupt the real bundled asset.
- Preserve fixed-width remainder padding and positive-only scale components; otherwise large numbers lose place values. Rewrite cost `1 - matched Unicode character count` favors complete phrases over partial rewrites.
- Components join across space/tab/NBSP, not LF/CRLF. Marks/joiners remain inside words, and connector punctuation protects identifiers. Escape literal rewrite inputs with `pynini.escape`, especially brackets.
- Spoken final `േ` for canonical `്` is accepted on complete known numbers everywhere, including standalone words in prose. Do not restore the old expectation that `നാലേ` stays unchanged. Never strip endings from arbitrary words or hardcode full-number aliases.
- That ending rule is applied after whole-number composition: endings inside scale multipliers such as `നാലേ ലക്ഷം` are not fully composed. Do not claim internal-component coverage without extending the rule and tests.
- Decimals require an integer and horizontal whitespace around `പോയിന്റ്` or `ദശാംശം`; fractional input is one whole-number reading or individual digits. Preserve all spoken fractional zeros. Signs, omitted integers, and mixed grouped/digit readings need separate agreement.

## Failure log
- Test spoken endings across standalone, spaced, fused, and multi-scale numbers with and without decimal markers; canonical fixtures alone do not establish colloquial coverage.
- Keep Rust sentence and CLI regressions together in `tests/sentences.rs` so the README's focused command covers every supported number category.
- Judge grammar output only on phrases Malayalam speakers actually say; faithful conversion of ambiguous input (e.g. `പത്ത് അഞ്ച്` -> `10 5`) is correct, not a bug.
