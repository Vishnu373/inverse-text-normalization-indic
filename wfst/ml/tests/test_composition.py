"""Plan rules 1-14, using approved vocabulary, not data/grammar as an oracle."""

from pathlib import Path
from typing import ClassVar
import unittest

import pynini
from pynini.lib import rewrite


# These explicit forms are the approved plan vocabulary and its combining forms.
SMALL = (
    "പൂജ്യം", "ഒന്ന്", "രണ്ട്", "മൂന്ന്", "നാല്", "അഞ്ച്", "ആറ്", "ഏഴ്", "എട്ട്", "ഒമ്പത്",
    "പത്ത്", "പതിനൊന്ന്", "പന്ത്രണ്ട്", "പതിമൂന്ന്", "പതിനാല്", "പതിനഞ്ച്", "പതിനാറ്",
    "പതിനേഴ്", "പതിനെട്ട്", "പത്തൊമ്പത്",
)
TENS = ("ഇരുപത്", "മുപ്പത്", "നാല്പത്", "അമ്പത്", "അറുപത്", "എഴുപത്", "എൺപത്", "തൊണ്ണൂറ്")
ROOTS = ("ഇരുപത്ത", "മുപ്പത്ത", "നാല്പത്ത", "അമ്പത്ത", "അറുപത്ത", "എഴുപത്ത", "എൺപത്ത", "തൊണ്ണൂറ്റ")
UNITS = ("ൊന്ന്", "ിരണ്ട്", "ിമൂന്ന്", "ിനാല്", "ിയഞ്ച്", "ിയാറ്", "ിയേഴ്", "ിയെട്ട്", "ൊമ്പത്")
HUNDREDS = ("നൂറ്", "ഇരുനൂറ്", "മുന്നൂറ്", "നാനൂറ്", "അഞ്ഞൂറ്", "അറുനൂറ്", "എഴുനൂറ്", "എണ്ണൂറ്", "തൊള്ളായിരം")
HUNDRED_LINKS = ("നൂറ്റി", "ഇരുനൂറ്റി", "മുന്നൂറ്റി", "നാനൂറ്റി", "അഞ്ഞൂറ്റി", "അറുനൂറ്റി", "എഴുനൂറ്റി", "എണ്ണൂറ്റി", "തൊള്ളായിരത്തി")
THOUSAND_PREFIXES = (
    "", "ആ", "രണ്ടാ", "മൂവാ", "നാലാ", "അയ്യാ", "ആറാ", "ഏഴാ", "എണ്ണാ", "ഒമ്പതിനാ",
    "പതിനാ", "പതിനൊന്നാ", "പന്ത്രണ്ടാ", "പതിമൂന്നാ", "പതിനാലാ", "പതിനഞ്ചാ", "പതിനാറാ",
    "പതിനേഴാ", "പതിനെട്ടാ", "പത്തൊമ്പതിനാ",
)
THOUSAND_TENS = ("ഇരുപതിനാ", "മുപ്പതിനാ", "നാല്പതിനാ", "അമ്പതിനാ", "അറുപതിനാ", "എഴുപതിനാ", "എൺപതിനാ", "തൊണ്ണൂറാ")
THOUSAND_UNITS = ("ൊന്നാ", "ിരണ്ടാ", "ിമൂന്നാ", "ിനാലാ", "ിയഞ്ചാ", "ിയാറാ", "ിയേഴാ", "ിയെട്ടാ", "ൊമ്പതിനാ")


def small_word(value: int) -> str:
    assert 0 <= value < 100
    if value < 20:
        return SMALL[value]
    tens, unit = divmod(value, 10)
    return TENS[tens - 2] if unit == 0 else ROOTS[tens - 2] + UNITS[unit - 1]


def thousand_prefix(value: int) -> str:
    assert 1 <= value < 100
    if value < 20:
        return THOUSAND_PREFIXES[value]
    tens, unit = divmod(value, 10)
    return THOUSAND_TENS[tens - 2] if unit == 0 else ROOTS[tens - 2] + THOUSAND_UNITS[unit - 1]


def spoken(value: int) -> str:
    """Canonical spaced composition from rules 6/8/10/12/14; no elisions."""
    assert 0 <= value <= 999_999_999
    if value < 100:
        return small_word(value)
    if value < 1000:
        count, rest = divmod(value, 100)
        return HUNDREDS[count - 1] if rest == 0 else HUNDRED_LINKS[count - 1] + " " + small_word(rest)
    if value < 100_000:
        count, rest = divmod(value, 1000)
        prefix = thousand_prefix(count)
        return prefix + "യിരം" if rest == 0 else prefix + "യിരത്തി " + spoken(rest)
    scale = 100_000 if value < 10_000_000 else 10_000_000
    count, rest = divmod(value, scale)
    head = "ഒരു" if count == 1 else small_word(count)
    if scale == 100_000:
        return head + " ലക്ഷം" if rest == 0 else head + " ലക്ഷത്തി " + spoken(rest)
    return head + " കോടി" if rest == 0 else head + " കോടി " + spoken(rest)


class CompositionTests(unittest.TestCase):
    graph: ClassVar[pynini.Fst]

    @classmethod
    def setUpClass(cls) -> None:
        cls.graph = pynini.Fst.read(str(Path(__file__).parents[1] / "assets" / "numbers.fst"))

    def check(self, text: str, expected: str) -> None:
        with self.subTest(text=text, expected=expected):
            actual = rewrite.one_top_rewrite(
                pynini.escape(text), self.graph,
                input_token_type="byte", output_token_type="byte",
            )
            self.assertEqual(actual, expected)
            self.assertEqual(
                rewrite.one_top_rewrite(pynini.escape(actual), self.graph), actual,
                "Normalized text must be idempotent",
            )

    def test_all_81_compound_forms(self) -> None:
        # 8 decades x 9 units, plus the approved alternate 40-root x 9 units.
        roots = tuple(enumerate(ROOTS, 2)) + ((4, "നാൽപ്പത്ത"),)
        for tens, root in roots:
            for unit, suffix in enumerate(UNITS, 1):
                self.check(root + suffix, str(tens * 10 + unit))

    def test_all_0_to_999(self) -> None:
        for value in range(1000):
            self.check(spoken(value), str(value))

    def test_spaced_tens_and_spoken_unit_variants(self) -> None:
        roots = tuple(enumerate(ROOTS, 2)) + ((4, "നാൽപ്പത്ത"),)
        units = tuple((word, value) for value, word in enumerate(SMALL[1:10], 1))
        units += (("നാലേ", 4), ("ഏഴേ", 7))
        for tens, root in roots:
            for word, value in units:
                for separator in (" ", "\t", "\u00a0"):
                    self.check(f"{root}ി{separator}{word}", str(tens * 10 + value))
        self.check("മുന്നൂറ്റി അമ്പത്തി ഏഴ്", "357")
        self.check("ഇവിടെ മുപ്പത്തി നാലേ ഉള്ളൂ", "ഇവിടെ 34 ഉള്ളൂ")
        self.check("ഇവിടെ നാലേ ഉള്ളൂ", "ഇവിടെ 4 ഉള്ളൂ")
        self.check("ഇരുപത്തി നാല് കോടി മുപ്പത്തി ഏഴേ", "240000037")
        self.check("തൊണ്ണൂറ്റി പൂജ്യം", "തൊണ്ണൂറ്റി 0")
        self.check("തൊണ്ണൂറ്റി പത്ത്", "തൊണ്ണൂറ്റി 10")

    def test_spoken_ending_on_every_whole_number_form(self) -> None:
        # A spoken final "േ" stands for "്" on any complete whole number.
        samples = list(range(1000)) + [
            1001, 2005, 12345, 99999, 100004, 2500004, 10000001, 999999999,
        ]
        for value in samples:
            word = spoken(value)
            if word.endswith("്"):
                variant = word[:-1] + "േ"
                self.check(variant, str(value))
                self.check(f"ഇവിടെ {variant} ഉള്ളൂ", f"ഇവിടെ {value} ഉള്ളൂ")
        self.check("മുപ്പത്തിനാലേ", "34")
        self.check("നൂറ്റി ഒന്നേ", "101")
        self.check("ഇരുപത്തി അഞ്ച് ലക്ഷത്തി നാലേ", "2500004")

    def test_all_thousand_multipliers(self) -> None:
        # Rules 7-10 allow every count, with exact and 1/2/3-digit remainders.
        for count in range(1, 100):
            for rest in (0, 1, 9, 10, 99, 100, 999):
                value = count * 1000 + rest
                self.check(spoken(value), str(value))

    def test_all_exact_lakh_and_crore_counts(self) -> None:
        for scale in (100_000, 10_000_000):
            for count in range(1, 100):
                self.check(spoken(count * scale), str(count * scale))

    def test_seeded_large_compositions(self) -> None:
        # Same bounded LCG in Rust: reproducible cross-runtime coverage, no deps.
        state = 0x5EED
        for scale in (100_000, 10_000_000):
            for _ in range(256):
                state = (state * 1_664_525 + 1_013_904_223) & 0xFFFF_FFFF
                count = state % 99 + 1
                state = (state * 1_664_525 + 1_013_904_223) & 0xFFFF_FFFF
                rest = state % (scale - 1) + 1
                value = count * scale + rest
                self.check(spoken(value), str(value))

    def test_zero_gaps_and_nine_upper_bounds(self) -> None:
        # Omitted scale chunks must not shift the remaining place values.
        for crore in (1, 9, 10, 99):
            for lakh in (0, 1, 99):
                for thousand in (0, 1, 99):
                    for rest in (0, 1, 9, 99, 100, 999):
                        value = crore * 10_000_000 + lakh * 100_000 + thousand * 1000 + rest
                        self.check(spoken(value), str(value))
        for digits in range(1, 10):
            value = 10 ** digits - 1
            self.check(spoken(value), str(value))

    def test_repeated_numbers_in_sentences(self) -> None:
        for value in (0, 9, 19, 99, 101, 999, 9000, 19000, 90000, 99000,
                      100001, 200500, 2500125, 9999999, 10000001, 20005000,
                      250000125, 990000000, 999000001, 999999999):
            text = f"എണ്ണം [{spoken(value)}]; വീണ്ടും {spoken(value)}; പൂജ്യം."
            self.check(text, f"എണ്ണം [{value}]; വീണ്ടും {value}; 0.")

    def test_punctuation_unicode_and_identifiers(self) -> None:
        value = 990010009
        for left, right in (("[", "]"), ("(", ")"), ("{", "}"), ("\"", "\""),
                            ("«", "»"), ("₹", "!"), ("\u00a0", "\u2003"), ("\n", "\t")):
            self.check(left + spoken(value) + right, left + str(value) + right)
        # Letters, marks, digits, connector punctuation and joiners protect IDs.
        word = small_word(99)
        for affix in ("a", "മ", "7", "_", "\u203f", "\u200c", "\u200d", "\u0301"):
            self.check(affix + word, affix + word)
            self.check(word + affix, word + affix)
        for text in ("00009", "ID_00009", "൯", "പൂജ്യംx"):
            self.check(text, text)
        for separator in (" ", "\t", "\u00a0"):
            self.check("നൂറ്റി" + separator + small_word(25), "125")

    def test_composition_does_not_cross_lines(self) -> None:
        for separator in (" ", "\t", "\u00a0"):
            self.check("ഒരു കോടി" + separator + "ഒന്ന്", "10000001")
        for separator in ("\n", "\r\n"):
            self.check("ഒരു കോടി" + separator + "ഒന്ന്", "10000000" + separator + "1")


if __name__ == "__main__":
    unittest.main()
