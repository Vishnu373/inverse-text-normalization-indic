"""Fraction composition with independent numeral vocabulary and literal outputs."""

import unittest
from typing import ClassVar

import pynini
from pynini.lib import rewrite

from wfst.ml.postprocess import reorder_fractions
from wfst.ml.rules.numbers import build_graph
from wfst.ml.rules.numbers.cardinal import build_numbers
from wfst.ml.rules.numbers.fraction import build_fractions
from wfst.ml.tests.test_composition import spoken


class FractionTests(unittest.TestCase):
    graph: ClassVar[pynini.Fst]

    @classmethod
    def setUpClass(cls) -> None:
        cls.graph = build_graph()

    def check(self, text: str, expected: str) -> None:
        with self.subTest(text=text):
            actual = reorder_fractions(rewrite.one_top_rewrite(pynini.escape(text), self.graph))
            self.assertEqual(actual, expected)
            self.assertEqual(
                reorder_fractions(rewrite.one_top_rewrite(pynini.escape(actual), self.graph)), actual,
            )

    def test_common_fractions_and_leg_spelling(self) -> None:
        for word, expected in (
            ("അര", "1/2"), ("കാൽ", "1/4"), ("മുക്കാൽ", "3/4"), ("കാല്", "കാല്"),
        ):
            self.check(word, expected)
        self.check(
            "കാല് വേദനിക്കുന്നു; കാൽ മണിക്കൂർ വിശ്രമിക്കാം.",
            "കാല് വേദനിക്കുന്നു; 1/4 മണിക്കൂർ വിശ്രമിക്കാം.",
        )
        self.check("അരിവാൾ കാലിൽ തട്ടി.", "അരിവാൾ കാലിൽ തട്ടി.")
        self.check("അര കിലോ", "1/2 കിലോ")

    def test_productive_joins_reuse_whole_numbers(self) -> None:
        for value in list(range(1, 101)) + [125, 999, 1001, 200005, 10000001, 999999999]:
            word = spoken(value)
            if not word.endswith("്"):
                continue
            self.check(word[:-1] + "ര", f"{value} 1/2")
            for tail, expected in (("കാൽ", "1/4"), ("മുക്കാൽ", "3/4")):
                for separator in ("", " ", "\t", "\u00a0"):
                    self.check(word[:-1] + "േ" + separator + tail, f"{value} {expected}")
        for text, expected in (
            ("ഇരുപത്തഞ്ചര", "25 1/2"), ("നൂറ്റി ഒന്നര", "101 1/2"),
        ):
            self.check(text, expected)

    def test_plain_adjacency_preserves_fraction_words(self) -> None:
        for value in (0, 5, 25, 900, 1000, 1001, 100000, 10000000, 999999999):
            for word in ("അര", "കാൽ", "മുക്കാൽ"):
                for separator in (" ", "  ", "\t", "\u00a0", " \t\u00a0"):
                    self.check(f"{spoken(value)}{separator}{word}", f"{value}{separator}{word}")
        for text, expected in (
            ("അഞ്ച് കാൽ മേശ", "5 കാൽ മേശ"),
            ("അഞ്ച് കാല് മേശ", "5 കാല് മേശ"),
            ("രണ്ടേ അര", "2 അര"),
            ("അഞ്ച്, കാൽ", "5, 1/4"),
            ("5 കാൽ മേശ", "5 കാൽ മേശ"),
            ("0005\tഅര", "0005\tഅര"),
        ):
            self.check(text, expected)

    def test_bar_operands_and_preserved_components(self) -> None:
        values = (0, 1, 2, 3, 4, 7, 99, 100, 1001, 200005, 10000001, 999999999)
        for numerator in values:
            for denominator in values[1:]:
                self.check(
                    f"{spoken(numerator)} ബാർ {spoken(denominator)}",
                    f"{numerator}/{denominator}",
                )
        self.check("രണ്ട് ബാർ നാല്", "2/4")
        self.check("അഞ്ച് ബാർ മൂന്ന്", "5/3")
        self.check("പൂജ്യം ബാർ ഏഴ്", "0/7")

    def test_fractional_lakh_and_crore_amounts(self) -> None:
        for count in range(1, 100):
            word = spoken(count)
            for scale, multiplier in (("ലക്ഷം", 100000), ("കോടി", 10000000)):
                for tail, numerator, denominator in (
                    ("ര", 1, 2), ("േകാൽ", 1, 4), ("േ മുക്കാൽ", 3, 4),
                ):
                    self.check(
                        f"{word[:-1]}{tail} {scale}",
                        str(count * multiplier + numerator * multiplier // denominator),
                    )
        for text, expected in (
            ("രണ്ടര ലക്ഷം രൂപ", "250000 രൂപ"),
            ("ഒന്നേകാൽ ലക്ഷം", "125000"),
            ("മൂന്നേ മുക്കാൽ കോടി", "37500000"),
            ("അര ലക്ഷം", "50000"), ("കാൽ ലക്ഷം", "25000"),
            ("മുക്കാൽ ലക്ഷം", "75000"), ("അര കോടി", "5000000"),
            ("കാൽ കോടി", "2500000"), ("മുക്കാൽ കോടി", "7500000"),
        ):
            self.check(text, expected)
        for separator in (" ", "  ", "\t", "\u00a0", " \t\u00a0"):
            self.check(f"രണ്ടര{separator}ലക്ഷം", "250000")
            self.check(f"ഒന്നേ{separator}കാൽ{separator}കോടി", "12500000")
            self.check(f"അര{separator}ലക്ഷം", "50000")
        for separator in ("\n", "\r\n"):
            self.check(f"രണ്ടര{separator}ലക്ഷം", f"2 1/2{separator}ലക്ഷം")
            self.check(f"അര{separator}കോടി", f"1/2{separator}കോടി")
        self.check("[രണ്ടര ലക്ഷം], (കാൽ കോടി)!", "[250000], (2500000)!")
        self.check("id_രണ്ടര ലക്ഷം", "id_രണ്ടര ലക്ഷം")
        self.check("രണ്ടര ലക്ഷം_id", "2 1/2 ലക്ഷം_id")

    def test_adjacency_guard_does_not_displace_compound_numbers(self) -> None:
        for separator in (" " * 5, " " * 6, " " * 10, " " * 11, "\t" * 8, "\u00a0" * 8):
            for text, expected in (
                ("രണ്ട് ബാർ അഞ്ച്", "2/5"),
                ("അഞ്ച് പോയിന്റ് രണ്ട്", "5.2"),
                ("നാലിൽ മൂന്ന്", "3/4"),
            ):
                self.check(f"{text}{separator}അര", f"{expected}{separator}1/2")
            self.check(
                f"നൂറ്റി{separator}അഞ്ച്{separator}കാൽ", f"105{separator}കാൽ",
            )

    def test_all_denominators_one_through_one_hundred(self) -> None:
        for denominator in range(1, 101):
            word = spoken(denominator)
            self.assertTrue(word.endswith("്"))
            for numerator in (0, 3, 325, 999999999):
                self.check(
                    f"{word[:-1]}ിൽ {spoken(numerator)}", f"{numerator}/{denominator}",
                )
        for text, expected in (
            ("മൂന്നിൽ ഒന്ന്", "1/3"), ("നാലിൽ രണ്ട്", "2/4"),
            ("പത്തിൽ അഞ്ച്", "5/10"), ("ഇരുപതിൽ അഞ്ച്", "5/20"),
            ("ഇരുപത്തി അഞ്ചിൽ അഞ്ച്", "5/25"), ("നാൽപ്പതിൽ രണ്ട്", "2/40"),
            ("ഒന്‍പതിൽ രണ്ട്", "2/9"),
        ):
            self.check(text, expected)

    def test_spoken_endings_in_complete_components(self) -> None:
        for value in (1, 4, 7, 15, 25, 100, 325, 1325, 200325, 10000325):
            word = spoken(value)
            self.assertTrue(word.endswith("്"))
            variant = word[:-1] + "േ"
            self.check(f"{variant} ബാർ {variant}", f"{value}/{value}")
            self.check(f"നാലിൽ {variant}", f"{value}/4")
            self.check(f"{variant} അര", f"{value} അര")

    def test_fused_one_out_of_each_denominator(self) -> None:
        for denominator in range(1, 101):
            word = spoken(denominator)
            self.assertTrue(word.endswith("്"))
            self.check(f"{word[:-1]}ിലൊന്ന്", f"1/{denominator}")
        for text, expected in (
            ("മൂന്നിലൊന്ന്", "1/3"), ("അഞ്ചിലൊന്ന്", "1/5"),
            ("ഇരുപത്തി അഞ്ചിലൊന്ന്", "1/25"), ("ഒന്‍പതിലൊന്ന്", "1/9"),
            ("[മൂന്നിലൊന്ന്], (അഞ്ചിലൊന്ന്)!", "[1/3], (1/5)!"),
            ("ജനസംഖ്യയുടെ മൂന്നിലൊന്ന് ഇവിടെ താമസിക്കുന്നു.",
             "ജനസംഖ്യയുടെ 1/3 ഇവിടെ താമസിക്കുന്നു."),
            ("സീറ്റുകളിൽ അഞ്ചിലൊന്ന് ഒഴിഞ്ഞുകിടക്കുന്നു.",
             "സീറ്റുകളിൽ 1/5 ഒഴിഞ്ഞുകിടക്കുന്നു."),
            ("നൂറ്റൊന്നിലൊന്ന്", "നൂറ്റൊന്നിലൊന്ന്"),
            ("ആയിരത്തിലൊന്ന്", "ആയിരത്തിലൊന്ന്"),
            ("പൂജ്യത്തിലൊന്ന്", "പൂജ്യത്തിലൊന്ന്"),
            ("മൂന്നിലിരണ്ട്", "മൂന്നിലിരണ്ട്"),
            ("നാലിലൊന്ന് ബാർ മൂന്ന്", "1/4 ബാർ 3"),
        ):
            self.check(text, expected)

    def test_spacing_and_line_boundaries(self) -> None:
        for separator in (" ", "  ", "\t", "\u00a0", " \t\u00a0"):
            for text, expected in (
                (f"രണ്ട്{separator}ബാർ{separator}മൂന്ന്", "2/3"),
                (f"നാലിൽ{separator}രണ്ട്", "2/4"),
                (f"പൂജ്യം{separator}അര", f"0{separator}അര"),
                (f"രണ്ടേ{separator}മുക്കാൽ", "2 3/4"),
            ):
                self.check(f"  {text}\t", f"  {expected}\t")
        for separator in ("\n", "\r\n"):
            self.check(f"രണ്ട്{separator}ബാർ മൂന്ന്", f"2{separator}ബാർ 3")
            self.check(f"രണ്ട് ബാർ{separator}മൂന്ന്", f"2 ബാർ{separator}3")
            self.check(f"നാലിൽ{separator}രണ്ട്", f"നാലിൽ{separator}2")
            self.check(f"രണ്ടേ{separator}കാൽ", f"2{separator}1/4")

    def test_word_boundaries_and_unsupported_forms(self) -> None:
        for word in (
            "അര", "കാൽ", "മുക്കാൽ", "ഒന്നര", "രണ്ടേകാൽ", "രണ്ടേമുക്കാൽ",
            "മൂന്നിലൊന്ന്", "അഞ്ചിലൊന്ന്",
        ):
            for affix in ("a", "മ", "7", "_", "\u203f", "\u200c", "\u200d", "\u0301"):
                self.check(affix + word, affix + word)
                self.check(word + affix, word + affix)
        for text, expected in (
            ("[രണ്ട് ബാർ നാല്], (നാലിൽ മൂന്ന്)!", "[2/4], (3/4)!"),
            ("രണ്ട് ബാർ പൂജ്യം", "2 ബാർ 0"),
            ("പൂജ്യം ബാർ പൂജ്യം", "0 ബാർ 0"),
            ("നൂറ്റൊന്നിൽ രണ്ട്", "നൂറ്റൊന്നിൽ 2"),
            ("ആയിരത്തിൽ ഒന്ന്", "ആയിരത്തിൽ 1"),
            ("പൂജ്യത്തിൽ ഒന്ന്", "പൂജ്യത്തിൽ 1"),
            ("നാലിൽരണ്ട്", "നാലിൽരണ്ട്"),
            ("രണ്ട് bar മൂന്ന്", "2 bar 3"),
            ("രണ്ട് ബൈ മൂന്ന്", "2 ബൈ 3"),
            ("രണ്ട് ബാർ", "2 ബാർ"),
            ("ബാർ മൂന്ന്", "ബാർ 3"),
            ("രണ്ട്, ബാർ മൂന്ന്", "2, ബാർ 3"),
            ("നാലിൽ, രണ്ട്", "നാലിൽ, 2"),
            ("നാലിൽ രണ്ട്_id", "നാലിൽ രണ്ട്_id"),
            ("id_നാലിൽ രണ്ട്", "id_നാലിൽ 2"),
            ("രണ്ട് ബാർ മൂന്ന്_id", "2 ബാർ മൂന്ന്_id"),
            ("അര ബാർ മൂന്ന്", "1/2 ബാർ 3"),
            ("രണ്ട് ബാർ അര", "2 ബാർ 1/2"),
            ("2 ബാർ മൂന്ന്", "2 ബാർ 3"),
            ("നാലിൽ മൂന്ന് ഭാഗം", "3/4 ഭാഗം"),
            ("", ""), ("0002/0004 0/7 5/3 2/4 0 1/2", "0002/0004 0/7 5/3 2/4 0 1/2"),
        ):
            self.check(text, expected)

class FractionEntityTests(unittest.TestCase):
    def test_denominator_first_marks_spoken_order(self) -> None:
        fractions = build_fractions(build_numbers())
        for text, marked, expected in (
            ("അഞ്ചിൽ ഏഴ്", "\ue0005\ue0017\ue002", "7/5"),
            ("മൂന്നിലൊന്ന്", "\ue0003\ue0011\ue002", "1/3"),
            ("അഞ്ചിലൊന്ന്", "\ue0005\ue0011\ue002", "1/5"),
        ):
            with self.subTest(text=text):
                raw = rewrite.one_top_rewrite(text, fractions)
                self.assertEqual(raw, marked)
                self.assertEqual(reorder_fractions(raw), expected)

    def test_fixed_scale_suffixes(self) -> None:
        fractions = build_fractions(build_numbers())
        for text, expected in (
            ("രണ്ടര ലക്ഷം", "250000"),
            ("ഒന്നേകാൽ ലക്ഷം", "125000"),
            ("മൂന്നേ മുക്കാൽ കോടി", "37500000"),
            ("അര ലക്ഷം", "50000"), ("കാൽ ലക്ഷം", "25000"),
        ):
            with self.subTest(text=text):
                self.assertEqual(reorder_fractions(rewrite.one_top_rewrite(text, fractions)), expected)
        for text in (
            "നൂറര ലക്ഷം", "രണ്ടര ലക്ഷത്തി അയ്യായിരം",
            "രണ്ടര കോടി അഞ്ച് ലക്ഷം", "ഒരു കോടി രണ്ടര ലക്ഷം",
        ):
            with self.subTest(text=text):
                self.assertEqual((pynini.accep(text) @ fractions).num_states(), 0)

    def test_plain_adjacency_is_not_a_fraction(self) -> None:
        fractions = build_fractions(build_numbers())
        for number in ("പൂജ്യം", "അഞ്ച്", "ആയിരം", "ഒരു കോടി"):
            for word in ("അര", "കാൽ", "മുക്കാൽ"):
                text = f"{number} {word}"
                with self.subTest(text=text):
                    self.assertEqual(
                        (pynini.accep(pynini.escape(text)) @ fractions).num_states(), 0,
                    )

    def test_fraction_grammar_is_functional(self) -> None:
        deterministic = pynini.determinize(
            build_fractions(build_numbers()), det_type="functional",
        )
        self.assertGreater(deterministic.num_states(), 0)

    def test_fraction_output_range(self) -> None:
        outputs = build_fractions(build_numbers()).project("output").rmepsilon().optimize()
        digit = pynini.union(*"0123456789")
        positive = pynini.union(*"123456789") + pynini.closure(digit, 0, 8)
        integer = pynini.union("0", positive)
        count = pynini.union(*"123456789") + digit.ques
        scaled = (count.ques + pynini.union(
            "25000", "50000", "75000", "2500000", "5000000", "7500000",
        )).optimize()
        marked = (
            "\ue000" + pynini.union(*(str(value) for value in range(1, 101)))
            + "\ue001" + integer + "\ue002"
        ).optimize()
        expected = (
            integer + "/" + positive
            | positive + " " + pynini.union("1/2", "1/4", "3/4")
            | scaled
            | marked
        ).optimize()
        self.assertEqual(pynini.difference(outputs, expected).num_states(), 0)
        self.assertTrue(pynini.equivalent((outputs @ positive).optimize(), scaled))
        pure_fractions = (integer + "/" + positive).optimize()
        self.assertTrue(pynini.equivalent((outputs @ pure_fractions).optimize(), pure_fractions))
        self.assertTrue(pynini.equivalent((outputs @ marked).optimize(), marked))


if __name__ == "__main__":
    unittest.main()
