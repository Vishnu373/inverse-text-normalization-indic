"""Normalize one input using the compiled Malayalam WFST, without Rust."""

import argparse
import sys
from pathlib import Path

import pynini
from pynini.lib import rewrite

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from wfst.ml.postprocess import reorder_fractions


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("text", help="Text to normalize; wrap it in quotes")
    arguments = parser.parse_args()
    if any(marker in arguments.text for marker in ("\0", "\ue000", "\ue001", "\ue002")):
        parser.error("NUL and fraction markers U+E000-U+E002 are reserved")
    asset = ROOT / "wfst" / "ml" / "assets" / "numbers.fst"
    if not asset.is_file():
        parser.error("numbers.fst is missing; run python tools/compile.py first")
    graph = pynini.Fst.read(str(asset))
    print(reorder_fractions(rewrite.one_top_rewrite(
        pynini.escape(arguments.text), graph,
        input_token_type="byte", output_token_type="byte",
    )))


if __name__ == "__main__":
    main()
