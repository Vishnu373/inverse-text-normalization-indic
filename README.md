# Inverse Text Normalization for Indian languages

## 1. What The Project Is For

A Rust library that implements inverse text normalization (ITN). It targets Indian languages, starting with Malayalam: `അഞ്ച്` becomes `5`.

Python uses Pynini to combine Malayalam word mappings and grammar rules into a weighted finite-state transducer (WFST). Rust executes that compiled graph using `rustfst`.

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

### Step 3: Try Your Own Input

Pass your text as a quoted argument:

```sh
python tools/normalize.py "മുപ്പത്തിയേഴ്"
# Output: 37
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

`tests/sentences.rs` contains authored Malayalam examples, not collected ASR transcripts. They check complete number phrases, multiple numbers, surrounding text, whitespace, and mixed Malayalam-English input. 

### Step 6: Try A Full Sentence In Rust

Pass the entire sentence as one quoted argument, not word by word:

```sh
cargo run -- "എനിക്ക് നൂറ്റി ഇരുപത്തിയഞ്ച് രൂപ വേണം"
# Output: എനിക്ക് 125 രൂപ വേണം
```

The Rust command uses the embedded `numbers.fst`, prints the normalized text.
