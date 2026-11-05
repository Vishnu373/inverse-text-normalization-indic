"""Build the sentence-level Malayalam number graph."""

import sys
import unicodedata

import pynini
from pynini.lib import pynutil, utf8

from wfst.ml.rules.numbers.cardinal import DIGIT, SPACE, build_numbers, table
from wfst.ml.rules.numbers.decimal import build_decimals
from wfst.ml.rules.numbers.fraction import build_fractions


def build_graph() -> pynini.Fst:
    whole_numbers = build_numbers()
    fractions = build_fractions(whole_numbers)
    # Cost 1 per rewrite minus matched characters favors complete scale phrases.
    input_cost = pynutil.add_weight(utf8.VALID_UTF8_CHAR, -1).star
    # Consume unjoined words literally; include written digits for idempotence.
    integer = whole_numbers | DIGIT.plus
    separator = pynini.project(SPACE, "input")
    fraction_words = table("fractions").project("input")
    adjacent = integer + separator + fraction_words
    # Separating whitespace must not let this guard displace a bar/decimal parse.
    unjoined = pynini.difference(
        pynini.project(adjacent, "input").optimize(),
        pynini.project(fractions, "input").optimize(),
    ) @ ((input_cost @ integer) + separator + (input_cost @ fraction_words))
    numbers = whole_numbers | build_decimals(whole_numbers) | fractions
    numbers = (
        pynutil.add_weight(input_cost @ numbers, 1) | pynutil.add_weight(unjoined, 1)
    ).optimize()

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
