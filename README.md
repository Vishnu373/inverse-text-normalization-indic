# Inverse Text Normalization for Indian languages

## 1. What The Project Is For

A Rust library that implements inverse text normalization (ITN). It targets Indian languages, starting with Malayalam: `അഞ്ച്` becomes `5`.

Python uses Pynini to combine Malayalam word mappings and grammar rules into a weighted finite-state transducer (WFST). Rust executes that compiled graph using `rustfst`.

### 1.1 What Is ITN

Inverse text normalization converts spoken-form text, into standard written forms.

| Input | Output |
|-------|--------|
| എനിക്ക് നൂറ്റി ഇരുപത്തിയഞ്ച് രൂപ വേണം | എനിക്ക് 125 രൂപ വേണം |

## 2. Repositories Used

- [Open-Speech-EkStep/indic-punct](https://github.com/Open-Speech-EkStep/indic-punct): reference for Malayalam number vocabulary and Pynini grammar composition.
- [FluidInference/text-processing-rs](https://github.com/FluidInference/text-processing-rs): reference for Rust library structure and WFST integration patterns.

These are reference repositories, not runtime dependencies. This crate has its own shared Rust runtime using `rustfst`.

### 2.1 Why Build This

- `indic-punct` has partial Malayalam ITN support.
- `text-processing-rs` has a Rust runtime but no Malayalam ITN support.
- Build this for missing Malayalam support and a Rust runtime.

## 3. What Is Currently Being Added

| Type | Language |
|-------|--------|
| numbers | malayalam (ml) |
| decimals | malayalam (ml) |
| fractions | malayalam (ml) |

## 4. How To Set It Up Locally

For the Python steps, use Linux or WSL on x86-64 and Python 3.12 with pip and venv available. A current stable Rust toolchain is only needed for the Rust build.

Run from `inverse-text-normalization-indic/`:

### Step 1: Build The WFST With Python

Create and activate the Python environment:

```sh
python3.12 -m venv .venv
source .venv/bin/activate
```

Install the dependencies:

```sh
python -m pip install -r requirements.txt
```

Compile the rules into `wfst/ml/assets/numbers.fst`:

```sh
python tools/compile.py
```

### Step 2: Run The Python Tests

```sh
python -m unittest discover -s wfst/ml/tests
```

The compiler integration test rebuilds the full grammar five times; each subprocess has a 30-second timeout.

### Step 3: Try Your Own Input

Pass your text as a quoted argument:

```sh
python tools/normalize.py "മുപ്പത്തിയേഴ്"
# Output: 37
```

Try a decimal:

```sh
python tools/normalize.py "രണ്ട് പോയിന്റ് പൂജ്യം അഞ്ച്"
# Output: 2.05
```

Try a fraction:

```sh
python tools/normalize.py "മൂന്ന് ബാർ നാല്"
# Output: 3/4
```

This reads `wfst/ml/assets/numbers.fst` without rebuilding or modifying it, and prints the result.

### Step 4: Build The Rust Library

Rust embeds the compiled WFST and executes it without Python:

```sh
cargo build
cargo test
```

### Step 5: Run the sentence and command-line tests:

```sh
cargo test --test sentences
```

`tests/sentences.rs` contains Malayalam sentence regressions, not a collected ASR corpus. They check whole-number, decimal, and fraction phrases, multiple numbers, surrounding text, whitespace, and mixed Malayalam-English input.

### Step 6: Try A Full Sentence In Rust

Pass the entire sentence as one quoted argument, not word by word:

```sh
cargo run -- "എനിക്ക് നൂറ്റി ഇരുപത്തിയഞ്ച് രൂപ വേണം"
# Output: എനിക്ക് 125 രൂപ വേണം
```

Try a sentence containing a decimal:

```sh
cargo run -- "അളവ് മൂന്ന് ദശാംശം ഇരുപത്തഞ്ച് കിലോ."
# Output: അളവ് 3.25 കിലോ.
```

Try a sentence containing a mixed number:

```sh
cargo run -- "ഒന്നര കിലോ അരി വേണം."
# Output: 1 1/2 കിലോ അരി വേണം.
```

Fractions keep their spoken components without reduction: `രണ്ട് ബാർ നാല്` becomes `2/4`. Denominator-first forms such as `നാലിൽ മൂന്ന്` become `3/4`, with denominators limited to 1-100; bar forms use the full existing number range. Fused numerator-one forms also work: `മൂന്നിലൊന്ന്` becomes `1/3`, and `അഞ്ചിലൊന്ന്` becomes `1/5`. Other numerators still require whitespace.

Internally, denominator-first fractions use markers U+E000-U+E002 and a small post-processing step in both Python and Rust to swap the digit strings. This keeps the graph compact without reducing the supported number ranges. These markers are rejected in user input; malformed graph markers raise an error rather than appearing in the result.

Mixed numbers require explicit joins such as `രണ്ടര` or `രണ്ടേ കാൽ`. Plain adjacency preserves the word: `അഞ്ച് കാൽ മേശ` becomes `5 കാൽ മേശ`, not `5 1/4 മേശ`. Standalone `അര കിലോ` still becomes `1/2 കിലോ`.

Before lakh or crore, fractional counts use fixed digit endings: `രണ്ടര ലക്ഷം രൂപ` becomes `250000 രൂപ`, and `മൂന്നേ മുക്കാൽ കോടി` becomes `37500000`. Whole counts follow the existing 1-99 limit; bare `അര ലക്ഷം` becomes `50000`. Extra remainders and fractional lakh amounts inside crore expressions are not combined.

The Rust command uses the embedded `numbers.fst`, prints the normalized text.
