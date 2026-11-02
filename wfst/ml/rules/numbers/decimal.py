"""Compose unsigned decimals from existing whole-number and digit mappings."""

import pynini

from wfst.ml.rules.numbers.cardinal import SPACE, build_digits, table


def build_decimals(numbers: pynini.Fst) -> pynini.Fst:
    digits = build_digits()
    fraction = numbers | (digits + (SPACE + digits).star)
    return (
        numbers + SPACE + table("decimal_markers") + SPACE + fraction
    ).optimize()
