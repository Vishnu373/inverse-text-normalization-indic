"""Compose number chunks, padding remainders to preserve their place values."""

import csv
from pathlib import Path

import pynini
from pynini.lib import pynutil

DATA = Path(__file__).parents[2] / "data" / "numbers.tsv"
DIGIT = pynini.union(*"0123456789")
POSITIVE = pynini.union(*"123456789") + DIGIT.star
SPACE = pynutil.delete(pynini.union(" ", "\t", "\u00a0").plus)


def padded(graph: pynini.Fst, width: int) -> pynini.Fst:
    padding = pynini.union(*(
        pynutil.insert("0" * (width - length)) + pynini.closure(DIGIT, length, length)
        for length in range(1, width + 1)
    ))
    return (graph @ padding).optimize()


def table(name: str) -> pynini.Fst:
    pairs: list[tuple[str, str]] = []
    with DATA.open(encoding="utf-8", newline="") as source:
        for line, row in enumerate(
            csv.reader(source, delimiter="\t", quoting=csv.QUOTE_NONE), start=1
        ):
            if len(row) != 3 or not all(row):
                raise ValueError(f"{DATA}:{line}: expected three nonempty TSV fields")
            category, spoken, written = row
            if category == name:
                pairs.append((spoken, written))
    if not pairs:
        raise ValueError(f"Unknown or empty number category: {name}")
    return pynini.string_map(pairs, input_token_type="byte", output_token_type="byte")


def build_numbers() -> pynini.Fst:
    digits = table("digits")
    digits |= table("teens")
    digits |= table("tens")
    digits |= table("tens_prefixes") + table("tens_units")
    small = (digits @ POSITIVE).optimize()

    digits |= table("hundreds")
    remainder = padded(small | table("hundred_contractions"), 2)
    digits |= table("hundred_prefixes") + SPACE.ques + remainder
    below_thousand = (digits @ POSITIVE).optimize()

    thousands = table("thousand_prefixes")
    digits |= thousands + pynini.cross("യിരം", "000")
    digits |= (
        thousands + pynutil.delete("യിരത്തി") + SPACE.ques
        + padded(below_thousand, 3)
    )
    large_thousands = (
        table("large_thousand_prefixes")
        | table("tens_prefixes") + table("thousand_units")
    )
    digits |= large_thousands + pynini.cross("യിരം", "000")
    digits |= (
        large_thousands + pynutil.delete("യിരത്തി") + SPACE.ques
        + padded(below_thousand, 3)
    )
    below_lakh = (digits @ POSITIVE).optimize()

    scale_count = small | pynini.cross("ഒരു", "1")
    digits |= scale_count + SPACE + pynini.cross("ലക്ഷം", "00000")
    digits |= (
        scale_count + SPACE + pynutil.delete("ലക്ഷത്തി") + SPACE
        + padded(below_lakh, 5)
    )
    below_crore = (digits @ POSITIVE).optimize()

    digits |= scale_count + SPACE + pynini.cross("കോടി", "0000000")
    digits |= (
        scale_count + SPACE + pynutil.delete("കോടി") + SPACE
        + padded(below_crore, 7)
    )
    return digits
