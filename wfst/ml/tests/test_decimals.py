"""Decimal behavior using an independent vocabulary and explicit expected digits."""

import unittest
from typing import ClassVar

import pynini
from pynini.lib import rewrite

from wfst.ml.rules.numbers import build_graph
from wfst.ml.rules.numbers.cardinal import build_numbers
from wfst.ml.rules.numbers.decimal import build_decimals
from wfst.ml.tests.test_composition import SMALL, spoken


class DecimalTests(unittest.TestCase):
    graph: ClassVar[pynini.Fst]

    @classmethod
    def setUpClass(cls) -> None:
        cls.graph = build_graph()

    def check(self, text: str, expected: str) -> None:
        with self.subTest(text=text):
            actual = rewrite.one_top_rewrite(pynini.escape(text), self.graph)
            self.assertEqual(actual, expected)
            self.assertEqual(
                rewrite.one_top_rewrite(pynini.escape(actual), self.graph), actual,
            )

    def test_both_markers_and_grouped_fractional_numbers(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            for integer in (0, 37, 123456789, 999999999):
                for fraction in (0, 9, 10, 25, 101, 1001, 100000, 10000001, 999999999):
                    self.check(
                        f"{spoken(integer)} {marker} {spoken(fraction)}",
                        f"{integer}.{fraction}",
                    )

    def test_fractional_digits_preserve_zeros_and_order(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            for digits in [f"{value:02d}" for value in range(100)] + [
                "0", "0050", "000000", "12345678901234567890",
            ]:
                fraction = " ".join(SMALL[int(digit)] for digit in digits)
                self.check(f"പന്ത്രണ്ട് {marker} {fraction}", f"12.{digits}")

    def test_reported_spoken_decimal(self) -> None:
        self.check(
            "നിങ്ങളുടെ ഷുഗർ തൊണ്ണൂറ്റി ഏഴേ ദശാംശം നാലേ അഞ്ച് ആണ്",
            "നിങ്ങളുടെ ഷുഗർ 97.45 ആണ്",
        )
        self.check(
            "മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ ദശാംശം രണ്ട് ഒന്ന് എട്ട് ഒന്ന് രണ്ട് നാല്",
            "325.218124",
        )

    def test_compound_spoken_endings_reuse_whole_number_rules(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            for word, value in (
                ("അഞ്ചേ", "5"), ("പതിനഞ്ചേ", "15"), ("ഇരുപത്തിയഞ്ചേ", "25"),
                ("തൊണ്ണൂറ്റൊമ്പതേ", "99"), ("നൂറേ", "100"),
                ("മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "325"),
                ("ആയിരത്തി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "1325"),
                ("രണ്ട് ലക്ഷത്തി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "200325"),
                ("ഒരു കോടി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "10000325"),
            ):
                self.check(f"{word} {marker} രണ്ട് ഒന്ന് എട്ട് ഒന്ന് രണ്ട് നാല്", f"{value}.218124")
                self.check(f"രണ്ട് {marker} {word}", f"2.{value}")
                if " " not in word:
                    self.check(word, value)

    def test_spoken_endings_apply_to_all_nonzero_fractional_digits(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            for word, digit in (
                ("ഒന്നേ", "1"), ("രണ്ടേ", "2"), ("മൂന്നേ", "3"),
                ("നാലേ", "4"), ("അഞ്ചേ", "5"), ("ആറേ", "6"),
                ("ഏഴേ", "7"), ("എട്ടേ", "8"), ("ഒമ്പതേ", "9"),
            ):
                self.check(f"പന്ത്രണ്ട് {marker} പൂജ്യം {word} പൂജ്യം", f"12.0{digit}0")
                self.check(f"{word} {marker} {word}", f"{digit}.{digit}")
                self.check(f"{word}x {marker} അഞ്ച്", f"{word}x {marker} 5")
                self.check(f"രണ്ട് {marker} {word}_id", f"2 {marker} {word}_id")
        self.check("ഇരുപത്തിയഞ്ചേ\nദശാംശം അഞ്ച്", "25\nദശാംശം 5")
        self.check("പന്ത്രണ്ട് ദശാംശം രണ്ട് ഒന്നേ എട്ട് ഒന്നേ രണ്ട് നാല്", "12.218124")

    def test_spoken_digit_variants_are_compositional(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            for integer, expected_integer in (
                ("പന്ത്രണ്ട്", "12"), ("നാലേ", "4"), ("ഏഴേ", "7"),
                ("മുപ്പത്തി നാലേ", "34"), ("നൂറ്റി തൊണ്ണൂറ്റി ഏഴേ", "197"),
            ):
                for fraction, expected_fraction in (
                    ("നാലേ അഞ്ച്", "45"), ("ഏഴേ നാലേ", "74"),
                    ("പൂജ്യം നാലേ പൂജ്യം", "040"), ("നാലേ", "4"),
                    ("അറുപത്തി ഏഴേ", "67"),
                ):
                    self.check(
                        f"{integer} {marker} {fraction}",
                        f"{expected_integer}.{expected_fraction}",
                    )

    def test_spoken_digit_variants_respect_word_boundaries(self) -> None:
        for variant, digit in (("നാലേ", "4"), ("ഏഴേ", "7")):
            self.check(variant, digit)
            self.check(f"ഇവിടെ {variant} ഉള്ളൂ", f"ഇവിടെ {digit} ഉള്ളൂ")
            for affix in ("x", "_", "\u200d", "\u0301"):
                self.check(
                    f"രണ്ട് ദശാംശം {variant}{affix}",
                    f"2 ദശാംശം {variant}{affix}",
                )
                self.check(
                    f"{affix}{variant} ദശാംശം അഞ്ച്",
                    f"{affix}{variant} ദശാംശം 5",
                )
        self.check("തൊണ്ണൂറ്റി\nഏഴേ ദശാംശം നാലേ അഞ്ച്", "തൊണ്ണൂറ്റി\n7.45")
        self.check("തൊണ്ണൂറ്റി ഏഴേ ദശാംശം\nനാലേ അഞ്ച്", "97 ദശാംശം\n4 5")

    def test_decimal_grammar_is_functional(self) -> None:
        deterministic = pynini.determinize(
            build_decimals(build_numbers()), det_type="functional",
        )
        self.assertGreater(deterministic.num_states(), 0)

    def test_decimal_output_range(self) -> None:
        outputs = build_decimals(build_numbers()).project("output").rmepsilon().optimize()
        digit = pynini.union(*"0123456789")
        integer = pynini.union(
            "0", pynini.union(*"123456789") + pynini.closure(digit, 0, 8),
        )
        expected = (integer + "." + digit.plus).optimize()
        self.assertTrue(pynini.equivalent(outputs, expected))

    def test_spacing_and_line_boundaries(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            for separator in (" ", "  ", "\t", "\u00a0", " \t\u00a0"):
                self.check(
                    f"  രണ്ട്{separator}{marker}{separator}അഞ്ച്{separator}പൂജ്യം\n",
                    "  2.50\n",
                )
            for separator in ("\n", "\r\n"):
                self.check(
                    f"രണ്ട്{separator}{marker} അഞ്ച്", f"2{separator}{marker} 5",
                )
                self.check(
                    f"രണ്ട് {marker}{separator}അഞ്ച്", f"2 {marker}{separator}5",
                )
                self.check(f"രണ്ട് {marker} അഞ്ച്{separator}പൂജ്യം", f"2.5{separator}0")

    def test_boundaries_and_incomplete_phrases(self) -> None:
        for marker in ("പോയിന്റ്", "ദശാംശം"):
            self.check(f"അളവ് [രണ്ട് {marker} അഞ്ച്].", "അളവ് [2.5].")
            self.check(marker, marker)
            self.check(f"രണ്ട് {marker}", f"2 {marker}")
            self.check(f"{marker} അഞ്ച്", f"{marker} 5")
            self.check(f"രണ്ട്, {marker} അഞ്ച്", f"2, {marker} 5")
            self.check(f"രണ്ട് {marker}, അഞ്ച്", f"2 {marker}, 5")
            for affix in ("a", "മ", "7", "_", "\u203f", "\u200c", "\u200d", "\u0301"):
                self.check(f"{affix}രണ്ട് {marker} അഞ്ച്", f"{affix}രണ്ട് {marker} 5")
                self.check(f"രണ്ട് {marker} അഞ്ച്{affix}", f"2 {marker} അഞ്ച്{affix}")
        for text in ("", "2.05", "2.50", "0002.0050", "ID_0002.0050"):
            self.check(text, text)


if __name__ == "__main__":
    unittest.main()
