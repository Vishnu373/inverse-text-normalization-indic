# Contributing

Thank you for your interest in contributing.

## Scope

The project currently supports Malayalam (`ml`) whole numbers, unsigned decimals, and fractions. Please open an issue before starting work.

## What To Work On

In priority order:

1. **Fix errors**: wrong or missing conversions for supported Malayalam phrases.
2. **Add rules for Malayalam**: new categories such as dates, currency, or signed numbers.
3. **Add a new language**: a new grammar package under `wfst/<language code>/` with its own rules and tests.

## Setup

Follow [How To Set It Up Locally](README.md#5-how-to-set-it-up-locally) in the README. Read [design.md](design.md) for how the grammar, compiled graph, and Rust runtime fit together.

## Making A Change

1. Add a failing test first:
   - New expected conversions go in `wfst/ml/tests/numbers.tsv`, `decimals.tsv`, or `fractions.tsv`: one `spoken<TAB>expected` pair per line.
   - Sentence-level cases go in `tests/sentences.rs`.
2. Edit the vocabulary in `wfst/ml/data/numbers.tsv` or the rules in `wfst/ml/rules/numbers/`.
3. Rebuild the graph with `python tools/compile.py` and commit the regenerated `wfst/ml/assets/numbers.fst` with your source changes.

## Before Opening A Pull Request

Run all checks and make sure they pass. `mypy` is installed separately; it is not in `requirements.txt`.

```sh
python tools/compile.py --check
python -m unittest discover -s wfst/ml/tests
mypy --strict tools wfst
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

## Versioning

Releases follow [semantic versioning](https://semver.org/). While the crate is `0.x`, bump the patch number (`0.1.1`) for compatible changes such as bug fixes, vocabulary, and grammar improvements. Bump the minor number (`0.2.0`) for anything that can break users' code, including adding an `AppError` variant or changing a public function.
