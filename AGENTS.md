# Agent Instructions

## Working Boundaries
- Design/performance questions are discussion, not permission to implement. Preserve the concise README structure and simple pip/venv and Cargo commands; do not introduce uv or unrequested CLI flags.
- Scope is Malayalam whole numbers `0` through `999999999`, unsigned decimals, and fractions. Signs, dates, currency formatting, and other languages are not implemented. This is sentence rewriting, not whole-input validation: unsupported phrases may partially normalize.

## Code Map
- Read `design.md` first: build flow, component lookup table, and key decisions. Add to it when you have to search for something that belongs there.
- `src/`: `lib.rs` (public `Normalizer`, `AppError`), `runtime.rs` (WFST execution), `bin/normalize.rs` (Rust CLI).
- `wfst/ml/`: Malayalam grammar package: `data/numbers.tsv` vocabulary, `rules/numbers/` grammar, `postprocess.py` fraction-marker decoding, `assets/numbers.fst` generated graph, `tests/` Python tests and shared fixture TSVs.
- `tests/`: Rust integration tests: `numbers.rs`, `decimals.rs`, and `fractions.rs` (fixtures/category checks), `number_composition.rs` (independent oracle), `sentences.rs` (sentences and CLI).
- `tools/`: `compile.py` builds/checks the asset; `normalize.py` runs one input against the saved asset.
- Root: `CONTRIBUTING.md` (contributor workflow and versioning), `Cargo.toml`/`Cargo.lock` (Rust deps), `requirements.txt` (pinned Python deps), `pyproject.toml` (strict mypy config only).

## Execution And Assets
- `src/lib.rs` exposes `Normalizer::new("ml")` and embeds `wfst/ml/assets/numbers.fst`. Reuse the normalizer; only exact lowercase `ml` is accepted. Empty input works; NUL is rejected because label zero is epsilon. U+E000-U+E002 are reserved fraction markers and also rejected as `INVALID_INPUT`; other private-use characters are preserved.
- `src/runtime.rs` orchestrates UTF-8 byte acceptor -> compose -> epsilon removal -> shortest path -> checked UTF-8 output. `rustfst` implements the algorithms; runtime consumers need neither Python nor native OpenFst.
- Denominator-first graph output is `\uE000denominator\uE001numerator\uE002`. Always call `reorder_fractions` after graph rewriting: `src/lib.rs` does this after `runtime.apply()`, and `wfst/ml/postprocess.py` serves the Python CLI, compiler fixtures, and direct-graph tests. Missing, stray, nested, or out-of-order markers and empty/non-ASCII-digit operands are errors, not literal output.
- Preserve the library error codes: `UNSUPPORTED_LANGUAGE`, `INVALID_INPUT`, `GRAPH_LOAD_FAILED`, `GRAPH_EXECUTION_FAILED`. These are not HTTP response envelopes.
- Python imports are `wfst.ml`, not `ml`. In `wfst/ml/rules/numbers/`, `cardinal.py` builds numbers; `decimal.py` and `fraction.py` independently reuse them; `__init__.py` adds shared sentence boundaries and weights. The single `numbers.fst` contains all three categories.
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

- Python shared fixtures, including decimals and fractions: `python -m unittest wfst.ml.tests.test_numbers.NumbersTests.test_shared_fixtures`.
- Python decimals: `python -m unittest wfst.ml.tests.test_decimals`.
- Python fractions: `python -m unittest wfst.ml.tests.test_fractions`; Rust fractions: `cargo test --test fractions`.
- Rust single fixture test: `cargo test --test numbers approved_numbers_and_preservation_fixtures`.
- Rust sentence/API/CLI coverage: `cargo test --test sentences`; decimal-specific checks: `cargo test --test decimals`.

## Tests And Grammar Traps
- `wfst/ml/data/numbers.tsv` is the only mapping TSV: three nonempty headerless columns, category/spoken/output. Categories encode different composition roles; do not flatten or split the vocabulary by rule.
- Shared fixtures are `wfst/ml/tests/numbers.tsv`, `decimals.tsv`, and `fractions.tsv`: two nonempty headerless columns, spoken/expected. Python discovers fixture TSVs; Rust explicitly embeds each in its category test. Adding a TSV does not automatically add Rust coverage.
- Keep `numbers.tsv` fixtures to novel whole-number combinations, not copied mapping/plan examples or sentences. Boundary/preservation cases belong in test code, not entity-only fixture TSVs.
- `tests/sentences.rs` owns the combined sentence list, library checks, and CLI checks, including feeding normalized output back through the CLI. `tests/decimals.rs` and `tests/fractions.rs` are category-specific. These are regression examples, not a collected ASR corpus.
- Keep Python/Rust composition oracles independent of production vocabulary/grammar. Fresh-graph tests and saved-asset tests cover different failure modes; neither replaces the other. Compiler failure tests use temporary project copies, never corrupt the real bundled asset.
- Keep the bundled graph below 10,000 states and compiler-test subprocess timeouts at 30 seconds. Denominator-first output-range tests validate internal marker format; shared fixtures validate final decoded fractions in both languages.
- Preserve fixed-width remainder padding and positive-only scale components; otherwise large numbers lose place values. Rewrite cost `1 - matched Unicode character count` favors complete phrases over partial rewrites.
- Components join across space/tab/NBSP, not LF/CRLF. Marks/joiners remain inside words, and connector punctuation protects identifiers. Escape literal rewrite inputs with `pynini.escape`, especially brackets.
- Spoken final `േ` for canonical `്` is accepted on complete known numbers everywhere, including standalone words in prose. Do not restore the old expectation that `നാലേ` stays unchanged. Never strip endings from arbitrary words or hardcode full-number aliases.
- That ending rule is applied after whole-number composition: endings inside scale multipliers such as `നാലേ ലക്ഷം` are not fully composed. Do not claim internal-component coverage without extending the rule and tests.
- Decimals require an integer and horizontal whitespace around `പോയിന്റ്` or `ദശാംശം`; fractional input is one whole-number reading or individual digits. Preserve all spoken fractional zeros. Signs, omitted integers, and mixed grouped/digit readings need separate agreement.
- Fractions preserve components without reduction, including `0/7`, `5/3`, and `2/4`. `ബാർ` uses full-range whole-number operands with a positive denominator; denominator-first readings use denominators 1-100 and whitespace before the numerator, except fused `-ിലൊന്ന്` fixes the numerator at 1. Mixed joins restore final virama for fused halves and reuse long-e numbers for fused/spaced quarters. Plain integer/fraction-word adjacency preserves the word and whitespace. Do not feed a fraction-augmented graph into the decimal builder.
- Fractional lakh/crore amounts concatenate a bounded whole prefix (1-99) and a fixed scale ending from `numbers.tsv`; bare common fractions emit just the ending. Keep remainder addition and fractional lakh inside crore expressions out of this rule, and do not promote these forms into decimal/bar operands.

## Failure log
- Test spoken endings across standalone, spaced, fused, and multi-scale numbers with and without decimal markers; canonical fixtures alone do not establish colloquial coverage.
- Keep Rust sentence and CLI regressions together in `tests/sentences.rs` so the README's focused command covers every supported number category.
- Judge grammar output only on phrases Malayalam speakers actually say; faithful conversion of ambiguous input (e.g. `പത്ത് അഞ്ച്` -> `10 5`) is correct, not a bug.
- Keep fraction `കാൽ` distinct from leg `കാല്`; do not add the latter as a quarter alias.
- Keep denominator-first operands in spoken order with reserved markers and swap their digit strings after rewriting; do not duplicate the full numerator grammar per denominator or hide that cost by raising the 30-second compiler-test timeout.
- Require explicit mixed joins and protect adjacent fraction words after spoken and written integers; removing the broad mixed branch alone does not stop standalone-fraction rewrites.
- Exclude separating whitespace from adjacency-guard scores so long gaps cannot displace complete bar/decimal parses.
- Do not use blocking `sleep` commands to poll the long-running Python grammar tests; check progress directly and state which checks passed and which are still running.
- Record mistakes and user corrections in this failure log as concise, actionable rules before continuing the affected work.
- Check monitoring-tool options for the background Python suite: `pidwait` matches command patterns and does not support `-p PID`.
- Match fractional lakh/crore amounts before rewriting standalone mixed numbers; use the existing mixed joins and fixed digit endings instead of emitting `2 1/2 ലക്ഷം` for `രണ്ടര ലക്ഷം`.
- Do not block the conversation with `pidwait` or other waits for the full Python suite; report the fix and focused results promptly, leave slow checks in the background, and label full verification pending until they finish.
- Map the fused-half suffix `ര` directly to `1/2`; do not filter and project the fractions table just to recover a constant output.
- Accept fused numerator-one readings such as `മൂന്നിലൊന്ന്` through the existing bounded denominator rule and marker decoder; keep other denominator-first numerators space-separated and protect word boundaries.
- Document crates.io installation and public API usage for application developers; keep local checkout setup separate. Do not add "not yet published" notes to README: it ships in the crate and freezes into the release.
- Anchor package include paths such as `/LICENSE` at the crate root to avoid bundling matching files from `.venv`.
- Run package verification separately from Rust tests when sharing `target/`; package builds can invalidate artifacts needed by doctests.
- Keep `Normalizer` debug formatting graph-free so application structs can derive `Debug` without dumping the WFST.
