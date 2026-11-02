"""Build the sentence-level Malayalam number graph."""

import sys
import unicodedata

import pynini
from pynini.lib import pynutil, utf8

from wfst.ml.rules.numbers.cardinal import build_numbers
from wfst.ml.rules.numbers.decimal import build_decimals


def build_graph() -> pynini.Fst:
    numbers = build_numbers()
    numbers |= build_decimals(numbers)
    # Cost 1 per rewrite minus matched characters favors complete scale phrases.
    input_cost = pynutil.add_weight(utf8.VALID_UTF8_CHAR, -1).star
    numbers = pynutil.add_weight(input_cost @ numbers, 1).optimize()

    # Marks and joiners stay inside words; connector punctuation protects IDs.
    boundaries = [
        pynini.escape(chr(code))
        for code in range(1, sys.maxunicode + 1)
        if (
            unicodedata.category(chr(code))[0] in "PSZ"
            and unicodedata.category(chr(code)) != "Pc"
        ) or chr(code).isspace()
    ]
    boundary = pynini.union(*boundaries).optimize()
    return pynini.cdrewrite(
        numbers,
        pynini.union("[BOS]", boundary),
        pynini.union("[EOS]", boundary),
        utf8.VALID_UTF8_CHAR.star,
    ).optimize()
