"""Compose exact fractions from the existing whole-number graph."""

import pynini
from pynini.lib import pynutil, utf8

from wfst.ml.rules.numbers.cardinal import DIGIT, POSITIVE, SPACE, table


def build_fractions(numbers: pynini.Fst) -> pynini.Fst:
    fractions = table("fractions")

    # Restore the final virama in fused half forms, such as രണ്ടര -> രണ്ട് + അര.
    stem = (utf8.VALID_UTF8_CHAR.star + pynutil.insert("്")) @ numbers
    half = pynini.cross("ര", "1/2")
    fused_half = stem + pynutil.insert(" ") + half

    long_e = (utf8.VALID_UTF8_CHAR.star + "േ") @ numbers
    quarters = fractions @ pynini.union("1/4", "3/4")
    mixed_quarters = long_e + SPACE.ques + pynutil.insert(" ") + quarters
    mixed = fused_half | mixed_quarters

    count = pynini.union(*"123456789") + DIGIT.ques
    scaled = pynini.Fst()
    # Keep the whole prefix; the scale word emits the fixed fractional digits.
    for value, category in (
        ("1/2", "half_scales"), ("1/4", "quarter_scales"),
        ("3/4", "three_quarter_scales"),
    ):
        prefix = mixed @ (count + pynutil.delete(f" {value}"))
        bare = pynutil.delete(pynini.project(fractions @ value, "input"))
        scaled |= (prefix | bare) + SPACE + table(category)

    bar = numbers + SPACE + table("fraction_markers") + SPACE + (numbers @ POSITIVE)

    denominators = (
        (utf8.VALID_UTF8_CHAR.star + pynini.cross("ിൽ", "്"))
        @ numbers @ pynini.union(*(str(value) for value in range(1, 101)))
    ).optimize()
    # Restore the denominator ending in fused forms such as മൂന്നിലൊന്ന്.
    fused_one = (
        (utf8.VALID_UTF8_CHAR.star + pynini.cross("ിലൊന്ന്", "ിൽ")) @ denominators
    ) + pynutil.insert("\ue001") + pynutil.insert("1")
    # Emit spoken order; post-processing swaps the marked digit strings.
    denominator_first = (
        pynutil.insert("\ue000") + (
            denominators + pynutil.insert("\ue001") + SPACE + numbers | fused_one
        ) + pynutil.insert("\ue002")
    )

    return (
        fractions | mixed | scaled | bar | denominator_first
    ).optimize()
