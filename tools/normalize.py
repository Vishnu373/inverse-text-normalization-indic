"""Normalize one input using the compiled Malayalam WFST, without Rust."""

import argparse
from pathlib import Path

import pynini
from pynini.lib import rewrite


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("text", help="Text to normalize; wrap it in quotes")
    arguments = parser.parse_args()
    asset = Path(__file__).resolve().parents[1] / "wfst" / "ml" / "assets" / "numbers.fst"
    if not asset.is_file():
        parser.error("numbers.fst is missing; run python tools/compile.py first")
    graph = pynini.Fst.read(str(asset))
    print(rewrite.one_top_rewrite(
        pynini.escape(arguments.text), graph,
        input_token_type="byte", output_token_type="byte",
    ))


if __name__ == "__main__":
    main()
